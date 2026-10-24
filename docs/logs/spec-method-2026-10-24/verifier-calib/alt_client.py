#!/usr/bin/env python3
"""Alternative load client: 50 independent single-threaded processes.

Load shape and timing scope match the frozen acc_perf.py rounds exactly:
  - per round: 50 worker processes (multiprocessing, fork context), each with
    one persistent http.client connection; connect() happens BEFORE the
    barrier, so TCP connect is outside the measured window
  - timer: time.perf_counter() around conn.request() -> getresponse() -> read()
    (request written ... response fully read), same as the frozen script
  - warmup and the serial session helper use one single connection (A20 shape)

Differences vs the frozen thread client (deliberate, CALIBRATION-PLAN 0.2):
  - I1: workers are processes, not threads (no client-side GIL contention)
  - I6: erroneous requests are NOT added to the latency samples (a sample is
    recorded only for a fully successful, protocol-clean response)
"""
from __future__ import annotations

import http.client
import json
import multiprocessing
import time

HOST = "127.0.0.1"
CLIENTS = 50
JUMP_REQUESTS_PER_CLIENT = 10   # -> 500 per round (frozen)
CREATE_REQUESTS_PER_CLIENT = 2  # -> 100 per round (frozen)


# ---------------------------------------------------------------------------
# timed request primitive (used by serial sessions and worker bodies alike)
# ---------------------------------------------------------------------------
def request_once(conn, method, path, body=None, headers=None) -> dict:
    started = time.perf_counter()
    conn.request(method, path, body=body, headers=headers or {})
    resp = conn.getresponse()
    data = resp.read()
    elapsed = (time.perf_counter() - started) * 1000.0
    return {
        "status": resp.status,
        "version": resp.version,
        "content_length": resp.getheader("Content-Length"),
        "location": resp.getheader("Location"),
        "body_len": len(data),
        "elapsed_ms": elapsed,
    }


def protocol_issues(r: dict) -> list[str]:
    issues = []
    if r["version"] != 11:
        issues.append(f"version={r['version']}")
    if r["content_length"] != str(r["body_len"]):
        issues.append(f"Content-Length={r['content_length']!r} actual={r['body_len']}")
    return issues


# ---------------------------------------------------------------------------
# serial single-connection session (warmup shape, A20; validation driver)
# ---------------------------------------------------------------------------
def serial_requests(port, specs, expected_status):
    """Run specs [(method, path, body, headers), ...] on one persistent
    connection; after any error the connection is recreated. Returns
    (samples_ms, errors, request_count). Erroneous requests are not sampled.
    """
    samples: list[float] = []
    errors: list[str] = []
    count = 0
    conn = http.client.HTTPConnection(HOST, port, timeout=20)
    try:
        conn.connect()
        for idx, (method, path, body, headers) in enumerate(specs):
            count += 1
            try:
                r = request_once(conn, method, path, body, headers)
            except BaseException as exc:
                errors.append(f"{idx}: exception={exc!r}")
                try:
                    conn.close()
                except Exception:
                    pass
                conn = http.client.HTTPConnection(HOST, port, timeout=20)
                conn.connect()
                continue
            issues = protocol_issues(r)
            ok = r["status"] == expected_status and not issues
            if ok:
                samples.append(r["elapsed_ms"])
            else:
                msg = f"{idx}: status={r['status']}"
                if issues:
                    msg += f" proto={'|'.join(issues)}"
                errors.append(msg)
                try:
                    conn.close()
                except Exception:
                    pass
                conn = http.client.HTTPConnection(HOST, port, timeout=20)
                conn.connect()
    finally:
        try:
            conn.close()
        except Exception:
            pass
    return samples, errors, count


def jump_warmup(port, codes, count=50):
    specs = [("GET", "/" + codes[i % len(codes)], None, None) for i in range(count)]
    return serial_requests(port, specs, expected_status=302)


def create_warmup(port, count=20):
    specs = []
    for i in range(count):
        body = json.dumps({"url": f"https://example.com/perf/warmup/{i}"}).encode()
        specs.append(("POST", "/api/links", body,
                      {"Content-Type": "application/json"}))
    return serial_requests(port, specs, expected_status=201)


