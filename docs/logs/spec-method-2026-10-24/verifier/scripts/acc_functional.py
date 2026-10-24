"""Functional acceptance checks: create, stats, redirect, encoding, aliases,
click concurrency and TTL expiry."""
from __future__ import annotations

import json
import socket
import threading
import time
import urllib.parse

from acc_core import (HOST, Group, get_redirect, get_stats, iso_z,
                      parse_iso_z, post_json, short_json, truncate)

PRIMARY_URL = "https://example.com/alpha?x=1"


# --------------------------------------------------------------------------
# helpers
# --------------------------------------------------------------------------
def run_threads(count: int, worker) -> list:
    barrier = threading.Barrier(count)
    results: list = [None] * count
    threads = []

    def wrapped(index: int):
        try:
            barrier.wait(timeout=30)
            results[index] = worker(index)
        except BaseException as exc:  # recorded, never silently dropped
            results[index] = exc

    for i in range(count):
        thread = threading.Thread(target=wrapped, args=(i,), daemon=True)
        threads.append(thread)
        thread.start()
    for thread in threads:
        thread.join(timeout=120)
    for i, thread in enumerate(threads):
        if thread.is_alive() and results[i] is None:
            results[i] = TimeoutError(f"worker {i} did not finish")
    return results


def json_keys(obj) -> str:
    if isinstance(obj, dict):
        return ",".join(sorted(obj.keys()))
    return f"<{type(obj).__name__}>"


def body_with_size(url: str, target: int) -> bytes:
    padding = 0
    body = b""
    for _ in range(12):
        body = json.dumps({"url": url, "pad": "x" * padding},
                          ensure_ascii=False).encode("utf-8")
        if len(body) == target:
            break
        padding += target - len(body)
        if padding < 0:
            padding = 0
            break
    return body


# --------------------------------------------------------------------------
# creation: success shape and short_url
# --------------------------------------------------------------------------
def phase_basic_create(ctx, rec):
    port = ctx["port"]
    resp = post_json(rec, port, {"url": PRIMARY_URL}, label="create-primary")
    obj = resp.json_body()
    group = Group(rec, "CREATE-01")
    group.check(resp.status == 201, f"status={resp.status} body={truncate(resp.body, 300)!r}")
    group.check(isinstance(obj, dict), f"body is not an object: {truncate(resp.body, 300)!r}")
    if isinstance(obj, dict):
        group.check(set(obj) == {"code", "short_url", "expires_at"},
                    f"keys={json_keys(obj)}")
        group.check(isinstance(obj.get("code"), str), "code is not a string")
        group.check(isinstance(obj.get("short_url"), str), "short_url is not a string")
        group.check("expires_at" in obj and (obj.get("expires_at") is None
                                             or isinstance(obj.get("expires_at"), str)),
                    "expires_at is not null/string")
    group.finish()

    code = obj.get("code") if isinstance(obj, dict) else None
    if not isinstance(code, str):
        code = ""
    ctx.update({
        "primary": code,
        "primary_url": PRIMARY_URL,
        "primary_created": obj if isinstance(obj, dict) else {},
        "primary_clicks": 0,
    })

    group = Group(rec, "CREATE-02")
    expected_short = f"http://127.0.0.1:{port}/{code}"
    group.check(isinstance(obj, dict) and obj.get("short_url") == expected_short,
                f"expected {expected_short!r}, got {obj.get('short_url') if isinstance(obj, dict) else None!r}")
    group.finish()

    group = Group(rec, "CREATE-03")
    group.check(bool(code) and len(code) == 7 and code.isalnum() and code.isascii(),
                f"code={code!r}")
    group.finish()

    group = Group(rec, "CREATE-04")
    group.check(isinstance(obj, dict) and obj.get("expires_at") is None,
                f"expires_at={obj.get('expires_at') if isinstance(obj, dict) else None!r}")
    group.finish()


