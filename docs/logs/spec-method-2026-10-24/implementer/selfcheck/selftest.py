#!/usr/bin/env python3
"""Independent-ish self-check client for the short-link service.

Only the frozen spec and run constraints were used to write this script.
Client side only: this process never serves HTTP.

Phases:
  functional  protocol / validation / encoding / TTL / concurrency / restart / SIGTERM
  perf        exact seeding + jump + create measurement from the frozen constraints
  all         functional then perf
"""
import argparse
import json
import math
import os
import re
import select
import shutil
import signal
import socket
import sqlite3
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

START_RE = re.compile(r"^listening on http://127\.0\.0\.1:(\d+)$")
ISO_RE = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
CODE_RE = re.compile(r"^[A-Za-z0-9]{7}$")
ALIAS_RE = re.compile(r"^[A-Za-z0-9_-]{4,32}$")


def valid_code(c):
    return bool(CODE_RE.match(c) or ALIAS_RE.match(c))

RESULTS = []
LOG_LOCK = threading.Lock()


def log(msg=""):
    with LOG_LOCK:
        print(msg, flush=True)


def check(cond, name, detail=""):
    RESULTS.append((bool(cond), name, detail))
    if cond:
        log(f"PASS {name}")
    else:
        log(f"FAIL {name}" + (f": {detail}" if detail else ""))
    return bool(cond)


def check_eq(got, want, name):
    return check(got == want, name, f"got={got!r} want={want!r}")


class Server:
    def __init__(self, binary, db_path, workdir, logdir, port=0, env_extra=None):
        self.binary = str(Path(binary).resolve())
        self.db_path = str(db_path)
        self.workdir = str(workdir)
        self.logdir = Path(logdir)
        self.port_req = port
        self.port = None
        self.proc = None
        self.stdout_path = self.logdir / "server-stdout.log"
        self.stderr_path = self.logdir / "server-stderr.log"
        self.env_extra = env_extra or {}
        self.start_seconds = None
        self.stop_seconds = None

    def start(self):
        env = os.environ.copy()
        env["PORT"] = str(self.port_req)
        env["DB_PATH"] = self.db_path
        env.update(self.env_extra)
        self.out_f = open(self.stdout_path, "ab", buffering=0)
        self.err_f = open(self.stderr_path, "ab", buffering=0)
        t0 = time.monotonic()
        self.proc = subprocess.Popen(
            [self.binary],
            cwd=self.workdir,
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=self.err_f,
        )
        deadline = t0 + 10.0
        line = b""
        while time.monotonic() < deadline:
            if self.proc.poll() is not None:
                break
            r, _, _ = select.select([self.proc.stdout], [], [], 0.1)
            if not r:
                continue
            line = self.proc.stdout.readline()
            if line:
                break
        if not line:
            self.stop(force=True)
            raise RuntimeError(
                f"server did not print startup line; exit={self.proc.returncode if self.proc else None}; "
                f"stderr={self.read_stderr()!r}"
            )
        text = line.decode("utf-8", "replace").rstrip("\r\n")
        self.out_f.write(line)
        m = START_RE.match(text)
        if not m:
            self.stop(force=True)
            raise RuntimeError(f"bad startup line: {text!r}")
        self.port = int(m.group(1))
        self.start_seconds = time.monotonic() - t0
        log(f"INFO server pid={self.proc.pid} port={self.port} startup={self.start_seconds:.3f}s db={self.db_path}")
        return self.port

    def read_stderr(self):
        try:
            return self.stderr_path.read_text(errors="replace")
        except OSError:
            return ""

    def alive(self):
        return self.proc is not None and self.proc.poll() is None

    def stop(self, force=False):
        if self.proc is None:
            return True
        if self.proc.poll() is not None:
            self._close_files()
            return True
        t0 = time.monotonic()
        self.proc.send_signal(signal.SIGTERM)
        try:
            rc = self.proc.wait(timeout=5.0)
            self.stop_seconds = time.monotonic() - t0
            log(f"INFO SIGTERM exit rc={rc} in {self.stop_seconds:.3f}s")
        except subprocess.TimeoutExpired:
            self.stop_seconds = time.monotonic() - t0
            log(f"FAIL server did not exit within 5s after SIGTERM; killing")
            self.proc.kill()
            self.proc.wait(timeout=2)
            self._close_files()
            return False
        self._close_files()
        return True

    def _close_files(self):
        for attr in ("out_f", "err_f"):
            f = getattr(self, attr, None)
            if f is not None:
                try:
                    f.close()
                except Exception:
                    pass
                setattr(self, attr, None)


class Resp:
    def __init__(self, version, status, reason, headers, body):
        self.version = version
        self.status = status
        self.reason = reason
        self.headers = headers  # lowercase name -> list of values
        self.body = body
        self.raw_header_bytes = b""

    def header(self, name):
        vals = self.headers.get(name.lower())
        return vals[0] if vals else None

    def json(self):
        return json.loads(self.body.decode("utf-8"))


