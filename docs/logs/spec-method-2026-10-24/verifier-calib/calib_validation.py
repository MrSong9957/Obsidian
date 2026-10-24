#!/usr/bin/env python3
"""Validation of the calibration measurement code (CALIBRATION-PLAN section 3).

Must run (and pass) BEFORE any matrix measurement. Three groups:
  1. nearest-rank p95 + 2-decimal rounding on known arrays,
     using the FROZEN acc_core functions (read-only import).
  2. sleep-5ms echo server: alternative-client timing primitive, single
     connection, 50 serial requests, p50 must lie in [5.0, 7.0] ms.
  3. every-3rd-request 500+disconnect server: error accounting must be exact
     (30 requests, 10 errors, 20 latency samples, errors not sampled).

Real outputs are written to verifier-calib/validation/.
"""
from __future__ import annotations

import http.server
import json
import math
import os
import sys
import threading
import time

HERE = os.path.dirname(os.path.abspath(__file__))
VERIFIER_DIR = os.path.abspath(os.path.join(HERE, "..", "verifier"))
sys.dont_write_bytecode = True          # never write into verifier/
sys.path.insert(0, VERIFIER_DIR)
from acc_core import median3, nearest_rank_p95   # noqa: E402  (frozen code path)

sys.path.insert(0, HERE)
import alt_client  # noqa: E402

OUT = os.path.join(HERE, "validation")
os.makedirs(OUT, exist_ok=True)

FAILURES: list[str] = []


def check(ok: bool, detail: str, bucket: list):
    status = "PASS" if ok else "FAIL"
    line = f"{status} {detail}"
    print(line, flush=True)
    bucket.append({"ok": bool(ok), "detail": detail})
    if not ok:
        FAILURES.append(detail)


def serve_in_thread(handler_cls):
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler_cls)
    port = srv.server_address[1]
    thread = threading.Thread(target=srv.serve_forever, daemon=True)
    thread.start()
    return srv, port


# ---------------------------------------------------------------------------
# group 1: formula + rounding on the frozen code path
# ---------------------------------------------------------------------------
def validation_1() -> list:
    rows: list = []
    print("--- validation 1: nearest-rank p95 + rounding (frozen acc_core) ---",
          flush=True)

    cases = [
        (list(range(1, 21)), 19.0),          # n=20  -> idx ceil(19)-1 = 18
        ([float(i) for i in range(1, 101)], 95.0),   # n=100 -> idx 94
        ([float(i) for i in range(1, 501)], 475.0),  # n=500 -> idx 474
        (list(range(20, 0, -1)), 19.0),      # unsorted input
    ]
    for samples, expected in cases:
        got = nearest_rank_p95(samples)
        idx = math.ceil(0.95 * len(samples)) - 1
        check(got == expected,
              f"p95 n={len(samples)} -> {got} (expected {expected}, "
              f"idx={idx}, sorted[idx]={sorted(samples)[idx]})", rows)

    got = nearest_rank_p95([])
    check(got is None, "p95 of empty list -> None", rows)

    rounding_cases = [
        (3.14159, 3.14), (2.71828, 2.72), (9.994, 9.99), (9.996, 10.0),
        (0.125, 0.12),   # exactly representable half: Python round-half-to-even,
                         # identical round() semantics as the frozen script
        (474.123, 474.12),
    ]
    for value, expected in rounding_cases:
        got = round(value, 2)
        check(got == expected, f"round({value}, 2) -> {got} (expected {expected})",
              rows)

    pipeline = [i + 0.123 for i in range(500)]
    p95 = round(nearest_rank_p95(pipeline), 2)
    check(p95 == 474.12, f"pipeline round(p95(x,500),2) -> {p95} (expected 474.12)",
          rows)

    med_cases = [([1.0, 2.0, 3.0], 2.0), ([3.0, 1.0, 2.0], 2.0),
                 ([13.98, 15.36, 15.50], 15.36)]
    for values, expected in med_cases:
        got = median3(values)
        check(got == expected, f"median3({values}) -> {got} (expected {expected})",
              rows)
    return rows