def phase_create_variants(ctx, rec):
    port = ctx["port"]
    created = ctx["primary_created"]

    # alias is echoed exactly (case preserved)
    alias = "varAlias"
    r = post_json(rec, port, {"url": "https://example.com/alias-echo", "alias": alias},
                  label="create-alias-echo")
    obj = r.json_body()
    group = Group(rec, "CREATE-05")
    group.check(r.status == 201, f"status={r.status} body={truncate(r.body, 200)!r}")
    group.check(isinstance(obj, dict) and obj.get("code") == alias,
                f"code={obj.get('code') if isinstance(obj, dict) else None!r}")
    group.check(isinstance(obj, dict) and obj.get("short_url") == f"http://127.0.0.1:{port}/{alias}",
                "short_url does not use alias")
    group.finish()

    # alias null behaves as absent: auto code
    r = post_json(rec, port, {"url": "https://example.com/alias-null", "alias": None},
                  label="create-alias-null")
    obj = r.json_body()
    group = Group(rec, "CREATE-06")
    group.check(r.status == 201, f"status={r.status} body={truncate(r.body, 200)!r}")
    code = obj.get("code") if isinstance(obj, dict) else None
    group.check(isinstance(code, str) and len(code) == 7 and code.isalnum() and code.isascii(),
                f"code={code!r}")
    group.finish()

    # ttl null behaves as absent
    r = post_json(rec, port, {"url": "https://example.com/ttl-null", "ttl_seconds": None},
                  label="create-ttl-null")
    obj = r.json_body()
    group = Group(rec, "CREATE-07")
    group.check(r.status == 201, f"status={r.status} body={truncate(r.body, 200)!r}")
    group.check(isinstance(obj, dict) and obj.get("expires_at") is None,
                f"expires_at={obj.get('expires_at') if isinstance(obj, dict) else None!r}")
    group.finish()

    # expires_at == created_at + ttl_seconds
    ttl = 3600
    r = post_json(rec, port, {"url": "https://example.com/ttl-delta", "ttl_seconds": ttl},
                  label="create-ttl-delta")
    obj = r.json_body()
    code = obj.get("code") if isinstance(obj, dict) else None
    group = Group(rec, "CREATE-08")
    group.check(r.status == 201, f"status={r.status}")
    exp_epoch = parse_iso_z(obj.get("expires_at") if isinstance(obj, dict) else None)
    group.check(exp_epoch is not None, f"expires_at={obj.get('expires_at') if isinstance(obj, dict) else None!r}")
    if code and exp_epoch is not None:
        stats = get_stats(rec, port, code, label="stats-ttl-delta")
        st = stats.json_body()
        created_epoch = parse_iso_z(st.get("created_at") if isinstance(st, dict) else None)
        group.check(created_epoch is not None, "created_at missing/bad")
        if created_epoch is not None:
            group.check(exp_epoch == created_epoch + ttl,
                        f"expires_at-created_at={exp_epoch - created_epoch}, expected {ttl}")
    group.finish()

    # ttl boundaries 1 and 31536000 accepted
    group = Group(rec, "CREATE-09")
    for ttl_value in (1, 31536000):
        r = post_json(rec, port, {"url": f"https://example.com/ttl-{ttl_value}",
                                 "ttl_seconds": ttl_value},
                      label=f"create-ttl-{ttl_value}")
        group.check(r.status == 201, f"ttl={ttl_value}: status={r.status} body={truncate(r.body, 120)!r}")
        obj = r.json_body()
        exp = parse_iso_z(obj.get("expires_at") if isinstance(obj, dict) else None)
        group.check(exp is not None, f"ttl={ttl_value}: expires_at invalid")
        if exp is not None:
            # Do not require a strict "future" timestamp: with ttl=1 the
            # second boundary may pass while the response is being read.
            group.check(exp >= time.time() - 2,
                        f"ttl={ttl_value}: expires_at is implausibly old")
    group.finish()

    # unknown fields are ignored
    r = post_json(rec, port, {"url": "https://example.com/extra", "unknown": {"a": [1, 2]},
                             "another": "x"}, label="create-extra-fields")
    group = Group(rec, "CREATE-10")
    group.check(r.status == 201, f"status={r.status} body={truncate(r.body, 200)!r}")
    group.finish()

    # same URL twice -> different codes
    url = "https://example.com/twice"
    r1 = post_json(rec, port, {"url": url}, label="create-twice-1")
    r2 = post_json(rec, port, {"url": url}, label="create-twice-2")
    o1 = r1.json_body()
    o2 = r2.json_body()
    c1 = o1.get("code") if isinstance(o1, dict) else None
    c2 = o2.get("code") if isinstance(o2, dict) else None
    group = Group(rec, "CREATE-11")
    group.check(r1.status == 201 and r2.status == 201, f"statuses={r1.status},{r2.status}")
    group.check(isinstance(c1, str) and isinstance(c2, str) and c1 != c2,
                f"codes={c1!r},{c2!r}")
    group.finish()

    # 20 auto codes are unique and well formed
    codes = []
    group = Group(rec, "CREATE-12")
    for i in range(20):
        r = post_json(rec, port, {"url": f"https://example.com/unique/{i}"},
                      label=f"create-unique-{i}")
        obj = r.json_body()
        code = obj.get("code") if isinstance(obj, dict) else None
        if r.status != 201:
            group.check(False, f"case {i}: status={r.status}")
            continue
        group.check(isinstance(code, str) and len(code) == 7 and code.isalnum()
                    and code.isascii(), f"case {i}: code={code!r}")
        codes.append(code)
    group.check(len(set(codes)) == len(codes), f"duplicate codes among {len(codes)}: {short_json(codes)}")
    group.finish()

    # exactly 1 MiB body accepted (padding field ignored)
    body = body_with_size("https://example.com/big-body", 1 << 20)
    r = post_json(rec, port, body, label="create-body-1mib")
    group = Group(rec, "CREATE-13")
    group.check(len(body) == 1 << 20, f"constructed body length={len(body)}")
    group.check(r.status == 201, f"status={r.status} body={truncate(r.body, 200)!r}")
    # the service must still be alive after the large body
    try:
        alive = get_stats(rec, port, ctx["primary"], label="alive-after-1mib")
        group.check(alive.status == 200,
                    f"service after 1 MiB body: status={alive.status}")
    except Exception as exc:
        group.check(False, f"service after 1 MiB body raised: {exc!r}")
    group.finish()

    # alias length/charset boundaries accepted
    group = Group(rec, "CREATE-14")
    for candidate in ("abcd", "ab_cd", "ab-cd", "a" * 32):
        r = post_json(rec, port, {"url": f"https://example.com/alias/{candidate}",
                                 "alias": candidate}, label=f"create-alias-{candidate[:6]}")
        obj = r.json_body()
        group.check(r.status == 201, f"alias len={len(candidate)}: status={r.status}")
        group.check(isinstance(obj, dict) and obj.get("code") == candidate,
                    f"alias len={len(candidate)}: code mismatch")
    group.finish()

    # media type is case-insensitive and may carry ; charset=utf-8
    group = Group(rec, "CREATE-15")
    for ctype in ("application/json; charset=utf-8", "Application/JSON",
                  "Application/JSON; charset=utf-8"):
        r = post_json(rec, port, {"url": f"https://example.com/ct/{len(ctype)}"},
                      content_type=ctype, label=f"create-ct-{len(ctype)}")
        group.check(r.status == 201, f"{ctype!r}: status={r.status} body={truncate(r.body, 120)!r}")
    group.finish()