class RawConn:
    def __init__(self, port, timeout=15.0):
        self.sock = socket.create_connection(("127.0.0.1", port), timeout=timeout)
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        self.buf = b""

    def close(self):
        try:
            self.sock.close()
        except OSError:
            pass

    def send_raw(self, data):
        self.sock.sendall(data)

    def read_response(self):
        while b"\r\n\r\n" not in self.buf:
            chunk = self.sock.recv(65536)
            if not chunk:
                raise RuntimeError("connection closed before response headers")
            self.buf += chunk
        head, self.buf = self.buf.split(b"\r\n\r\n", 1)
        lines = head.split(b"\r\n")
        status_line = lines[0].decode("latin-1")
        parts = status_line.split(" ", 2)
        if len(parts) < 2 or not parts[0].startswith("HTTP/"):
            raise RuntimeError(f"bad status line {status_line!r}")
        version = parts[0]
        status = int(parts[1])
        reason = parts[2] if len(parts) > 2 else ""
        headers = {}
        for raw in lines[1:]:
            if b":" not in raw:
                raise RuntimeError(f"bad header line {raw!r}")
            name, value = raw.split(b":", 1)
            name = name.decode("latin-1").strip().lower()
            value = value.decode("latin-1").strip()
            headers.setdefault(name, []).append(value)
        if "transfer-encoding" in headers:
            raise RuntimeError("unexpected Transfer-Encoding in response")
        cl = headers.get("content-length")
        if cl is None:
            raise RuntimeError("missing Content-Length in response")
        if len(cl) != 1 or not cl[0].isdigit():
            raise RuntimeError(f"bad Content-Length {cl!r}")
        n = int(cl[0])
        total = len(self.buf)
        while total < n:
            chunk = self.sock.recv(max(4096, n - total))
            if not chunk:
                raise RuntimeError(f"connection closed with {total}/{n} body bytes")
            self.buf += chunk
            total = len(self.buf)
        body, self.buf = self.buf[:n], self.buf[n:]
        resp = Resp(version, status, reason, headers, body)
        resp.raw_header_bytes = head + b"\r\n\r\n"
        if n != len(body):
            raise RuntimeError("Content-Length mismatch")
        return resp

    def request(self, method, path, body=None, headers=None, http_version="HTTP/1.1"):
        headers = dict(headers or {})
        lines = [f"{method} {path} {http_version}"]
        if "Host" not in headers:
            lines.append("Host: 127.0.0.1")
        if body is not None and "Content-Length" not in headers:
            headers["Content-Length"] = str(len(body))
        if "Connection" not in headers:
            headers["Connection"] = "keep-alive"
        for k, v in headers.items():
            lines.append(f"{k}: {v}")
        raw = ("\r\n".join(lines) + "\r\n\r\n").encode("latin-1")
        if body is not None:
            raw += body
        self.send_raw(raw)
        return self.read_response()

    def post_json(self, path, obj=None, raw=None, content_type="application/json"):
        if raw is None:
            raw = json.dumps(obj, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        headers = {}
        if content_type is not None:
            headers["Content-Type"] = content_type
        return self.request("POST", path, body=raw, headers=headers)


def one_shot(port, method, path, body=None, headers=None):
    c = RawConn(port)
    try:
        return c.request(method, path, body=body, headers=headers)
    finally:
        c.close()


def assert_response(resp, status, name):
    ok = resp.status == status
    if not ok:
        return check(False, name, f"status={resp.status} body={resp.body[:300]!r}")
    cl = resp.header("content-length")
    ok = cl is not None and int(cl) == len(resp.body)
    return check(ok, name, f"Content-Length={cl} bodylen={len(resp.body)}")


def assert_error(resp, status, code, name):
    ok = assert_response(resp, status, name + " status")
    try:
        obj = resp.json()
    except Exception as e:
        return check(False, name + " json", f"{e}: {resp.body[:300]!r}")
    ok = check(isinstance(obj, dict) and set(obj.keys()) == {"error"} and obj["error"] == code,
               name + " body", f"got={obj!r}")
    return ok


def assert_json_object(resp, keys, name):
    try:
        obj = resp.json()
    except Exception as e:
        return None, check(False, name + " json", f"{e}: {resp.body[:300]!r}")
    ok = check(isinstance(obj, dict) and set(obj.keys()) == set(keys), name + " keys",
               f"got={sorted(obj.keys()) if isinstance(obj, dict) else type(obj)}")
    return obj, ok


def create(port, obj, content_type="application/json", raw=None):
    return one_shot(port, "POST", "/api/links", body=raw if raw is not None else
                    json.dumps(obj, ensure_ascii=False, separators=(",", ":")).encode("utf-8"),
                    headers={} if content_type is None else {"Content-Type": content_type})


def create_ok(port, obj, name):
    resp = create(port, obj)
    if not assert_response(resp, 201, name + " status"):
        return None
    data, ok = assert_json_object(resp, ["code", "short_url", "expires_at"], name + " fields")
    if not data:
        return None
    check(CODE_RE.match(data["code"]) is not None or (isinstance(obj, dict) and obj.get("alias")),
          name + " code shape", repr(data.get("code")))
    check_eq(data["short_url"], f"http://127.0.0.1:{port}/{data['code']}", name + " short_url")
    ct = resp.header("content-type")
    check_eq(ct, "application/json; charset=utf-8", name + " content-type")
    if data.get("expires_at") is not None:
        check(ISO_RE.match(data["expires_at"]) is not None, name + " expires_at format",
              repr(data.get("expires_at")))
    return data


def stats(port, code):
    return one_shot(port, "GET", f"/api/links/{code}")


def jump(port, code, conn=None):
    if conn is not None:
        return conn.request("GET", f"/{code}")
    return one_shot(port, "GET", f"/{code}")


def read_vmhwm(pid):
    text = Path(f"/proc/{pid}/status").read_text()
    for line in text.splitlines():
        if line.startswith("VmHWM:"):
            return int(line.split()[1])
    raise RuntimeError("VmHWM not found")


# ---------------------------------------------------------------- functional

def run_functional(server, logdir):
    port = server.port
    log("== functional: startup ==")
    check(server.port is not None and server.port > 0, "startup line parsed with real port",
          str(server.port))
    check(server.alive(), "server process alive after startup line")

    log("== functional: create + stats + jump ==")
    d1 = create_ok(port, {"url": "https://example.com"}, "create minimal")
    if not d1:
        return False
    code1 = d1["code"]
    check_eq(d1["expires_at"], None, "create minimal expires_at null")

    st = stats(port, code1)
    if assert_response(st, 200, "stats minimal status"):
        obj, ok = assert_json_object(st, ["code", "url", "clicks", "created_at", "expires_at", "expired"],
                                     "stats minimal")
        if obj:
            check_eq(obj["code"], code1, "stats code")
            check_eq(obj["url"], "https://example.com", "stats url")
            check_eq(obj["clicks"], 0, "stats clicks 0")
            check(ISO_RE.match(obj["created_at"]) is not None, "stats created_at format", obj["created_at"])
            check_eq(obj["expires_at"], None, "stats expires_at null")
            check_eq(obj["expired"], False, "stats expired false")
            created1 = obj["created_at"]

    jr = jump(port, code1)
    if assert_response(jr, 302, "jump status"):
        check_eq(jr.header("location"), "https://example.com", "jump Location exact")
        check_eq(jr.header("content-length"), "0", "jump Content-Length header")
        check_eq(jr.body, b"", "jump empty body")
    st = stats(port, code1)
    if assert_response(st, 200, "stats after jump status"):
        obj = st.json()
        check_eq(obj["clicks"], 1, "stats clicks after jump = 1")
        check_eq(obj["created_at"], created1, "stats created_at stable")

    d2 = create_ok(port, {"url": "https://example.com"}, "create duplicate url")
    if d2:
        check(d2["code"] != code1, "same URL gets different code", f"{d1['code']} vs {d2['code']}")

    assert_error(jump(port, "nosuch1"), 404, "not_found", "jump unknown")
    assert_error(stats(port, "nosuch1"), 404, "not_found", "stats unknown")
    assert_error(one_shot(port, "GET", "/api/links"), 404, "not_found", "stats missing code path")

    log("== functional: alias ==")
    da = create_ok(port, {"url": "https://example.com/a", "alias": "Abc1"}, "create alias Abc1")
    db = create_ok(port, {"url": "https://example.com/b", "alias": "abc1"}, "create alias abc1")
    if da and db:
        check_eq(da["code"], "Abc1", "alias code Abc1")
        check_eq(db["code"], "abc1", "alias code abc1")
        check(da["code"] != db["code"], "alias case sensitive")
    assert_error(create(port, {"url": "https://example.com/c", "alias": "Abc1"}),
                 409, "alias_taken", "duplicate alias")
    assert_error(create(port, {"url": "https://example.com/d", "alias": "api"}),
                 400, "invalid_alias", "reserved alias api")
    assert_error(create(port, {"url": "https://example.com/d", "alias": "health"}),
                 400, "invalid_alias", "reserved alias health")
    for bad in ["abc", "a" * 33, "ab c1", "abc!", "abc/1", "中文ab"]:
        assert_error(create(port, {"url": "https://example.com/e", "alias": bad}),
                     400, "invalid_alias", f"invalid alias {bad[:12]!r}")
    for good in ["aBc_", "a-b9", "A" * 32]:
        create_ok(port, {"url": "https://example.com/e", "alias": good}, f"valid alias {good[:8]!r}")
    dn = create_ok(port, {"url": "https://example.com/f", "alias": None}, "alias null")
    if dn:
        check(CODE_RE.match(dn["code"]) is not None, "alias null auto code", dn["code"])

    log("== functional: Content-Type ==")
    for ct in ["application/json; charset=utf-8", "Application/JSON",
               "application/json ;  Charset=UTF-8", "APPLICATION/JSON;charset=utf-8"]:
        create_ok(port, {"url": "https://example.com/ct"}, f"content-type {ct!r}")
    for ct in [None, "text/json", "application/jsonx", "application/json-patch+json"]:
        assert_error(create(port, {"url": "https://example.com/ct"}, content_type=ct),
                     400, "invalid_request", f"bad content-type {ct!r}")

    log("== functional: invalid JSON / fields ==")
    bad_bodies = [
        (b"", "empty body"),
        (b"{", "truncated object"),
        (b"[]", "array body"),
        (b"null", "null body"),
        (b'"x"', "string body"),
        (b"{}{}", "trailing garbage"),
        (b'{"url":123}', "url number"),
        (b'{"url":null}', "url null"),
        (b'{"url":true}', "url bool"),
        (b'{"url":"https://example.com","alias":123}', "alias number"),
        (b'{"url":"https://example.com","alias":true}', "alias bool"),
        (b'{"url":"https://example.com","ttl_seconds":true}', "ttl bool true"),
        (b'{"url":"https://example.com","ttl_seconds":false}', "ttl bool false"),
        (b'{"url":"https://example.com","ttl_seconds":1.5}', "ttl float"),
        (b'{"url":"https://example.com","ttl_seconds":1.0}', "ttl integral float"),
        (b'{"url":"https://example.com","ttl_seconds":"5"}', "ttl string"),
        (b'{"url":"https://example.com","ttl_seconds":0}', "ttl zero"),
        (b'{"url":"https://example.com","ttl_seconds":-1}', "ttl negative"),
        (b'{"url":"https://example.com","ttl_seconds":31536001}', "ttl too large"),
        (b'{"url":"https://example.com","ttl_seconds":999999999999999999999}', "ttl huge"),
        (b'{"url":"https://example.com",}', "trailing comma"),
        (b'{"url":"https://example.com"', "missing brace"),
        (b"{'url':'https://example.com'}", "single quotes"),
        (b'{"url":"https://example.com/\\q"}', "bad escape"),
        (b'{"url":"https://example.com/\\u12"}', "short unicode escape"),
        (b'{"url":\xff}', "invalid utf8"),
    ]
    for raw, label in bad_bodies:
        assert_error(create(port, None, raw=raw), 400, "invalid_request", f"bad body {label}")
    create_ok(port, {"url": "https://example.com/extra", "unknown": {"a": [1, True, None, "x"]}},
              "unknown fields ignored with nested value")

    log("== functional: URL rules ==")
    bad_urls = [
        "", "example.com", "http://", "https://", "http:///path", "ftp://example.com",
        "http://exa mple.com/path", "https://exa mple.com", "http://:80/path",
        "http://example.com/%2", "http://example.com/%zz", "http://example.com/100%",
        "http://example.com/\u0000", "http://example.com/\u0001x", "http://example.com/\u001f",
        "http://example.com/\u007f",
        "http://example.com/" + "a" * 2100,
    ]
    for u in bad_urls:
        assert_error(create(port, {"url": u}), 400, "invalid_url", f"bad url {u[:40]!r}")

    base = "https://example.com/"
    u2048 = base + "a" * (2048 - len(base))
    check_eq(len(u2048.encode()), 2048, "constructed 2048-byte url")
    create_ok(port, {"url": u2048}, "url exactly 2048 bytes")
    u2049 = base + "a" * (2049 - len(base))
    assert_error(create(port, {"url": u2049}), 400, "invalid_url", "url 2049 bytes")

    spaced = create_ok(port, {"url": "https://example.com/path with space"}, "url path space accepted")
    if spaced:
        st = stats(port, spaced["code"])
        obj = st.json()
        check_eq(obj["url"], "https://example.com/path with space", "stats preserves url space")
        jr = jump(port, spaced["code"])
        check_eq(jr.header("location"), "https://example.com/path%20with%20space",
                 "Location encodes path space")

    log("== functional: Location percent rules ==")
    cases = [
        ("https://example.com/a%2Fb?x=%20&y=1", "https://example.com/a%2Fb?x=%20&y=1"),
        ("https://example.com/a%2fb", "https://example.com/a%2fb"),
        ("https://example.com/路径?x=中 文#片段",
         "https://example.com/%E8%B7%AF%E5%BE%84?x=%E4%B8%AD%20%E6%96%87#%E7%89%87%E6%AE%B5"),
        ("https://example.com/a?b=c&d=e#f", "https://example.com/a?b=c&d=e#f"),
        ("http://127.0.0.1:9/x", "http://127.0.0.1:9/x"),
    ]
    for src, want in cases:
        d = create_ok(port, {"url": src}, f"create encoded url {src[:28]!r}")
        if not d:
            continue
        jr = jump(port, d["code"])
        if assert_response(jr, 302, f"jump encoded {src[:20]!r} status"):
            check_eq(jr.header("location"), want, f"Location exact for {src[:28]!r}")
        st = stats(port, d["code"])
        if assert_response(st, 200, f"stats encoded {src[:20]!r} status"):
            check_eq(st.json()["url"], src, f"stats url exact for {src[:28]!r}")

    log("== functional: TTL ==")
    ttl_d = create_ok(port, {"url": "https://example.com/ttl", "ttl_seconds": 1}, "create ttl=1")
    if ttl_d:
        st = stats(port, ttl_d["code"])
        obj = st.json()
        # Exact numeric check through parsed ISO seconds.
        def iso_to_epoch(s):
            import datetime
            return int(datetime.datetime.strptime(s, "%Y-%m-%dT%H:%M:%SZ")
                       .replace(tzinfo=datetime.timezone.utc).timestamp())
        check_eq(iso_to_epoch(obj["expires_at"]) - iso_to_epoch(obj["created_at"]), 1,
                 "expires_at = created_at + ttl")
        jr = jump(port, ttl_d["code"])
        assert_response(jr, 302, "jump before ttl expiry")
        st = stats(port, ttl_d["code"])
        check_eq(st.json()["clicks"], 1, "ttl clicks before expiry")
        deadline = time.time() + 4
        while time.time() < iso_to_epoch(obj["expires_at"]) + 1 and time.time() < deadline:
            time.sleep(0.05)
        st = stats(port, ttl_d["code"])
        if assert_response(st, 200, "stats expired status"):
            eo, ok = assert_json_object(st, ["code", "url", "clicks", "created_at", "expires_at", "expired"],
                                        "stats expired")
            if eo:
                check_eq(eo["expired"], True, "stats expired true")
                check_eq(eo["clicks"], 1, "stats clicks frozen after expiry")
                check_eq(eo["expires_at"], obj["expires_at"], "stats expires_at preserved")
        assert_error(jump(port, ttl_d["code"]), 410, "expired", "jump expired")
        st = stats(port, ttl_d["code"])
        check_eq(st.json()["clicks"], 1, "clicks unchanged after expired jump")
        # Already-expired link can still be re-created under a different alias/code.
        dnull = create_ok(port, {"url": "https://example.com/ttl", "ttl_seconds": None},
                          "ttl null means no expiry")
        if dnull:
            check_eq(dnull["expires_at"], None, "ttl null expires_at null")

    log("== functional: keep-alive and pipelining ==")
    ka = create_ok(port, {"url": "https://example.com/ka"}, "create keepalive link")
    if ka:
        c = RawConn(port)
        try:
            r1 = c.request("GET", f"/api/links/{ka['code']}")
            r2 = c.request("GET", f"/{ka['code']}")
            r3 = c.request("GET", f"/api/links/{ka['code']}")
            check_eq(r1.status, 200, "keep-alive request 1")
            check_eq(r2.status, 302, "keep-alive request 2")
            check_eq(r3.status, 200, "keep-alive request 3")
            check_eq(r3.json()["clicks"], 1, "keep-alive clicks")
            # Pipelined pair on a fresh connection.
            c2 = RawConn(port)
            try:
                req = (f"GET /api/links/{ka['code']} HTTP/1.1\r\nHost: x\r\nConnection: keep-alive\r\n\r\n"
                       f"GET /{ka['code']} HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n").encode()
                c2.send_raw(req)
                p1 = c2.read_response()
                p2 = c2.read_response()
                check_eq(p1.status, 200, "pipelined response 1")
                check_eq(p2.status, 302, "pipelined response 2")
            finally:
                c2.close()
        finally:
            c.close()

    log("== functional: concurrent alias create (20) ==")
    con_alias = "conc1"
    barrier = threading.Barrier(20)
    outcomes = []
    lock = threading.Lock()

    def alias_worker(i):
        try:
            c = RawConn(port)
            barrier.wait(timeout=10)
            r = c.post_json("/api/links", {"url": f"https://example.com/conc/{i}", "alias": con_alias})
            c.close()
            with lock:
                outcomes.append((r.status, r.body))
        except Exception as e:
            with lock:
                outcomes.append(("exc", repr(e).encode()))

    threads = [threading.Thread(target=alias_worker, args=(i,)) for i in range(20)]
    for t in threads:
        t.start()
    for t in threads:
        t.join(timeout=20)
    n201 = sum(1 for s, _ in outcomes if s == 201)
    n409 = sum(1 for s, _ in outcomes if s == 409)
    check_eq(n201, 1, "concurrent alias exactly one 201")
    check_eq(n409, 19, "concurrent alias nineteen 409")
    for s, b in outcomes:
        if s == 409:
            try:
                check_eq(json.loads(b), {"error": "alias_taken"}, "concurrent alias 409 body")
                break
            except Exception:
                pass
    st = stats(port, con_alias)
    if st.status == 200:
        check_eq(st.json()["clicks"], 0, "concurrent alias stats exists")

    log("== functional: 50 concurrent jumps ==")
    cj = create_ok(port, {"url": "https://example.com/jump50"}, "create 50-jump link")
    if cj:
        barrier = threading.Barrier(50)
        sts = []
        lock = threading.Lock()

        def jump_worker(i):
            try:
                c = RawConn(port)
                barrier.wait(timeout=10)
                r = c.request("GET", f"/{cj['code']}")
                c.close()
                with lock:
                    sts.append(r.status)
            except Exception as e:
                with lock:
                    sts.append(("exc", str(e)))

        threads = [threading.Thread(target=jump_worker, args=(i,)) for i in range(50)]
        for t in threads:
            t.start()
        for t in threads:
            t.join(timeout=20)
        check(sum(1 for s in sts if s == 302) == 50, "50 concurrent jumps all 302",
              repr([s for s in sts if s != 302][:5]))
        st = stats(port, cj["code"])
        if st.status == 200:
            check_eq(st.json()["clicks"], 50, "50 concurrent jumps clicks exactly 50")

    log("== functional: oversized body ==")
    big = b'{"url":"https://example.com/big"}' + b" " * (1024 * 1024)
    check(len(big) > 1024 * 1024, "oversize body is > 1 MiB", str(len(big)))
    assert_error(create(port, None, raw=big), 400, "invalid_request", "body over 1 MiB")
    create_ok(port, {"url": "https://example.com/after-big"}, "server alive after oversize body")

    log("== functional: SIGTERM / restart / persistence ==")
    pd = create_ok(port, {"url": "https://example.com/persist", "alias": "persist1", "ttl_seconds": 3600},
                   "create persist link")
    if pd:
        j1 = jump(port, "persist1")
        j2 = jump(port, "persist1")
        check_eq(j1.status, 302, "persist jump 1")
        check_eq(j2.status, 302, "persist jump 2")
        st = stats(port, "persist1")
        before = st.json()
        check_eq(before["clicks"], 2, "persist clicks before restart = 2")
        db_file = Path(server.db_path)
        check(db_file.exists(), "sqlite db file exists before shutdown", server.db_path)
        exit_ok = server.stop()
        check(exit_ok and server.stop_seconds is not None and server.stop_seconds < 5.0,
              "SIGTERM exits within 5s", f"{server.stop_seconds}")
        check_eq(server.proc.returncode, 0, "SIGTERM clean exit code")
        # Inspect committed rows directly in SQLite (service is down).
        try:
            con = sqlite3.connect(server.db_path)
            row = con.execute("SELECT code,url,clicks,created_at,expires_at FROM links WHERE code=?",
                              ("persist1",)).fetchone()
            con.close()
            check(row is not None, "persist row present in sqlite after shutdown")
            if row:
                check_eq(row[0], "persist1", "sqlite code")
                check_eq(row[1], "https://example.com/persist", "sqlite url")
                check_eq(row[2], 2, "sqlite clicks committed")
        except Exception as e:
            check(False, "direct sqlite inspection", repr(e))

        s2 = Server(server.binary, server.db_path, server.workdir, Path(logdir) / "restart", port=0)
        Path(logdir, "restart").mkdir(parents=True, exist_ok=True)
        try:
            s2.start()
            p2 = s2.port
            st = one_shot(p2, "GET", "/api/links/persist1")
            if assert_response(st, 200, "stats after restart status"):
                after = st.json()
                for k in ["code", "url", "clicks", "created_at", "expires_at", "expired"]:
                    check_eq(after[k], before[k], f"restart preserves {k}")
            r = one_shot(p2, "GET", "/persist1")
            check_eq(r.status, 302, "jump after restart")
            check_eq(r.header("location"), "https://example.com/persist", "Location after restart")
            st = one_shot(p2, "GET", "/api/links/persist1")
            check_eq(st.json()["clicks"], 3, "clicks continue after restart")
            check_eq(one_shot(p2, "GET", "/nosuch2").status, 404, "unknown still 404 after restart")
            exit_ok2 = s2.stop()
            check(exit_ok2, "second SIGTERM exits within 5s")
            # Reopen once more to prove the post-restart jump was committed.
            s3 = Server(server.binary, server.db_path, server.workdir, Path(logdir) / "restart2", port=0)
            Path(logdir, "restart2").mkdir(parents=True, exist_ok=True)
            s3.start()
            st = one_shot(s3.port, "GET", "/api/links/persist1")
            check_eq(st.json()["clicks"], 3, "clicks persisted after second restart")
            s3.stop()
        finally:
            if s2.alive():
                s2.stop()
        # keep `server` object marked stopped
        server.proc = None

    failed = [r for r in RESULTS if not r[0]]
    log(f"SUMMARY functional: {len(RESULTS) - len(failed)} passed, {len(failed)} failed")
    return not failed


# ---------------------------------------------------------------------- perf

class BenchConn:
    def __init__(self, port):
        self.sock = socket.create_connection(("127.0.0.1", port), timeout=30.0)
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        self.buf = b""

    def close(self):
        try:
            self.sock.close()
        except OSError:
            pass

    def send(self, data):
        self.sock.sendall(data)

    def read_resp(self):
        while b"\r\n\r\n" not in self.buf:
            chunk = self.sock.recv(65536)
            if not chunk:
                raise RuntimeError("closed before headers")
            self.buf += chunk
        head, self.buf = self.buf.split(b"\r\n\r\n", 1)
        lines = head.split(b"\r\n")
        status = int(lines[0].split(b" ")[1])
        n = 0
        for line in lines[1:]:
            if line.lower().startswith(b"content-length:"):
                n = int(line.split(b":", 1)[1].strip())
        while len(self.buf) < n:
            chunk = self.sock.recv(max(4096, n - len(self.buf)))
            if not chunk:
                raise RuntimeError("closed before body")
            self.buf += chunk
        body, self.buf = self.buf[:n], self.buf[n:]
        return status, body


def p95(lat_ms):
    s = sorted(lat_ms)
    idx = math.ceil(0.95 * len(s)) - 1
    return round(s[idx], 2)


def csv_append(path, line):
    with open(path, "a") as f:
        f.write(line)


def median(vals):
    return sorted(vals)[len(vals) // 2]


def parallel_requests(port, tasks, timeout=60):
    """tasks: list of (list_of_request_bytes, validator(status,body)).
    Each task is one client = one OS process (frozen constraints say
    "50 concurrent clients"; processes keep the client-side overhead out of
    the measured latencies, unlike Python threads which convoy on the GIL).
    One keep-alive connection per client per invocation."""
    import multiprocessing as mp

    def worker(i, port, reqs, kind, barrier, q):
        try:
            conn = BenchConn(port)
            barrier.wait(timeout=timeout)
            validator = validate_jump if kind == "jump" else validate_create
            lats = []
            for raw in reqs:
                t0 = time.perf_counter_ns()
                conn.send(raw)
                status, body = conn.read_resp()
                t1 = time.perf_counter_ns()
                validator(status, body)
                lats.append((t1 - t0) / 1e6)
            conn.close()
            q.put((i, None, lats))
        except Exception as e:
            q.put((i, repr(e), None))

    ctx = mp.get_context("fork")
    q = ctx.Queue()
    barrier = ctx.Barrier(len(tasks))
    procs = []
    for i, (reqs, validator) in enumerate(tasks):
        kind = "jump" if validator is validate_jump else "create"
        procs.append(ctx.Process(target=worker, args=(i, port, reqs, kind, barrier, q)))
    for p in procs:
        p.start()
    by_index = {}
    errors = []
    for _ in procs:
        i, err, lats = q.get(timeout=timeout)
        if err is not None:
            errors.append(err)
        else:
            by_index[i] = lats
    for p in procs:
        p.join(timeout=30)
    if errors:
        raise RuntimeError(f"bench worker errors: {errors[:3]}")
    if len(by_index) != len(tasks):
        raise RuntimeError("bench worker did not finish")
    return [by_index[i] for i in range(len(tasks))]


def encode_post(url, alias=None):
    obj = {"url": url}
    if alias is not None:
        obj["alias"] = alias
    body = json.dumps(obj, separators=(",", ":")).encode()
    return (b"POST /api/links HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\n"
            b"Content-Length: " + str(len(body)).encode() + b"\r\nConnection: keep-alive\r\n\r\n" + body)


def jump_req(code):
    return f"GET /{code} HTTP/1.1\r\nHost: x\r\nConnection: keep-alive\r\n\r\n".encode()


def validate_jump(status, body):
    if status != 302 or body != b"":
        raise RuntimeError(f"jump status={status} body={body[:80]!r}")


def validate_create(status, body):
    if status != 201:
        raise RuntimeError(f"create status={status} body={body[:120]!r}")
    obj = json.loads(body)
    if not valid_code(obj.get("code", "")):
        raise RuntimeError(f"bad created code {obj.get('code')!r}")


def run_perf(server, logdir, csv_path):
    port = server.port
    pid = server.proc.pid
    summary = {}
    log("== perf: seed 1000 links (seed0001-seed1000) ==")
    t0 = time.monotonic()
    tasks = []
    for w in range(50):
        reqs = []
        for j in range(20):
            idx = w * 20 + j + 1
            code = f"seed{idx:04d}"
            reqs.append(encode_post(f"https://example.com/seed/{code}", code))
        tasks.append((reqs, validate_create))
    parallel_requests(port, tasks, timeout=120)
    seed_secs = time.monotonic() - t0
    hwm_seed = read_vmhwm(pid)
    log(f"INFO seeded 1000 links in {seed_secs:.3f}s; VmHWM={hwm_seed} KiB")
    summary["seed_seconds"] = seed_secs
    summary["vmhwm_after_seed_kib"] = hwm_seed
    check(seed_secs > 0, "seed completed", f"{seed_secs:.3f}s")

    # Warm-up: 50 jumps, not counted.
    warm = [jump_req(f"seed{(i % 1000) + 1:04d}") for i in range(50)]
    wc = BenchConn(port)
    try:
        for raw in warm:
            wc.send(raw)
            st, body = wc.read_resp()
            validate_jump(st, body)
    finally:
        wc.close()
    log("INFO jump warmup 50 complete")

    jump_rounds = []
    for rnd in range(3):
        tasks = []
        for w in range(50):
            reqs = [jump_req(f"seed{((w * 10 + j) % 1000) + 1:04d}") for j in range(10)]
            tasks.append((reqs, validate_jump))
        round_lats = []
        t0 = time.monotonic()
        per_worker = parallel_requests(port, tasks)
        elapsed = time.monotonic() - t0
        for wl in per_worker:
            round_lats.extend(wl)
        p = p95(round_lats)
        jump_rounds.append(p)
        log(f"INFO jump round {rnd + 1}: n={len(round_lats)} p95={p:.2f}ms elapsed={elapsed:.3f}s")
        for w, wl in enumerate(per_worker):
            for j, ms in enumerate(wl):
                csv_append(csv_path, f"jump,{rnd + 1},{w},{j},{ms:.4f}\n")
    jump_med = median(jump_rounds)
    hwm_after_jump = read_vmhwm(pid)
    log(f"INFO jump p95 rounds={jump_rounds} median={jump_med:.2f}ms VmHWM={hwm_after_jump} KiB")
    summary.update(jump_p95_rounds=jump_rounds, jump_p95_median=jump_med,
                   vmhwm_after_jump_kib=hwm_after_jump)

    # Creation warmup: 20, not counted.
    wc = BenchConn(port)
    try:
        for i in range(20):
            wc.send(encode_post(f"https://example.com/bench/warmup/{i}"))
            validate_create(*wc.read_resp())
    finally:
        wc.close()
    log("INFO create warmup 20 complete")

    create_rounds = []
    nonce = f"{os.getpid()}-{int(time.time() * 1000)}"
    for rnd in range(3):
        tasks = []
        for w in range(50):
            reqs = [encode_post(f"https://example.com/bench/create/{nonce}/{rnd}/{w}/{j}")
                    for j in range(2)]
            tasks.append((reqs, validate_create))
        round_lats = []
        t0 = time.monotonic()
        per_worker = parallel_requests(port, tasks)
        elapsed = time.monotonic() - t0
        for wl in per_worker:
            round_lats.extend(wl)
        p = p95(round_lats)
        create_rounds.append(p)
        log(f"INFO create round {rnd + 1}: n={len(round_lats)} p95={p:.2f}ms elapsed={elapsed:.3f}s")
        for w, wl in enumerate(per_worker):
            for j, ms in enumerate(wl):
                csv_append(csv_path, f"create,{rnd + 1},{w},{j},{ms:.4f}\n")
    create_med = median(create_rounds)
    hwm_final = read_vmhwm(pid)
    log(f"INFO create p95 rounds={create_rounds} median={create_med:.2f}ms "
        f"VmHWM={hwm_final} KiB")
    summary.update(create_p95_rounds=create_rounds, create_p95_median=create_med,
                   vmhwm_final_kib=hwm_final)

    check(jump_med <= 10.00, "jump p95 <= 10.00ms", f"{jump_med:.2f}ms rounds={jump_rounds}")
    check(create_med <= 15.00, "create p95 <= 15.00ms", f"{create_med:.2f}ms rounds={create_rounds}")
    check(hwm_final <= 10240, "VmHWM <= 10240 KiB", f"{hwm_final} KiB")
    summary["jump_pass"] = jump_med <= 10.00
    summary["create_pass"] = create_med <= 15.00
    summary["memory_pass"] = hwm_final <= 10240
    Path(logdir, "perf-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return summary


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--artifact", default=str(Path(__file__).resolve().parent.parent / "artifact"))
    ap.add_argument("--phase", choices=["functional", "perf", "all"], default="all")
    ap.add_argument("--logdir", default=str(Path(__file__).resolve().parent / "logs" / "run"))
    ap.add_argument("--keep-work", action="store_true")
    args = ap.parse_args()

    binary = Path(args.artifact) / "shortlink"
    logdir = Path(args.logdir)
    logdir.mkdir(parents=True, exist_ok=True)
    workroot = Path(__file__).resolve().parent / "work" / time.strftime("%Y%m%d-%H%M%S")
    workroot.mkdir(parents=True, exist_ok=True)
    log(f"INFO artifact={binary}")
    log(f"INFO logdir={logdir}")
    log(f"INFO workdir={workroot}")

    if not binary.exists():
        check(False, "artifact binary exists", str(binary))
        log("SUMMARY: cannot run, artifact missing")
        return 2

    functional_ok = True
    perf_ok = True
    try:
        if args.phase in ("functional", "all"):
            fdir = workroot / "functional"
            fdir.mkdir()
            srv = Server(binary, fdir / "nested" / "deep" / "links.db", fdir, logdir)
            try:
                srv.start()
                # db parent did not exist before startup
                check(Path(srv.db_path).exists(), "DB file auto-created with parent dirs", srv.db_path)
                functional_ok = run_functional(srv, logdir)
            finally:
                if srv.alive():
                    srv.stop()
            check_eq(srv.stop_seconds is not None and srv.stop_seconds < 5.0, True,
                     "final functional shutdown within 5s")
        if args.phase in ("perf", "all"):
            pdir = workroot / "perf"
            pdir.mkdir()
            srv = Server(binary, pdir / "perf.db", pdir, logdir)
            csv_path = logdir / "latencies.csv"
            csv_path.write_text("phase,round,worker,request,ms\n")
            try:
                srv.start()
                summary = run_perf(srv, logdir, csv_path)
            finally:
                if srv.alive():
                    srv.stop()
                csv_path.write_text(csv_path.read_text())
            check(srv.stop_seconds is not None and srv.stop_seconds < 5.0, True,
                  "perf shutdown within 5s")
            perf_ok = summary.get("jump_pass") and summary.get("create_pass") and summary.get("memory_pass")
    finally:
        if not args.keep_work:
            pass  # keep for evidence
    failed = [r for r in RESULTS if not r[0]]
    log(f"SUMMARY total: {len(RESULTS) - len(failed)} passed, {len(failed)} failed; "
        f"functional={'ok' if functional_ok else 'FAIL'} perf={'ok' if perf_ok else 'FAIL'}")
    log("STATUS: 待验收 (implementer self-check only; not acceptance)")
    return 0 if not failed else 1


if __name__ == "__main__":
    sys.exit(main())
