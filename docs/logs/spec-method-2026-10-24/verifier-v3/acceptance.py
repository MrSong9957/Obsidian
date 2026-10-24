#!/usr/bin/env python3
"""Independent acceptance script for the short-link service.

Only the frozen spec and run constraints are used as the source of truth.
The script launches the service itself (PORT=0, its own temporary DB_PATH),
runs every check, and writes raw evidence next to the report.

Usage:
    python3 acceptance.py --start-cmd '"/path/to/server"' \
        --start-cwd /path/to/artifact \
        --start-notes "see START.md: binary built with cargo build --release"

A missing check is a FAIL.  A failed check never stops the remaining checks
(unless the service cannot be started at all, in which case every remaining
check is explicitly marked NOT EXECUTED).
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import signal
import socket
import sys
import traceback
from datetime import datetime, timezone

from acc_core import (Recorder, Server, ephemeral_port_range, mem_total_kb,
                      port_open, service_snapshot, utc_now_iso, write_json)
from acc_functional import (phase_alias, phase_basic_create,
                            phase_click_concurrency, phase_create_variants,
                            phase_encoding, phase_http_keepalive,
                            phase_redirect_basic, phase_stats, phase_ttl)
from acc_perf import phase_perf
from acc_restart import phase_restart
from acc_validation import phase_validation

PLAN = [
    ("START-01", "PORT=0 startup line is exact and the port is in the kernel ephemeral range"),
    ("START-02", "announced port accepts TCP connections"),
    ("START-03", "missing DB_PATH parent directories are created"),
    ("START-04", "missing DB file is created"),
    ("START-05", "service runs as a single process"),
    ("START-06", "startup line is flushed and readable within the startup timeout"),
    ("START-07", "DB_PATH file is a SQLite 3 database after the first commit"),
    ("HTTP-01", "responses use HTTP/1.1"),
    ("HTTP-02", "HTTP/1.1 keep-alive carries two requests on one connection"),
    ("HTTP-03", "Content-Length is present and correct on every inspected response"),
    ("HTTP-04", "successful JSON responses use application/json; charset=utf-8"),
    ("CREATE-01", "POST /api/links returns 201 with exactly code/short_url/expires_at"),
    ("CREATE-02", "short_url is http://127.0.0.1:<actual port>/<code>"),
    ("CREATE-03", "auto-generated code is 7 characters A-Z a-z 0-9"),
    ("CREATE-04", "expires_at is null when no TTL is supplied"),
    ("CREATE-05", "a supplied alias is returned verbatim as code"),
    ("CREATE-06", "alias:null is treated as an omitted alias"),
    ("CREATE-07", "ttl_seconds:null means no expiry"),
    ("CREATE-08", "expires_at equals created_at + ttl_seconds"),
    ("CREATE-09", "ttl_seconds boundaries 1 and 31536000 are accepted"),
    ("CREATE-10", "unknown JSON fields are ignored"),
    ("CREATE-11", "creating the same URL twice yields two different codes"),
    ("CREATE-12", "20 auto-generated codes are unique and well formed"),
    ("CREATE-13", "a request body of exactly 1 MiB is accepted"),
    ("CREATE-14", "alias lengths 4 and 32 and _/- characters are accepted"),
    ("CREATE-15", "Content-Type media type is case-insensitive and may carry charset"),
    ("STATS-01", "GET /api/links/<code> returns 200 with exactly the six fields"),
    ("STATS-02", "stats returns the original URL"),
    ("STATS-03", "clicks starts at 0 and is a non-negative integer"),
    ("STATS-04", "created_at is UTC ISO 8601 to the second and plausible"),
    ("STATS-05", "expires_at null and expired false for a link without TTL"),
    ("STATS-06", "stats code matches the requested code"),
    ("STATS-07", "unknown codes return 404 not_found from stats"),
    ("REDIR-01", "GET /<code> returns 302, exact Location and Content-Length: 0"),
    ("REDIR-02", "one successful redirect increments clicks by exactly 1"),
    ("REDIR-03", "unknown codes return 404 not_found from redirect"),
    ("REDIR-04", "50 concurrent redirects increase clicks by exactly 50"),
    ("ENC-01", "existing %2F is preserved verbatim in Location"),
    ("ENC-02", "existing lowercase %2f is preserved verbatim in Location"),
    ("ENC-03", "existing %20 is preserved verbatim in Location"),
    ("ENC-04", "Chinese/space examples: 201 and exact 1.3 encoding (path/query/fragment spaces are legal)"),
    ("ENC-05", "Location comparison is against the raw header, not a decoded form"),
    ("ENC-06", "stats returns the original UTF-8 URL"),
    ("ENC-07", "a Chinese-only URL is UTF-8 percent-encoded exactly"),
    ("ALIAS-01", "20 concurrent creates for one alias yield exactly 1x201 and 19x409"),
    ("ALIAS-02", "aliases are case-sensitive (Abc1 and abc1 are distinct links)"),
    ("ALIAS-03", "a duplicate alias returns 409 alias_taken"),
    ("ALIAS-04", "concurrent alias losers return exact alias_taken bodies"),
    ("TTL-01", "before expiry: stats 200 expired=false, redirect 302, clicks 1"),
    ("TTL-02", "after expiry: redirect returns 410 expired"),
    ("TTL-03", "after expiry: stats 200, expired=true, clicks unchanged"),
    ("TTL-04", "further redirects after expiry do not increase clicks"),
    ("RESTART-01", "SIGTERM exits within 5 seconds and releases the listening port"),
    ("RESTART-02", "link, URL, clicks, created_at and expires_at survive restart"),
    ("RESTART-03", "clicks continue from the persisted value after restart"),
    ("RESTART-04", "a TTL link keeps its expires_at across restart"),
    ("RESTART-05", "a 201 response means committed: link survives SIGKILL"),
    ("RESTART-06", "counts and expiry stay correct after unclean shutdown"),
    ("VALID-01", "missing/wrong Content-Type returns 400 invalid_request"),
    ("VALID-02", "malformed JSON returns 400 invalid_request"),
    ("VALID-03", "valid JSON that is not an object returns 400 invalid_request"),
    ("VALID-04", "wrong url type or missing url is rejected with 400"),
    ("VALID-05", "non-http(s) schemes, missing host, and spaces in scheme/host return 400 invalid_url"),
    ("VALID-06", "URL length is limited to 2048 UTF-8 bytes"),
    ("VALID-07", "URLs containing ASCII control characters return 400 invalid_url"),
    ("VALID-08", "a '%' not followed by two hex digits returns 400 invalid_url"),
    ("VALID-09", "ttl_seconds rejects booleans, floats, strings and out-of-range integers"),
    ("VALID-10", "alias length/charset violations return 400 invalid_alias"),
    ("VALID-11", "reserved aliases api and health return 400 invalid_alias"),
    ("VALID-12", "a sequentially duplicated alias returns 409 alias_taken"),
    ("VALID-13", "every error response contains exactly one field: error"),
    ("VALID-14", "bodies larger than 1 MiB return 400 invalid_request"),
    ("VALID-15", "the service still answers after an oversized body"),
    ("PERF-01", "1000 seed links seed0001-seed1000 are created"),
    ("PERF-SEED-SPOT", "spot-checked seed links return the expected URL"),
    ("PERF-02", "jump load: 3 rounds x 500 requests, warmup 50, all 302"),
    ("PERF-03", "jump p95 median of 3 rounds <= 10.00 ms"),
    ("PERF-04", "create load: 3 rounds x 100 requests, warmup 20, all 201"),
    ("PERF-05", "create p95 median of 3 rounds <= 15.00 ms"),
    ("PERF-06", "service VmHWM after stress <= 10240 KiB"),
    ("PERF-07", "the service is still a single process after stress"),
]


def parse_args(argv):
    parser = argparse.ArgumentParser(
        description="Independent acceptance test for the short-link service.")
    parser.add_argument("--start-cmd", default=os.environ.get("ACCEPT_START_CMD"),
                        help="shell command that launches the service (PORT/DB_PATH are set by this script)")
    parser.add_argument("--start-cwd", default=os.environ.get("ACCEPT_START_CWD"),
                        help="working directory for the start command")
    parser.add_argument("--start-env", action="append", default=[],
                        metavar="KEY=VALUE", help="extra environment variable (repeatable)")
    parser.add_argument("--start-notes", default=os.environ.get("ACCEPT_START_NOTES"),
                        help="free-form startup instructions; '@file' reads a file")
    parser.add_argument("--startup-timeout", type=float, default=30.0)
    parser.add_argument("--work-dir", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "runs"))
    parser.add_argument("--run-id", default=None)
    parser.add_argument("--spec", default=os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                                       "..", "frozen", "spec.md"))
    parser.add_argument("--constraints", default=os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                                              "..", "frozen", "run-constraints.md"))
    args = parser.parse_args(argv)
    if not args.start_cmd:
        parser.error("--start-cmd (or ACCEPT_START_CMD) is required")
    return args


def sha256_file(path):
    try:
        with open(path, "rb") as f:
            return hashlib.sha256(f.read()).hexdigest()
    except OSError:
        return None


def load_notes(value):
    if value and value.startswith("@"):
        path = value[1:]
        try:
            with open(path, "r", encoding="utf-8") as f:
                return f.read(), path
        except (OSError, UnicodeError) as exc:
            return f"<could not read {path}: {exc}>", path
    return value or "", None


def write_report(rec, ctx, server, args, started_at):
    lines = []
    lines.append("=" * 78)
    lines.append("Independent acceptance run - spec-method-2026-10-24")
    lines.append("=" * 78)
    lines.append(f"started_utc:       {started_at}")
    lines.append(f"finished_utc:      {utc_now_iso()}")
    lines.append(f"start_cmd:         {args.start_cmd}")
    lines.append(f"start_cwd:         {args.start_cwd}")
    lines.append(f"startup_notes:     {ctx.get('start_notes', '')}")
    lines.append(f"db_path:           {ctx['db_path']}")
    lines.append(f"spec_sha256:       {ctx.get('spec_sha256')}")
    lines.append(f"constraints_sha256:{ctx.get('constraints_sha256')}")
    lines.append(f"script_sha256:     {ctx.get('script_sha256')}")
    lines.append(f"boots:             {json.dumps(server.boots, ensure_ascii=False)}")
    lines.append("-" * 78)
    for cid in rec.order:
        ok, detail = rec.results.get(cid, (False, "NOT EXECUTED"))
        line = f"{'PASS' if ok else 'FAIL'} [{cid}] {rec.plan[cid]}"
        if detail:
            line += f" :: {detail}"
        lines.append(line)
    lines.append("-" * 78)
    passed, failed = rec.counts()
    lines.append(f"checks: {passed} passed, {failed} failed, {len(rec.order)} total")
    lines.append(f"RESULT: {'PASS' if failed == 0 else 'FAIL'}")
    lines.append(f"evidence_dir: {ctx['run_dir']}")
    lines.append("=" * 78)
    text = "\n".join(lines) + "\n"
    with open(os.path.join(ctx["run_dir"], "report.txt"), "w", encoding="utf-8") as f:
        f.write(text)
    print("-" * 78)
    print(f"checks: {passed} passed, {failed} failed, {len(rec.order)} total")
    print(f"RESULT: {'PASS' if failed == 0 else 'FAIL'}")
    print(f"evidence_dir: {ctx['run_dir']}")
    return failed == 0


def main(argv=None):
    args = parse_args(argv)

    # Bad invocation must be reported as exit code 2, not as a service failure.
    if args.startup_timeout <= 0:
        print("--startup-timeout must be > 0", file=sys.stderr)
        return 2
    if args.start_cwd and not os.path.isdir(args.start_cwd):
        print(f"--start-cwd is not a directory: {args.start_cwd!r}", file=sys.stderr)
        return 2
    if os.path.exists(args.work_dir) and not os.path.isdir(args.work_dir):
        print(f"--work-dir is not a directory: {args.work_dir!r}", file=sys.stderr)
        return 2
    for label, path in (("--spec", args.spec), ("--constraints", args.constraints)):
        if not os.path.isfile(path):
            print(f"{label} does not exist: {path!r}", file=sys.stderr)
            return 2
    if args.start_notes and args.start_notes.startswith("@"):
        notes_path_arg = args.start_notes[1:]
        if not os.path.isfile(notes_path_arg):
            print(f"--start-notes file does not exist: {notes_path_arg!r}",
                  file=sys.stderr)
            return 2

    extra_env = {}
    for item in args.start_env:
        if "=" not in item:
            print(f"bad --start-env (expected KEY=VALUE): {item!r}", file=sys.stderr)
            return 2
        key, _, value = item.partition("=")
        if not key:
            print(f"bad --start-env (empty key): {item!r}", file=sys.stderr)
            return 2
        extra_env[key] = value

    started_at = utc_now_iso()
    run_id = args.run_id or (datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
                             + f"-{os.getpid()}")
    run_dir = os.path.abspath(os.path.join(args.work_dir, run_id))
    os.makedirs(run_dir, exist_ok=False)
    notes, notes_path = load_notes(args.start_notes)
    db_path = os.path.join(run_dir, "dbroot", "nested", "deeper", "links.db")
    rec = Recorder(PLAN, run_dir)

    ctx = {
        "run_dir": run_dir,
        "db_path": db_path,
        "port": None,
        "boot_no": 0,
        "start_notes": notes,
        "start_notes_path": notes_path,
        "spec_sha256": sha256_file(args.spec),
        "constraints_sha256": sha256_file(args.constraints),
        "script_sha256": sha256_file(os.path.abspath(__file__)),
        "component_sha256": {
            name: sha256_file(os.path.join(os.path.dirname(os.path.abspath(__file__)), name))
            for name in ("acc_core.py", "acc_functional.py", "acc_validation.py",
                         "acc_restart.py", "acc_perf.py")
        },
        "restart_done": False,
    }
    write_json(os.path.join(run_dir, "meta.json"), {
        "started_utc": started_at,
        "argv": sys.argv,
        "python": sys.version,
        "platform": platform.platform(),
        "hostname": socket.gethostname(),
        "cpu_count": os.cpu_count(),
        "mem_total_kb": mem_total_kb(),
        "cwd": os.getcwd(),
        "start_cmd": args.start_cmd,
        "start_cwd": args.start_cwd,
        "start_env": extra_env,
        "start_notes": notes,
        "start_notes_path": notes_path,
        "db_path": db_path,
        "spec_path": os.path.abspath(args.spec),
        "spec_sha256": ctx["spec_sha256"],
        "constraints_path": os.path.abspath(args.constraints),
        "constraints_sha256": ctx["constraints_sha256"],
        "script_sha256": ctx["script_sha256"],
        "component_sha256": ctx["component_sha256"],
        "plans": [{"id": cid, "description": desc} for cid, desc in PLAN],
    })

    server = Server(args.start_cmd, args.start_cwd, db_path,
                    os.path.join(run_dir, "server-logs"),
                    startup_timeout=args.startup_timeout,
                    extra_env=extra_env)
    fatal = None
    started_ok = False

    # ---------------- startup -------------------------------------------
    try:
        boot = server.start(1)
    except Exception as exc:
        fatal = f"startup failed: {exc}"
        rec.check("START-01", False, fatal)
        with open(os.path.join(run_dir, "errors.log"), "a", encoding="utf-8") as f:
            f.write(fatal + "\n" + traceback.format_exc() + "\n")
    else:
        ctx["port"] = boot["port"]
        ctx["boot_no"] = 1
        started_ok = True
        port_range = ephemeral_port_range()
        range_ok = (port_range is None
                    or port_range[0] <= boot["port"] <= port_range[1])
        rec.check("START-01", range_ok,
                  f"line={boot['startup_line']!r} port={boot['port']} "
                  f"kernel_range={port_range}")
        rec.check("START-02", port_open(boot["port"]),
                  f"port={boot['port']}")
        rec.check("START-06",
                  boot["startup_line_seconds"] <= args.startup_timeout,
                  f"read after {boot['startup_line_seconds']:.3f}s "
                  f"(timeout {args.startup_timeout:.1f}s)")
        try:
            snapshot = service_snapshot(boot["port"], server.proc.pid)
        except Exception as exc:
            rec.check("START-05", False,
                      f"process inspection failed: {exc!r}")
            with open(os.path.join(run_dir, "errors.log"), "a",
                      encoding="utf-8") as f:
                f.write("process inspection failed\n"
                        + traceback.format_exc() + "\n")
        else:
            rec.check("START-05", snapshot["process_count"] == 1,
                      f"process count={snapshot['process_count']} "
                      f"cmdlines={[p['cmdline'] for p in snapshot['processes']]}")

    # ---------------- functional ----------------------------------------
    if started_ok:
        phases = [
            ("basic_create", lambda: phase_basic_create(ctx, rec)),
            ("db-autocreate", None),  # handled inline below
            ("stats", lambda: phase_stats(ctx, rec)),
            ("redirect", lambda: phase_redirect_basic(ctx, rec)),
            ("http-keepalive", lambda: phase_http_keepalive(ctx, rec)),
            ("create-variants", lambda: phase_create_variants(ctx, rec)),
            ("encoding", lambda: phase_encoding(ctx, rec)),
            ("alias", lambda: phase_alias(ctx, rec)),
            ("click-concurrency", lambda: phase_click_concurrency(ctx, rec)),
            ("ttl", lambda: phase_ttl(ctx, rec)),
            ("validation", lambda: phase_validation(ctx, rec)),
            ("restart", lambda: phase_restart(ctx, rec, server)),
            ("perf", lambda: phase_perf(ctx, rec, server)),
        ]
        for name, action in phases:
            if action is None:
                parent = os.path.dirname(db_path)
                rec.check("START-03", os.path.isdir(parent),
                          f"parent dir={parent} exists={os.path.isdir(parent)}")
                rec.check("START-04", os.path.isfile(db_path),
                          f"db file={db_path} exists={os.path.isfile(db_path)}")
                header_ok = False
                header = b""
                try:
                    with open(db_path, "rb") as dbfile:
                        header = dbfile.read(16)
                    header_ok = header == b"SQLite format 3\x00"
                except OSError as exc:
                    header = f"<{exc}>".encode()
                rec.check("START-07", header_ok, f"header={header!r}")
                continue
            try:
                action()
            except Exception as exc:
                fatal = f"phase {name} raised: {exc}"
                print(f"FAIL [phase {name}] {fatal}", flush=True)
                with open(os.path.join(run_dir, "errors.log"), "a", encoding="utf-8") as f:
                    f.write(f"phase {name} raised\n" + traceback.format_exc() + "\n")
                if name == "restart":
                    # Do not run perf against an unknown DB/port state.
                    break

        # These global checks are evaluated after all functional + perf traffic.
        rec.check("HTTP-03", rec.responses_seen > 0 and not rec.protocol_issues,
                  "issues: " + " | ".join(rec.protocol_issues[:5])
                  if rec.protocol_issues else
                  f"all {rec.responses_seen} inspected responses had correct Content-Length")
        rec.check("HTTP-04", rec.success_responses_seen > 0 and not rec.ct_issues,
                  "issues: " + " | ".join(rec.ct_issues[:5])
                  if rec.ct_issues else
                  f"all {rec.success_responses_seen} 2xx responses had the required Content-Type")
        # Error bodies from every phase (including restart) are covered here.
        rec.check("VALID-13", rec.error_bodies_seen > 0 and not rec.error_body_issues,
                  f"malformed error bodies: {rec.error_body_issues[:5]}"
                  if rec.error_body_issues else
                  f"all {rec.error_bodies_seen} inspected error bodies had exactly one error field")

    # ---------------- shutdown ------------------------------------------
    proc_alive = server.proc is not None and server.proc.poll() is None
    port_alive = bool(server.port) and port_open(server.port)
    if proc_alive or port_alive:
        try:
            stop = server.stop(signal.SIGTERM, timeout=15.0)
            print(f"cleanup: SIGTERM stop elapsed={stop.elapsed_s:.3f}s "
                  f"timed_out={stop.timed_out} rc={stop.returncode}", flush=True)
        except Exception as exc:
            with open(os.path.join(run_dir, "errors.log"), "a", encoding="utf-8") as f:
                f.write(f"cleanup stop raised: {exc}\n" + traceback.format_exc() + "\n")

    rec.finalize()
    ok = write_report(rec, ctx, server, args, started_at)
    rec.close()
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