# --------------------------------------------------------------------------
# stats on a fresh link
# --------------------------------------------------------------------------
def phase_stats(ctx, rec):
    port = ctx["port"]
    code = ctx["primary"]
    resp = get_stats(rec, port, code, label="stats-primary")
    obj = resp.json_body()
    group = Group(rec, "STATS-01")
    group.check(resp.status == 200, f"status={resp.status} body={truncate(resp.body, 200)!r}")
    group.check(isinstance(obj, dict), "body is not an object")
    if isinstance(obj, dict):
        group.check(set(obj) == {"code", "url", "clicks", "created_at", "expires_at", "expired"},
                    f"keys={json_keys(obj)}")
        group.check(isinstance(obj.get("code"), str), "code type")
        group.check(isinstance(obj.get("url"), str), "url type")
        group.check(isinstance(obj.get("clicks"), int) and not isinstance(obj.get("clicks"), bool),
                    f"clicks type={type(obj.get('clicks')).__name__}")
        group.check(isinstance(obj.get("created_at"), str), "created_at type")
        group.check(obj.get("expires_at") is None or isinstance(obj.get("expires_at"), str),
                    "expires_at type")
        group.check(isinstance(obj.get("expired"), bool), "expired type")
    group.finish()

    group = Group(rec, "STATS-02")
    group.check(isinstance(obj, dict) and obj.get("url") == PRIMARY_URL,
                f"url={obj.get('url') if isinstance(obj, dict) else None!r}")
    group.finish()

    group = Group(rec, "STATS-03")
    group.check(isinstance(obj, dict) and obj.get("clicks") == 0,
                f"clicks={obj.get('clicks') if isinstance(obj, dict) else None!r}")
    group.finish()

    group = Group(rec, "STATS-04")
    created = parse_iso_z(obj.get("created_at") if isinstance(obj, dict) else None)
    group.check(created is not None,
                f"created_at={obj.get('created_at') if isinstance(obj, dict) else None!r}")
    if created is not None:
        skew = time.time() - created
        group.check(-2 <= skew <= 300, f"created_at skew={skew:.1f}s")
    group.finish()

    group = Group(rec, "STATS-05")
    group.check(isinstance(obj, dict) and obj.get("expires_at") is None
                and obj.get("expired") is False,
                f"expires_at={obj.get('expires_at') if isinstance(obj, dict) else None!r} "
                f"expired={obj.get('expired') if isinstance(obj, dict) else None!r}")
    group.finish()

    group = Group(rec, "STATS-06")
    group.check(isinstance(obj, dict) and obj.get("code") == code,
                f"code={obj.get('code') if isinstance(obj, dict) else None!r}")
    group.finish()