# ---------------------------------------------------------------------------
# worker bodies (run inside forked child processes)
# ---------------------------------------------------------------------------
def _jump_body(i, port, round_index, barrier, codes, per_client):
    samples: list[float] = []
    errors: list[str] = []
    proto: list[str] = []
    conn = http.client.HTTPConnection(HOST, port, timeout=20)
    try:
        conn.connect()
        try:
            barrier.wait(timeout=60)
        except BaseException as exc:
            errors.append(f"jump-r{round_index}-{i}: barrier {exc!r}")
            return {"samples": samples, "errors": errors, "proto": proto}
        for step in range(per_client):
            code = codes[(round_index * 131 + i * 17 + step * 29) % len(codes)]
            try:
                r = request_once(conn, "GET", "/" + code)
            except BaseException as exc:
                errors.append(f"{code}: exception={exc!r}")
                break
            issues = protocol_issues(r)
            for issue in issues:
                proto.append(f"jump-r{round_index}: {issue}")
            if r["status"] != 302:
                errors.append(f"{code}: status={r['status']}")
                continue
            if r["location"] != f"https://example.com/seed/{code}":
                errors.append(f"{code}: bad Location")
                continue
            if issues:
                continue
            samples.append(r["elapsed_ms"])
    finally:
        try:
            conn.close()
        except Exception:
            pass
    return {"samples": samples, "errors": errors, "proto": proto}


def _create_body(i, port, round_index, barrier, per_client):
    samples: list[float] = []
    errors: list[str] = []
    proto: list[str] = []
    conn = http.client.HTTPConnection(HOST, port, timeout=20)
    try:
        conn.connect()
        try:
            barrier.wait(timeout=60)
        except BaseException as exc:
            errors.append(f"create-r{round_index}-{i}: barrier {exc!r}")
            return {"samples": samples, "errors": errors, "proto": proto}
        for step in range(per_client):
            url = (f"https://example.com/perf/create/r{round_index}"
                   f"/c{i}/i{step}")
            body = json.dumps({"url": url}).encode()
            try:
                r = request_once(conn, "POST", "/api/links", body=body,
                                 headers={"Content-Type": "application/json"})
            except BaseException as exc:
                errors.append(f"{url}: exception={exc!r}")
                break
            issues = protocol_issues(r)
            for issue in issues:
                proto.append(f"create-r{round_index}: {issue}")
            if r["status"] != 201:
                errors.append(f"{url}: status={r['status']}")
                continue
            if issues:
                continue
            samples.append(r["elapsed_ms"])
    finally:
        try:
            conn.close()
        except Exception:
            pass
    return {"samples": samples, "errors": errors, "proto": proto}


def _worker_entry(body, i, port, round_index, barrier, send_conn, extra):
    result = {"samples": [], "errors": [f"worker {i} produced no result"], "proto": []}
    try:
        result = body(i, port, round_index, barrier, *extra)
    except BaseException as exc:
        result = {"samples": [], "errors": [f"worker {i} exception={exc!r}"],
                  "proto": []}
    finally:
        try:
            send_conn.send((i, result))
            send_conn.close()
        except BaseException:
            pass


# ---------------------------------------------------------------------------
# multiprocess round drivers
# ---------------------------------------------------------------------------
def _run_round(body, port, round_index, clients, extra):
    ctx = multiprocessing.get_context("fork")
    barrier = ctx.Barrier(clients)
    pipes = [ctx.Pipe(duplex=False) for _ in range(clients)]
    procs = []
    for i in range(clients):
        p = ctx.Process(target=_worker_entry,
                        args=(body, i, port, round_index, barrier, pipes[i][1],
                              extra),
                        daemon=True)
        procs.append(p)
    for p in procs:
        p.start()
    for p in procs:
        p.join(timeout=120)
    results: dict[int, dict] = {}
    missing: list[int] = []
    for i, (recv, _send) in enumerate(pipes):
        if procs[i].is_alive():
            procs[i].terminate()
            procs[i].join(timeout=5)
            missing.append(i)
            continue
        if recv.poll(timeout=10):
            idx, result = recv.recv()
            results[idx] = result
        else:
            missing.append(i)
    ordered = [results.get(i) for i in range(clients)]
    samples = [v for r in ordered if r for v in r["samples"]]
    errors = [e for r in ordered if r for e in r["errors"]]
    proto = [x for r in ordered if r for x in r["proto"]]
    return samples, errors, proto, missing


def jump_round(port, codes, round_index,
               clients=CLIENTS, per_client=JUMP_REQUESTS_PER_CLIENT):
    return _run_round(_jump_body, port, round_index, clients, (codes, per_client))


def create_round(port, round_index,
                 clients=CLIENTS, per_client=CREATE_REQUESTS_PER_CLIENT):
    return _run_round(_create_body, port, round_index, clients, (per_client,))