# ---------------------------------------------------------------------------
# group 2: sleep-5ms echo, alternative-client timing primitive
# ---------------------------------------------------------------------------
class Sleep5Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_GET(self):
        time.sleep(0.005)
        body = b""
        self.send_response(302)
        self.send_header("Location", "https://example.com/seed/x")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()

    def log_message(self, *args):
        pass


def validation_2() -> list:
    rows: list = []
    print("--- validation 2: sleep-5ms echo, single connection, 50 serial ---",
          flush=True)
    srv, port = serve_in_thread(Sleep5Handler)
    try:
        specs = [("GET", f"/seed{i}", None, None) for i in range(50)]
        samples, errors, count = alt_client.serial_requests(port, specs, 302)
        n = len(samples)
        p50 = sorted(samples)[math.ceil(0.50 * n) - 1] if n else None
        mean = sum(samples) / n if n else None
        check(count == 50 and not errors and n == 50,
              f"count={count} errors={len(errors)} samples={n}", rows)
        check(p50 is not None and 5.0 <= p50 <= 7.0,
              f"p50={p50}ms (required 5.0-7.0ms); mean={mean}", rows)
        check(min(samples) >= 5.0,
              f"min={min(samples)}ms (every request must include the 5ms sleep)",
              rows)
        return rows
    finally:
        srv.shutdown()


# ---------------------------------------------------------------------------
# group 3: every-3rd-request 500 + disconnect, error accounting
# ---------------------------------------------------------------------------
class Every3rdHandler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    _per_conn_count = 0

    def do_GET(self):
        Every3rdHandler._per_conn_count += 1
        if Every3rdHandler._per_conn_count % 3 == 0:
            body = b'{"error":"internal_error"}'
            self.send_response(500)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Connection", "close")
            self.end_headers()
            self.wfile.write(body)
            self.close_connection = True
            # reset for the next connection the client opens
            Every3rdHandler._per_conn_count = 0
        else:
            body = b"ok"
            self.send_response(200)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    def log_message(self, *args):
        pass


def validation_3() -> list:
    rows: list = []
    print("--- validation 3: every-3rd 500+disconnect, 30 requests ---", flush=True)
    srv, port = serve_in_thread(Every3rdHandler)
    try:
        specs = [("GET", f"/v3/{i}", None, None) for i in range(30)]
        samples, errors, count = alt_client.serial_requests(port, specs, 200)
        err_idx = sorted(int(e.split(":", 1)[0]) for e in errors
                         if e.split(":", 1)[0].isdigit())
        expected_idx = list(range(2, 30, 3))
        check(count == 30, f"total requests={count} (expected 30)", rows)
        check(len(errors) == 10,
              f"errors={len(errors)} (expected 10): {errors[:3]}...", rows)
        check(len(samples) == 20,
              f"latency samples={len(samples)} (expected 20; errors not sampled)",
              rows)
        check(err_idx == expected_idx,
              f"error positions={err_idx} (expected {expected_idx})", rows)
        check(all("status=500" in e for e in errors),
              "all errors are status=500 (no spurious exceptions)", rows)
        check(all(v > 0 for v in samples), "all samples positive", rows)
        return rows
    finally:
        srv.shutdown()


def main() -> int:
    result = {
        "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "group1": validation_1(),
        "group2": validation_2(),
        "group3": validation_3(),
        "loadavg_before": open("/proc/loadavg").read().strip(),
        "loadavg_after": None,
    }
    result["loadavg_after"] = open("/proc/loadavg").read().strip()
    result["all_passed"] = not FAILURES
    with open(os.path.join(OUT, "validation.json"), "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=1)
        f.write("\n")
    verdict = "PASSED" if not FAILURES else "FAILED"
    print(f"VALIDATION {verdict}: {len(FAILURES)} failure(s)", flush=True)
    return 0 if not FAILURES else 1


if __name__ == "__main__":
    sys.exit(main())