# --------------------------------------------------------------------------
# redirect and exact click counting
# --------------------------------------------------------------------------
def phase_redirect_basic(ctx, rec):
    port = ctx["port"]
    code = ctx["primary"]
    resp = get_redirect(rec, port, code, label="redirect-primary")
    group = Group(rec, "REDIR-01")
    group.check(resp.status == 302, f"status={resp.status} body={truncate(resp.body, 200)!r}")
    group.check(resp.get("Location") == PRIMARY_URL,
                f"Location={resp.get('Location')!r}")
    group.check(resp.get("Content-Length") == "0",
                f"Content-Length={resp.get('Content-Length')!r}")
    group.check(resp.body == b"", f"body length={len(resp.body)}")
    group.finish()

    stats = get_stats(rec, port, code, label="stats-after-one-redirect")
    obj = stats.json_body()
    group = Group(rec, "REDIR-02")
    group.check(stats.status == 200, f"stats status={stats.status}")
    group.check(isinstance(obj, dict) and obj.get("clicks") == 1,
                f"clicks={obj.get('clicks') if isinstance(obj, dict) else None!r}")
    group.finish()
    ctx["primary_clicks"] = 1


def phase_http_keepalive(ctx, rec):
    port = ctx["port"]
    code = ctx["primary"]
    sock = socket.create_connection((HOST, port), timeout=15)
    reader = sock.makefile("rb")
    results = []

    def one_request():
        request = (f"GET /{code} HTTP/1.1\r\nHost: {HOST}:{port}\r\n"
                   f"Connection: keep-alive\r\n\r\n").encode("ascii")
        sock.sendall(request)
        status_line = reader.readline().decode("latin-1").rstrip("\r\n")
        headers = {}
        while True:
            line = reader.readline()
            if line in (b"\r\n", b"\n", b""):
                break
            name, _, value = line.decode("latin-1").partition(":")
            headers[name.strip().lower()] = value.strip()
        length = int(headers.get("content-length", "0"))
        body = reader.read(length)
        return status_line, headers, body

    try:
        first = one_request()
        second = one_request()
        error = None
    except Exception as exc:  # keep-alive failure is a check failure, not a crash
        first = second = None
        error = repr(exc)
    finally:
        try:
            reader.close()
        finally:
            sock.close()

    group = Group(rec, "HTTP-01")
    group.check(error is None and first is not None
                and first[0].startswith("HTTP/1.1 "),
                f"first status line={(first[0] if first else error)!r}")
    group.finish()

    group = Group(rec, "HTTP-02")
    group.check(error is None and first is not None and second is not None,
                f"second request on same connection failed: {error}")
    if error is None and first and second:
        group.check(first[0].startswith("HTTP/1.1 302"),
                    f"first status={first[0]!r}")
        group.check(second[0].startswith("HTTP/1.1 302"),
                    f"second status={second[0]!r}")
        group.check(first[1].get("location") == PRIMARY_URL,
                    f"first Location={first[1].get('location')!r}")
        group.check(second[1].get("location") == PRIMARY_URL,
                    f"second Location={second[1].get('location')!r}")
        group.check(first[1].get("content-length") == "0"
                    and second[1].get("content-length") == "0",
                    "Content-Length is not 0 on redirect")
    group.finish()
    rec.trace({
        "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "label": "keep-alive-raw",
        "first_status_line": first[0] if first else None,
        "second_status_line": second[0] if second else None,
        "error": error,
    })


