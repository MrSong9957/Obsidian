"""Shared infrastructure for the independent acceptance script.

Only Python 3.12 standard library is used.  Nothing in this module knows how
the service under test is implemented; it only launches the command supplied
on the command line and talks HTTP to the announced port.
"""
from __future__ import annotations

import http.client
import json
import math
import os
import queue
import re
import signal
import socket
import subprocess
import threading
import time
from datetime import datetime, timezone

HOST = "127.0.0.1"
START_RE = re.compile(r"listening on http://127\.0\.0\.1:(\d+)")
ISO_RE = re.compile(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")
MISSING = object()


# --------------------------------------------------------------------------
# small helpers
# --------------------------------------------------------------------------
def utc_now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def iso_z(epoch: float) -> str:
    return datetime.fromtimestamp(int(epoch), timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def parse_iso_z(value) -> float | None:
    if not isinstance(value, str) or not ISO_RE.fullmatch(value):
        return None
    try:
        dt = datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    except ValueError:
        return None
    return dt.timestamp()


def nearest_rank_p95(samples: list[float]) -> float | None:
    if not samples:
        return None
    ordered = sorted(samples)
    idx = math.ceil(0.95 * len(ordered)) - 1
    return ordered[idx]


def median3(values: list[float]) -> float:
    return sorted(values)[1]


def ct_ok(value) -> bool:
    """Success responses must be application/json with charset=utf-8.

    Whitespace around ';' and '=' and the case of the charset token are
    normalised; the required media type and charset are not relaxed.
    """
    if not isinstance(value, str):
        return False
    norm = re.sub(r"\s*;\s*", ";", value.strip().lower())
    norm = re.sub(r"\s*=\s*", "=", norm)
    return norm == "application/json;charset=utf-8"


def truncate(value, limit: int = 2000) -> str:
    text = value.decode("utf-8", "replace") if isinstance(value, bytes) else str(value)
    if len(text) > limit:
        return text[:limit] + f"...<truncated {len(text) - limit} chars>"
    return text


def short_json(value, limit: int = 400) -> str:
    try:
        text = json.dumps(value, ensure_ascii=False, sort_keys=True)
    except Exception:
        text = repr(value)
    return text if len(text) <= limit else text[:limit] + "..."


# --------------------------------------------------------------------------
# HTTP response wrapper
# --------------------------------------------------------------------------
class Resp:
    def __init__(self, status: int, version: int, headers, body: bytes, elapsed_ms: float):
        self.status = status
        self.version = version
        self.headers = list(headers)
        self.body = body
        self.elapsed_ms = elapsed_ms

    def get_all(self, name: str) -> list[str]:
        low = name.lower()
        return [v for k, v in self.headers if k.lower() == low]

    def get(self, name: str):
        values = self.get_all(name)
        return values[0] if values else None

    def json_body(self):
        try:
            return json.loads(self.body.decode("utf-8"))
        except Exception:
            return MISSING

    def text(self) -> str:
        return self.body.decode("utf-8", "replace")

    def summary(self) -> str:
        return f"status={self.status} body={truncate(self.body, 300)!r}"


def http_request(rec, port, method, path, body=None, headers=None,
                 timeout=15.0, label="", check_error_body=True) -> Resp:
    started = time.perf_counter()
    conn = http.client.HTTPConnection(HOST, port, timeout=timeout)
    try:
        conn.request(method, path, body=body, headers=headers or {})
        raw = conn.getresponse()
        data = raw.read()
        hdrs = raw.getheaders()
        status = raw.status
        version = raw.version
    finally:
        conn.close()
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    resp = Resp(status, version, hdrs, data, elapsed_ms)
    tag = label or f"{method} {path}"
    rec.note_protocol(tag, resp)
    if 200 <= status < 300:
        rec.note_success_content_type(tag, resp)
    if status >= 400 and check_error_body:
        rec.note_error_body(tag, resp)
    rec.trace({
        "ts": utc_now_iso(),
        "label": tag,
        "method": method,
        "path": path,
        "request_headers": headers or {},
        "request_body_len": len(body) if isinstance(body, (bytes, bytearray)) else 0,
        "status": status,
        "version": f"HTTP/{version // 10}.{version % 10}",
        "response_headers": [[k, v] for k, v in resp.headers],
        "response_body": truncate(resp.body),
        "elapsed_ms": round(elapsed_ms, 3),
    })
    return resp


def post_json(rec, port, payload, content_type="application/json", label="",
              check_error_body=True):
    body = payload if isinstance(payload, (bytes, bytearray)) else json.dumps(
        payload, ensure_ascii=False).encode("utf-8")
    headers = {} if content_type is None else {"Content-Type": content_type}
    return http_request(rec, port, "POST", "/api/links", body=body,
                        headers=headers, label=label,
                        check_error_body=check_error_body)


def get_stats(rec, port, code, label=""):
    return http_request(rec, port, "GET", f"/api/links/{code}", label=label)


def get_redirect(rec, port, code, label=""):
    return http_request(rec, port, "GET", f"/{code}", label=label)


# --------------------------------------------------------------------------
# recorder
# --------------------------------------------------------------------------
class Recorder:
    def __init__(self, plan, evidence_dir: str):
        self.plan = dict(plan)
        self.order = [cid for cid, _ in plan]
        self.results: dict[str, tuple[bool, str]] = {}
        self.lock = threading.RLock()
        self.protocol_issues: list[str] = []
        self.ct_issues: list[str] = []
        self.error_body_issues: list[str] = []
        self.responses_seen = 0
        self.success_responses_seen = 0
        self.error_bodies_seen = 0
        self.evidence_dir = evidence_dir
        os.makedirs(evidence_dir, exist_ok=True)
        self._checks = open(os.path.join(evidence_dir, "checks.jsonl"),
                            "w", encoding="utf-8")
        self._trace = open(os.path.join(evidence_dir, "http-trace.jsonl"),
                           "w", encoding="utf-8")

    # -- recording ---------------------------------------------------------
    def check(self, cid: str, ok: bool, detail: str = "") -> None:
        if cid not in self.plan:
            raise AssertionError(f"check id not in plan: {cid}")
        detail = truncate(detail, 1200)
        with self.lock:
            self.results[cid] = (bool(ok), detail)
            row = {"ts": utc_now_iso(), "id": cid, "ok": bool(ok),
                   "description": self.plan[cid], "detail": detail}
            self._checks.write(json.dumps(row, ensure_ascii=False) + "\n")
            self._checks.flush()
            line = f"{'PASS' if ok else 'FAIL'} [{cid}] {self.plan[cid]}"
            if detail:
                line += f" :: {detail}"
            print(line, flush=True)

    def trace(self, row: dict) -> None:
        with self.lock:
            self._trace.write(json.dumps(row, ensure_ascii=False) + "\n")
            self._trace.flush()

    # -- protocol bookkeeping ---------------------------------------------
    def _append(self, bucket: list[str], message: str) -> None:
        with self.lock:
            if len(bucket) < 40:
                bucket.append(message)

    def note_protocol(self, label: str, resp: Resp) -> None:
        with self.lock:
            self.responses_seen += 1
        if resp.version != 11:
            self._append(self.protocol_issues,
                         f"{label}: response version HTTP/{resp.version // 10}.{resp.version % 10}")
        te = resp.get("Transfer-Encoding")
        if te:
            self._append(self.protocol_issues, f"{label}: Transfer-Encoding={te!r}")
        cl = resp.get("Content-Length")
        if cl is None:
            self._append(self.protocol_issues, f"{label}: missing Content-Length")
        else:
            try:
                declared = int(cl)
            except (TypeError, ValueError):
                self._append(self.protocol_issues, f"{label}: bad Content-Length={cl!r}")
            else:
                if declared != len(resp.body):
                    self._append(self.protocol_issues,
                                 f"{label}: Content-Length={declared} actual={len(resp.body)}")

    def note_success_content_type(self, label: str, resp: Resp) -> None:
        with self.lock:
            self.success_responses_seen += 1
        value = resp.get("Content-Type")
        if not ct_ok(value):
            self._append(self.ct_issues, f"{label}: Content-Type={value!r}")

    def note_error_body(self, label: str, resp: Resp) -> None:
        with self.lock:
            self.error_bodies_seen += 1
        obj = resp.json_body()
        if not isinstance(obj, dict) or set(obj) != {"error"}:
            self._append(self.error_body_issues,
                         f"{label}: status={resp.status} body={truncate(resp.body, 200)!r}")

    # -- completion --------------------------------------------------------
    def finalize(self) -> None:
        for cid in self.order:
            if cid not in self.results:
                self.check(cid, False,
                           "NOT EXECUTED (an earlier failure or exception prevented this check)")

    def counts(self):
        passed = sum(1 for ok, _ in self.results.values() if ok)
        failed = sum(1 for ok, _ in self.results.values() if not ok)
        return passed, failed

    def close(self) -> None:
        with self.lock:
            self._checks.close()
            self._trace.close()


# --------------------------------------------------------------------------
# server process management
# --------------------------------------------------------------------------
class StartupError(RuntimeError):
    pass


class StopResult:
    def __init__(self, elapsed_s: float, returncode, timed_out: bool):
        self.elapsed_s = elapsed_s
        self.returncode = returncode
        self.timed_out = timed_out


def port_open(port, timeout=0.15) -> bool:
    if not port:
        return False
    try:
        with socket.create_connection((HOST, port), timeout=timeout):
            return True
    except OSError:
        return False


def wait_port(port, timeout=10.0) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if port_open(port):
            return True
        time.sleep(0.02)
    return False


def ephemeral_port_range():
    """Kernel range used by bind(0); None if /proc is unavailable."""
    try:
        with open("/proc/sys/net/ipv4/ip_local_port_range", "r",
                  encoding="ascii") as f:
            low, high = f.read().split()
        return int(low), int(high)
    except (OSError, ValueError):
        return None


def mem_total_kb():
    """MemTotal from /proc/meminfo, for the evidence record."""
    try:
        with open("/proc/meminfo", "r", encoding="ascii") as f:
            for line in f:
                if line.startswith("MemTotal:"):
                    return int(line.split()[1])
    except (OSError, ValueError, IndexError):
        return None
    return None


class Server:
    def __init__(self, start_cmd: str, cwd: str | None, db_path: str,
                 logs_dir: str, startup_timeout: float = 30.0,
                 extra_env: dict | None = None):
        self.start_cmd = start_cmd
        self.cwd = cwd
        self.db_path = db_path
        self.logs_dir = logs_dir
        self.startup_timeout = startup_timeout
        self.extra_env = dict(extra_env or {})
        os.makedirs(logs_dir, exist_ok=True)
        self.proc: subprocess.Popen | None = None
        self.pgid: int | None = None
        self.port: int | None = None
        self.boots: list[dict] = []

    def _pump(self, stream, path: str, sink: queue.Queue | None) -> None:
        with open(path, "ab", buffering=0) as out:
            while True:
                line = stream.readline()
                if not line:
                    break
                out.write(line)
                if sink is not None:
                    sink.put(line)

    def _tail(self, path: str, limit: int = 1500) -> str:
        try:
            with open(path, "rb") as f:
                data = f.read()
            return data[-limit:].decode("utf-8", "replace")
        except OSError:
            return ""

    def start(self, boot_no: int) -> dict:
        env = os.environ.copy()
        env.update(self.extra_env)
        env["PORT"] = "0"
        env["DB_PATH"] = self.db_path
        out_log = os.path.join(self.logs_dir, f"boot-{boot_no:02d}.out")
        err_log = os.path.join(self.logs_dir, f"boot-{boot_no:02d}.err")
        for path in (out_log, err_log):
            open(path, "wb").close()
        q: queue.Queue = queue.Queue()
        started = time.monotonic()
        self.proc = subprocess.Popen(
            self.start_cmd,
            shell=True,
            cwd=self.cwd,
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        try:
            self.pgid = os.getpgid(self.proc.pid)
        except OSError:
            self.pgid = self.proc.pid
        threading.Thread(target=self._pump,
                         args=(self.proc.stdout, out_log, q),
                         daemon=True, name="stdout-pump").start()
        threading.Thread(target=self._pump,
                         args=(self.proc.stderr, err_log, None),
                         daemon=True, name="stderr-pump").start()

        deadline = started + self.startup_timeout
        line_text = None
        port = None
        seen: list[str] = []
        while time.monotonic() < deadline:
            if self.proc.poll() is not None and q.empty():
                break
            try:
                raw = q.get(timeout=0.05)
            except queue.Empty:
                continue
            text = raw.decode("utf-8", "replace").rstrip("\r\n")
            seen.append(text)
            match = START_RE.fullmatch(text)
            if match:
                line_text = text
                port = int(match.group(1))
                break
        if line_text is None:
            self._force_kill()
            raise StartupError(
                f"no matching startup line within {self.startup_timeout}s; "
                f"exit={self.proc.poll()}; stdout_tail={seen[-5:]!r}; "
                f"stderr_tail={self._tail(err_log)!r}")
        if port <= 0:
            raise StartupError(f"startup line announced non-positive port: {line_text!r}")
        elapsed = time.monotonic() - started
        self.port = port
        if not wait_port(port, timeout=10.0):
            self._force_kill()
            raise StartupError(f"startup line announced port {port} but it did not accept connections")
        boot = {
            "boot": boot_no,
            "pid": self.proc.pid,
            "port": port,
            "startup_line": line_text,
            "startup_line_seconds": round(elapsed, 4),
            "stdout_log": out_log,
            "stderr_log": err_log,
            "command": self.start_cmd,
        }
        self.boots.append(boot)
        return boot

    def _force_kill(self) -> None:
        if not self.proc:
            return
        try:
            os.killpg(self.pgid, signal.SIGKILL)
        except (ProcessLookupError, PermissionError, OSError):
            pass
        try:
            self.proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            pass

    def stop(self, sig=signal.SIGTERM, timeout: float = 10.0) -> StopResult:
        if not self.proc:
            raise RuntimeError("stop() called before start()")
        started = time.monotonic()
        pids_before = listener_pids(self.port) if self.port else set()
        try:
            os.killpg(self.pgid, sig)
        except (ProcessLookupError, PermissionError, OSError):
            pass
        for pid in pids_before:
            try:
                os.kill(pid, sig)
            except (ProcessLookupError, PermissionError):
                pass
        timed_out = False
        while time.monotonic() - started < timeout:
            if self.proc.poll() is not None and not port_open(self.port):
                break
            time.sleep(0.02)
        else:
            timed_out = True
            try:
                os.killpg(self.pgid, signal.SIGKILL)
            except (ProcessLookupError, PermissionError, OSError):
                pass
            for pid in listener_pids(self.port):
                try:
                    os.kill(pid, signal.SIGKILL)
                except (ProcessLookupError, PermissionError):
                    pass
        try:
            returncode = self.proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            returncode = None
        return StopResult(time.monotonic() - started, returncode, timed_out)


# --------------------------------------------------------------------------
# /proc inspection (service process / memory)
# --------------------------------------------------------------------------
def read_procs() -> dict[int, dict]:
    procs: dict[int, dict] = {}
    try:
        entries = os.listdir("/proc")
    except OSError:
        return procs
    for entry in entries:
        if not entry.isdigit():
            continue
        pid = int(entry)
        try:
            with open(f"/proc/{pid}/stat", "rb") as f:
                data = f.read().decode("latin-1")
        except OSError:
            continue
        rp = data.rfind(")")
        if rp < 0:
            continue
        fields = data[rp + 2:].split()
        if len(fields) < 2:
            continue
        try:
            procs[pid] = {"state": fields[0], "ppid": int(fields[1])}
        except ValueError:
            continue
    return procs


def proc_cmdline(pid: int) -> str:
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as f:
            return f.read().replace(b"\x00", b" ").decode("utf-8", "replace").strip()
    except OSError:
        return ""


def listener_inodes(port: int) -> set[str]:
    inodes: set[str] = set()
    for table in ("/proc/net/tcp", "/proc/net/tcp6"):
        try:
            with open(table, "r", encoding="latin-1") as f:
                lines = f.read().splitlines()[1:]
        except OSError:
            continue
        for line in lines:
            parts = line.split()
            if len(parts) < 10:
                continue
            local, state, inode = parts[1], parts[3], parts[9]
            try:
                local_port = int(local.rsplit(":", 1)[1], 16)
            except (ValueError, IndexError):
                continue
            if local_port == port and state.upper() == "0A":
                inodes.add(inode)
    return inodes


def listener_pids(port: int) -> set[int]:
    if not port:
        return set()
    inodes = listener_inodes(port)
    if not inodes:
        return set()
    needles = {f"socket:[{inode}]" for inode in inodes}
    pids: set[int] = set()
    try:
        entries = os.listdir("/proc")
    except OSError:
        return pids
    for entry in entries:
        if not entry.isdigit():
            continue
        pid = int(entry)
        fd_dir = f"/proc/{pid}/fd"
        try:
            fds = os.listdir(fd_dir)
        except OSError:
            continue
        for fd in fds:
            try:
                target = os.readlink(os.path.join(fd_dir, fd))
            except OSError:
                continue
            if target in needles:
                pids.add(pid)
                break
    return pids


def service_snapshot(port: int, fallback_pid: int | None = None) -> dict:
    procs = read_procs()
    roots = listener_pids(port)
    if not roots and fallback_pid and fallback_pid in procs:
        roots = {fallback_pid}
    children: dict[int, list[int]] = {}
    for pid, info in procs.items():
        children.setdefault(info["ppid"], []).append(pid)
    tree: set[int] = set(roots)
    stack = list(roots)
    while stack:
        pid = stack.pop()
        for child in children.get(pid, []):
            if child not in tree:
                tree.add(child)
                stack.append(child)
    alive = {pid for pid in tree if procs.get(pid, {}).get("state") != "Z"}
    pids = sorted(alive)
    memory_kb = 0
    per_process = []
    for pid in pids:
        hwm = read_vmhwm(pid)
        rss = read_vmrss(pid)
        memory_kb += hwm or 0
        per_process.append({
            "pid": pid,
            "cmdline": proc_cmdline(pid),
            "vmhwm_kb": hwm,
            "vmrss_kb": rss,
        })
    return {
        "port": port,
        "roots": sorted(roots),
        "tree_pids": pids,
        "process_count": len(pids),
        "memory_kb": memory_kb,
        "processes": per_process,
    }


def read_field_from_status(pid: int, field: str) -> int | None:
    try:
        with open(f"/proc/{pid}/status", "r", encoding="latin-1") as f:
            for line in f:
                if line.startswith(field + ":"):
                    return int(line.split()[1])
    except (OSError, ValueError, IndexError):
        return None
    return None


def read_vmhwm(pid: int) -> int | None:
    return read_field_from_status(pid, "VmHWM")


def read_vmrss(pid: int) -> int | None:
    return read_field_from_status(pid, "VmRSS")


# --------------------------------------------------------------------------
# grouped assertions
# --------------------------------------------------------------------------
class Group:
    """Collect several sub-cases into one planned PASS/FAIL line."""

    def __init__(self, rec: Recorder, cid: str):
        self.rec = rec
        self.cid = cid
        self.count = 0
        self.failures: list[str] = []

    def check(self, ok: bool, message: str = "") -> bool:
        self.count += 1
        if not ok:
            self.failures.append(message or "failed")
        return bool(ok)

    def finish(self, extra: str = "") -> bool:
        ok = not self.failures
        if ok:
            detail = f"{self.count} cases ok"
        else:
            detail = f"{len(self.failures)}/{self.count} cases failed: " + \
                     " | ".join(self.failures[:8])
        if extra:
            detail += f"; {extra}"
        self.rec.check(self.cid, ok, detail)
        return ok


def expect_error(rec: Recorder, group: Group, resp: Resp, status: int,
                 error: str, label: str, allowed_errors: set[str] | None = None) -> bool:
    errors = allowed_errors or {error}
    obj = resp.json_body()
    got = obj.get("error") if isinstance(obj, dict) else truncate(resp.body, 120)
    shape_ok = (isinstance(obj, dict) and set(obj) == {"error"}
                and isinstance(got, str) and got in errors)
    ok = resp.status == status and shape_ok
    if not ok:
        group.check(False, f"{label}: expected {status}/{'|'.join(sorted(errors))}, "
                            f"got {resp.status}/{got!r}")
    else:
        group.check(True)
    return ok


def write_json(path: str, value) -> None:
    with open(path, "w", encoding="utf-8") as f:
        json.dump(value, f, ensure_ascii=False, indent=2)
        f.write("\n")