# --------------------------------------------------------------------------
# Location encoding
# --------------------------------------------------------------------------
def _create_and_redirect(ctx, rec, label, url):
    port = ctx["port"]
    create = post_json(rec, port, {"url": url}, label=f"create-{label}")
    obj = create.json_body()
    code = obj.get("code") if isinstance(obj, dict) else None
    if create.status != 201 or not isinstance(code, str):
        return create, None, None
    redirect = get_redirect(rec, port, code, label=f"redirect-{label}")
    return create, redirect, code


def phase_encoding(ctx, rec):
    port = ctx["port"]

    cases = [
        ("ENC-01", "pct2F", "https://example.com/a%2Fb?x=%20&y=1",
         "https://example.com/a%2Fb?x=%20&y=1"),
        ("ENC-02", "pct2f", "https://example.com/x%2fy?z=%2f",
         "https://example.com/x%2fy?z=%2f"),
        ("ENC-03", "pct20", "https://example.com/a%20b", "https://example.com/a%20b"),
    ]
    created = {}
    for cid, label, url, expected in cases:
        create, redirect, code = _create_and_redirect(ctx, rec, label, url)
        group = Group(rec, cid)
        group.check(create.status == 201, f"create status={create.status} body={truncate(create.body, 120)!r}")
        group.check(redirect is not None and redirect.status == 302,
                    f"redirect status={redirect.status if redirect else 'none'}")
        if redirect is not None:
            group.check(redirect.get("Location") == expected,
                        f"raw Location={redirect.get('Location')!r} expected={expected!r}")
        group.finish()
        if isinstance(code, str):
            created[label] = code

    # Spec 1.1 permits ordinary spaces in path/query/fragment and requires
    # them to be percent-encoded per 1.3.  Both the spec example and a plain
    # space URL must therefore be accepted with an exact raw Location.
    space_cases = [
        ("utf8-space-spec", "https://example.com/路径?x=中 文#片段",
         "https://example.com/%E8%B7%AF%E5%BE%84"
         "?x=%E4%B8%AD%20%E6%96%87#%E7%89%87%E6%AE%B5"),
        ("utf8-space-plain", "https://example.com/a b?c=d e#f g",
         "https://example.com/a%20b?c=d%20e#f%20g"),
    ]
    group = Group(rec, "ENC-04")
    for label, space_url, space_expected in space_cases:
        create, redirect, _ = _create_and_redirect(
            ctx, rec, label, space_url)
        group.check(create.status == 201,
                    f"{label}: create status={create.status} "
                    f"body={truncate(create.body, 120)!r}")
        group.check(redirect is not None and redirect.status == 302,
                    f"{label}: redirect status="
                    f"{redirect.status if redirect else 'none'}")
        if redirect is not None:
            group.check(redirect.get("Location") == space_expected,
                        f"{label}: raw Location={redirect.get('Location')!r} "
                        f"expected={space_expected!r}")
    group.finish("path/query/fragment spaces accepted and encoded per 1.3")

    group = Group(rec, "ENC-05")
    loc = None
    code = created.get("pct2F")
    if code:
        resp = get_redirect(rec, port, code, label="redirect-pct2F-raw")
        loc = resp.get("Location")
    group.check(isinstance(loc, str) and "%2F" in loc,
                f"Location={loc!r} does not contain literal %2F")
    if isinstance(loc, str):
        group.check(urllib.parse.unquote(loc) != loc,
                    "Location had already been percent-decoded before comparison")
        group.check("/a/b" not in loc,
                    f"Location={loc!r} turned %2F into /")
    group.finish()

    # Pure Chinese URL: no whitespace, so this part is unambiguous.
    chinese_url = "https://example.com/路径?x=中文#片段"
    chinese_expected = ("https://example.com/%E8%B7%AF%E5%BE%84"
                        "?x=%E4%B8%AD%E6%96%87#%E7%89%87%E6%AE%B5")
    create, redirect, code = _create_and_redirect(ctx, rec, "utf8-only", chinese_url)
    group = Group(rec, "ENC-07")
    group.check(create.status == 201, f"create status={create.status} body={truncate(create.body, 120)!r}")
    group.check(redirect is not None and redirect.status == 302,
                f"redirect status={redirect.status if redirect else 'none'}")
    if redirect is not None:
        group.check(redirect.get("Location") == chinese_expected,
                    f"raw Location={redirect.get('Location')!r} expected={chinese_expected!r}")
    group.finish()

    group = Group(rec, "ENC-06")
    if isinstance(code, str):
        stats = get_stats(rec, port, code, label="stats-utf8")
        obj = stats.json_body()
        group.check(stats.status == 200, f"status={stats.status}")
        group.check(isinstance(obj, dict) and obj.get("url") == chinese_url,
                    f"url={obj.get('url') if isinstance(obj, dict) else None!r}")
    else:
        group.check(False, "utf8 link was not created")
    group.finish()



# --------------------------------------------------------------------------
# alias concurrency and case sensitivity
# --------------------------------------------------------------------------
def phase_alias(ctx, rec):
    port = ctx["port"]
    alias = "conc-alias"
    url = "https://example.com/concurrent-alias"

    def create_same(index):
        return post_json(rec, port, {"url": url, "alias": alias},
                         label=f"alias-concurrent-{index}")

    results = run_threads(20, create_same)
    statuses = []
    successes = []
    losers = []
    exceptions = []
    for item in results:
        if isinstance(item, BaseException):
            exceptions.append(item)
            continue
        statuses.append(item.status)
        if item.status == 201:
            successes.append(item)
        else:
            losers.append(item)

    group = Group(rec, "ALIAS-01")
    group.check(statuses.count(201) == 1,
                f"201 count={statuses.count(201)} statuses={sorted(statuses)}")
    group.check(statuses.count(409) == 19,
                f"409 count={statuses.count(409)} statuses={sorted(statuses)}")
    group.check(len(statuses) == 20,
                f"responses={len(statuses)} exceptions={[repr(e) for e in exceptions[:3]]}")
    group.finish()

    group = Group(rec, "ALIAS-04")
    if successes:
        obj = successes[0].json_body()
        group.check(isinstance(obj, dict) and obj.get("code") == alias,
                    f"winner code={obj.get('code') if isinstance(obj, dict) else None!r}")
        group.check(isinstance(obj, dict) and set(obj) == {"code", "short_url", "expires_at"},
                    f"winner keys={json_keys(obj) if isinstance(obj, dict) else obj}")
    else:
        group.check(False, "no 201 response")
    for exc in exceptions:
        group.check(False, f"exception in concurrent create: {exc!r}")
    for item in losers:
        if item.status != 409:
            group.check(False, f"loser status={item.status} body={truncate(item.body, 120)!r}")
            continue
        obj = item.json_body()
        group.check(isinstance(obj, dict) and set(obj) == {"error"}
                    and obj.get("error") == "alias_taken",
                    f"loser body={truncate(item.body, 120)!r}")
    group.finish()

    group = Group(rec, "ALIAS-03")
    stats = get_stats(rec, port, alias, label="stats-concurrent-alias")
    obj = stats.json_body()
    group.check(stats.status == 200 and isinstance(obj, dict)
                and obj.get("code") == alias and obj.get("url") == url,
                f"stats status={stats.status} body={truncate(stats.body, 200)!r}")
    duplicate = post_json(rec, port, {"url": "https://example.com/duplicate-alias",
                                     "alias": alias}, label="alias-duplicate-after")
    obj = duplicate.json_body()
    group.check(duplicate.status == 409 and isinstance(obj, dict)
                and set(obj) == {"error"} and obj.get("error") == "alias_taken",
                f"duplicate status={duplicate.status} body={truncate(duplicate.body, 120)!r}")
    group.finish()

    group = Group(rec, "ALIAS-02")
    upper_alias, lower_alias = "Abc1", "abc1"
    upper_url = "https://example.com/upper"
    lower_url = "https://example.com/lower"
    r1 = post_json(rec, port, {"url": upper_url, "alias": upper_alias}, label="alias-upper")
    r2 = post_json(rec, port, {"url": lower_url, "alias": lower_alias}, label="alias-lower")
    o1, o2 = r1.json_body(), r2.json_body()
    group.check(r1.status == 201 and r2.status == 201,
                f"statuses={r1.status},{r2.status}")
    group.check(isinstance(o1, dict) and o1.get("code") == upper_alias,
                f"upper code={o1.get('code') if isinstance(o1, dict) else None!r}")
    group.check(isinstance(o2, dict) and o2.get("code") == lower_alias,
                f"lower code={o2.get('code') if isinstance(o2, dict) else None!r}")
    s1 = get_stats(rec, port, upper_alias, label="stats-upper")
    s2 = get_stats(rec, port, lower_alias, label="stats-lower")
    b1, b2 = s1.json_body(), s2.json_body()
    group.check(isinstance(b1, dict) and b1.get("url") == upper_url,
                "upper alias resolves to wrong URL")
    group.check(isinstance(b2, dict) and b2.get("url") == lower_url,
                "lower alias resolves to wrong URL")
    group.finish()


def phase_click_concurrency(ctx, rec):
    port = ctx["port"]
    alias = "click50"
    url = "https://example.com/clicks-50"
    r = post_json(rec, port, {"url": url, "alias": alias}, label="create-click50")
    obj = r.json_body()
    baseline = 0
    group = Group(rec, "REDIR-04")
    group.check(r.status == 201, f"create status={r.status}")
    if r.status == 201:
        before = get_stats(rec, port, alias, label="stats-click50-before")
        before_obj = before.json_body()
        baseline = before_obj.get("clicks") if isinstance(before_obj, dict) else None
        group.check(baseline == 0, f"baseline clicks={baseline!r}")

        def jump(index):
            return get_redirect(rec, port, alias, label=f"click50-jump-{index}")

        results = run_threads(50, jump)
        statuses = []
        bad_locations = 0
        exceptions = 0
        for item in results:
            if isinstance(item, BaseException):
                exceptions += 1
                continue
            statuses.append(item.status)
            if item.get("Location") != url:
                bad_locations += 1
        group.check(exceptions == 0, f"exceptions={exceptions}")
        group.check(statuses.count(302) == 50,
                    f"302 count={statuses.count(302)} statuses={sorted(statuses)}")
        group.check(bad_locations == 0, f"wrong Location count={bad_locations}")

        after = get_stats(rec, port, alias, label="stats-click50-after")
        after_obj = after.json_body()
        clicks = after_obj.get("clicks") if isinstance(after_obj, dict) else None
        group.check(clicks == 50, f"clicks after 50 concurrent jumps={clicks!r}")
        extra = get_redirect(rec, port, alias, label="click50-extra")
        third = get_stats(rec, port, alias, label="stats-click50-extra")
        third_obj = third.json_body()
        clicks2 = third_obj.get("clicks") if isinstance(third_obj, dict) else None
        group.check(extra.status == 302 and clicks2 == 51,
                    f"after one more jump: status={extra.status} clicks={clicks2!r}")
    group.finish()


# --------------------------------------------------------------------------
# TTL expiry (short TTL, wall-clock wait, no database edits)
# --------------------------------------------------------------------------
def phase_ttl(ctx, rec):
    port = ctx["port"]
    code = None
    expires = None
    attempts = 0
    for attempt in range(5):
        attempts = attempt + 1
        url = f"https://example.com/ttl-expiry/{attempt}"
        r = post_json(rec, port, {"url": url, "ttl_seconds": 1}, label=f"create-ttl-expiry-{attempt}")
        obj = r.json_body()
        candidate = obj.get("code") if isinstance(obj, dict) else None
        exp = parse_iso_z(obj.get("expires_at") if isinstance(obj, dict) else None)
        if r.status == 201 and isinstance(candidate, str) and exp is not None \
                and exp - time.time() >= 0.4:
            code, expires, ttl_url = candidate, exp, url
            break
    if code is None:
        for cid in ("TTL-01", "TTL-02", "TTL-03", "TTL-04"):
            rec.check(cid, False, "could not obtain a TTL=1 link with enough time left")
        return

    started = time.time()
    before = get_stats(rec, port, code, label="ttl-stats-before")
    before_obj = before.json_body()
    jump1 = get_redirect(rec, port, code, label="ttl-jump-before")
    after_jump = get_stats(rec, port, code, label="ttl-stats-after-jump")
    after_obj = after_jump.json_body()

    group = Group(rec, "TTL-01")
    group.check(before.status == 200, f"stats status={before.status}")
    group.check(isinstance(before_obj, dict) and before_obj.get("expired") is False,
                f"expired={before_obj.get('expired') if isinstance(before_obj, dict) else None!r}")
    group.check(isinstance(before_obj, dict) and before_obj.get("clicks") == 0,
                f"clicks before={before_obj.get('clicks') if isinstance(before_obj, dict) else None!r}")
    group.check(jump1.status == 302,
                f"pre-expiry jump status={jump1.status} (attempt {attempts}, "
                f"remaining={expires - time.time():.3f}s)")
    group.check(isinstance(after_obj, dict) and after_obj.get("clicks") == 1,
                f"clicks after one jump={after_obj.get('clicks') if isinstance(after_obj, dict) else None!r}")
    group.check(isinstance(after_obj, dict) and after_obj.get("expires_at") == iso_z(expires),
                f"expires_at={after_obj.get('expires_at') if isinstance(after_obj, dict) else None!r}")
    group.finish()

    # Wait until the first part of the expiry second.  The server floors time
    # to Unix seconds, so its "now" is normally exactly expires_at here; this
    # tests the spec's "expires_at <= now" rule rather than a plain "< now".
    # No database modification is used.
    deadline = expires + 0.35
    while time.time() < deadline:
        time.sleep(0.02)
    waited = time.time() - started

    expired_resp = get_redirect(rec, port, code, label="ttl-jump-after")
    obj = expired_resp.json_body()
    group = Group(rec, "TTL-02")
    group.check(expired_resp.status == 410,
                f"status={expired_resp.status} body={truncate(expired_resp.body, 200)!r}")
    group.check(isinstance(obj, dict) and set(obj) == {"error"} and obj.get("error") == "expired",
                f"body={truncate(expired_resp.body, 200)!r}")
    group.finish()

    stats_expired = get_stats(rec, port, code, label="ttl-stats-expired")
    expired_obj = stats_expired.json_body()
    group = Group(rec, "TTL-03")
    group.check(stats_expired.status == 200, f"status={stats_expired.status}")
    group.check(isinstance(expired_obj, dict) and expired_obj.get("expired") is True,
                f"expired={expired_obj.get('expired') if isinstance(expired_obj, dict) else None!r}")
    group.check(isinstance(expired_obj, dict) and expired_obj.get("clicks") == 1,
                f"clicks={expired_obj.get('clicks') if isinstance(expired_obj, dict) else None!r}")
    group.check(isinstance(expired_obj, dict) and expired_obj.get("expires_at") == iso_z(expires),
                f"expires_at={expired_obj.get('expires_at') if isinstance(expired_obj, dict) else None!r}")
    group.finish()

    second = get_redirect(rec, port, code, label="ttl-jump-after-2")
    stats_final = get_stats(rec, port, code, label="ttl-stats-final")
    final_obj = stats_final.json_body()
    group = Group(rec, "TTL-04")
    group.check(second.status == 410, f"second expired jump status={second.status}")
    group.check(isinstance(final_obj, dict) and final_obj.get("clicks") == 1,
                f"clicks after second expired jump={final_obj.get('clicks') if isinstance(final_obj, dict) else None!r}")
    group.finish()
    rec.trace({"ts": iso_z(time.time()), "label": "ttl-timing",
               "attempt": attempts, "waited_seconds": round(waited, 3),
               "expires_at": iso_z(expires)})
