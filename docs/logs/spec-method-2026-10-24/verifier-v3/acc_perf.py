"""Throughput, latency and memory checks using the frozen measurement plan.

Frozen parameters (run-constraints.md), not configurable:
  jump:   50 concurrent clients x 500 requests/round (10 each), warmup 50, 3 rounds
  create: 50 concurrent clients x 100 requests/round (2 each),  warmup 20, 3 rounds
  p95:    nearest rank, then median of the three round p95 values
  VmHWM:  <= 10240 KiB, read after the stress run

v3 (verifier-v3, 2026-10-24 round 3) -- the ONLY change vs the frozen v2/rev
script is the load-client implementation (see MEASUREMENT-v3.md and
../verifier-calib/): the 50 concurrent clients of each measured round are
50 independent single-threaded processes (multiprocessing, fork context)
instead of 50 threads inside one Python process.  Unchanged: load shape
(clients/requests/warmup/rounds), persistent connections with connect()
before the barrier-synchronised round start, the timing scope
(time.perf_counter() around request write -> response fully read), seed
fixture (1000 links via 20 threads), warmup shape (serial single
connection), p95 maths, thresholds, and every assertion.  One behavioural
consequence of the new client: erroneous requests are NOT added to the
latency samples (any error fails PERF-02/PERF-04 independently, so the
verdict semantics are unchanged).
"""
from __future__ import annotations

import http.client
import json
import math
import multiprocessing
import threading
import time

from acc_core import (HOST, Group, get_redirect, get_stats, median3,
                      nearest_rank_p95, post_json, service_snapshot, truncate,
                      write_json)

SEED_COUNT = 1000
JUMP_CLIENTS = 50
JUMP_REQUESTS = 10            # per client, per round -> 500 per round
JUMP_WARMUP = 50
JUMP_ROUNDS = 3
CREATE_CLIENTS = 50
CREATE_REQUESTS = 2           # per client, per round -> 100 per round
CREATE_WARMUP = 20
CREATE_ROUNDS = 3
JUMP_P95_LIMIT_MS = 10.00
CREATE_P95_LIMIT_MS = 15.00
VMHWM_LIMIT_KB = 10240
SEED_THREADS = 20


def _wait_barrier(barrier, errors, label):
    try:
        barrier.wait(timeout=60)
        return True
    except Exception as exc:
        errors.append(f"{label}: barrier {exc!r}")
        return False


def _check_protocol(resp, body, label, issues):
    if resp.version != 11:
        issues.append(f"{label}: version={resp.version}")
    declared = resp.getheader("Content-Length")
    if declared != str(len(body)):
        issues.append(f"{label}: Content-Length={declared!r} actual={len(body)}")


# ---------------------------------------------------------------------------
# v3 process-based load client
# ---------------------------------------------------------------------------
def _timed_request(conn, method, path, body=None, headers=None):
    """Client-observed round trip: the timer covers the request write through
    the response body being fully read (same scope as the frozen v2 client)."""
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


def _resp_protocol_issues(r, label):
    issues = []
    if r["version"] != 11:
        issues.append(f"{label}: version={r['version']}")
    if r["content_length"] != str(r["body_len"]):
        issues.append(f"{label}: Content-Length={r['content_length']!r} "
                      f"actual={r['body_len']}")
    return issues


def _jump_body(index, port, codes, round_index, barrier):
    """Runs inside a forked child process: one persistent connection, 10 GETs."""
    latencies: list[float] = []
    errors: list[str] = []
    proto: list[str] = []
    conn = http.client.HTTPConnection(HOST, port, timeout=20)
    try:
        conn.connect()
        if not _wait_barrier(barrier, errors, f"jump-r{round_index}-{index}"):
            return {"latencies": latencies, "errors": errors, "proto": proto}
        for step in range(JUMP_REQUESTS):
            code = codes[(round_index * 131 + index * 17 + step * 29) % len(codes)]
            try:
                r = _timed_request(conn, "GET", "/" + code)
            except BaseException as exc:
                errors.append(f"{code}: exception={exc!r}")
                break
            issues = _resp_protocol_issues(r, f"jump-r{round_index}")
            proto.extend(issues)
            if r["status"] != 302:
                errors.append(f"{code}: status={r['status']}")
                continue
            if r["location"] != f"https://example.com/seed/{code}":
                errors.append(f"{code}: bad Location")
                continue
            if issues:
                continue
            latencies.append(r["elapsed_ms"])
    finally:
        conn.close()
    return {"latencies": latencies, "errors": errors, "proto": proto}


def _create_body(index, port, round_index, barrier):
    """Runs inside a forked child process: one persistent connection, 2 POSTs."""
    latencies: list[float] = []
    errors: list[str] = []
    proto: list[str] = []
    conn = http.client.HTTPConnection(HOST, port, timeout=20)
    try:
        conn.connect()
        if not _wait_barrier(barrier, errors, f"create-r{round_index}-{index}"):
            return {"latencies": latencies, "errors": errors, "proto": proto}
        for step in range(CREATE_REQUESTS):
            url = (f"https://example.com/perf/create/r{round_index}"
                   f"/c{index}/i{step}")
            body = json.dumps({"url": url}).encode()
            try:
                r = _timed_request(conn, "POST", "/api/links", body=body,
                                   headers={"Content-Type": "application/json"})
            except BaseException as exc:
                errors.append(f"{url}: exception={exc!r}")
                break
            issues = _resp_protocol_issues(r, f"create-r{round_index}")
            proto.extend(issues)
            if r["status"] != 201:
                errors.append(f"{url}: status={r['status']} body_len={r['body_len']}")
                continue
            if issues:
                continue
            latencies.append(r["elapsed_ms"])
    finally:
        conn.close()
    return {"latencies": latencies, "errors": errors, "proto": proto}


def _worker_entry(body, index, port, round_index, barrier, send_conn, codes):
    result = {"latencies": [], "errors": [f"worker {index} produced no result"],
              "proto": []}
    try:
        if codes is not None:
            result = body(index, port, codes, round_index, barrier)
        else:
            result = body(index, port, round_index, barrier)
    except BaseException as exc:
        result = {"latencies": [], "errors": [f"worker {index} exception={exc!r}"],
                  "proto": []}
    finally:
        try:
            send_conn.send((index, result))
            send_conn.close()
        except BaseException:
            pass


def _run_round(body, port, round_index, clients, codes=None):
    ctx = multiprocessing.get_context("fork")
    barrier = ctx.Barrier(clients)
    pipes = [ctx.Pipe(duplex=False) for _ in range(clients)]
    procs = []
    for index in range(clients):
        proc = ctx.Process(target=_worker_entry,
                           args=(body, index, port, round_index, barrier,
                                 pipes[index][1], codes),
                           daemon=True)
        procs.append(proc)
    for proc in procs:
        proc.start()
    for proc in procs:
        proc.join(timeout=120)
    results: dict[int, dict] = {}
    missing: list[int] = []
    for index, (recv, _send) in enumerate(pipes):
        if procs[index].is_alive():
            procs[index].terminate()
            procs[index].join(timeout=5)
            missing.append(index)
            continue
        if recv.poll(timeout=10):
            idx, result = recv.recv()
            results[idx] = result
        else:
            missing.append(index)
    ordered = [results.get(i) for i in range(clients)]
    latencies = [value for r in ordered if r for value in r["latencies"]]
    errors = [error for r in ordered if r for error in r["errors"]]
    proto = [issue for r in ordered if r for issue in r["proto"]]
    return latencies, errors, proto, missing


def _jump_round(port, codes, round_index):
    return _run_round(_jump_body, port, round_index, JUMP_CLIENTS, codes=codes)


def _create_round(port, round_index):
    return _run_round(_create_body, port, round_index, CREATE_CLIENTS)


def phase_seed(ctx, rec):
    port = ctx["port"]
    aliases = [f"seed{i:04d}" for i in range(1, SEED_COUNT + 1)]
    per_thread = SEED_COUNT // SEED_THREADS
    barrier = threading.Barrier(SEED_THREADS)
    results: list = [None] * SEED_THREADS

    def worker(index):
        errors = []
        created = 0
        conn = http.client.HTTPConnection(HOST, port, timeout=20)
        try:
            conn.connect()
            _wait_barrier(barrier, errors, f"seed-{index}")
            for offset in range(per_thread):
                alias = aliases[index * per_thread + offset]
                body = json.dumps({"url": f"https://example.com/seed/{alias}",
                                   "alias": alias}).encode()
                conn.request("POST", "/api/links", body=body,
                             headers={"Content-Type": "application/json"})
                resp = conn.getresponse()
                data = resp.read()
                if resp.status != 201:
                    errors.append(f"{alias}: status={resp.status} body={truncate(data, 80)!r}")
                    continue
                obj = json.loads(data.decode())
                if obj.get("code") != alias:
                    errors.append(f"{alias}: code={obj.get('code')!r}")
                    continue
                created += 1
        except BaseException as exc:
            errors.append(f"exception={exc!r}")
        finally:
            conn.close()
        results[index] = (created, errors)

    threads = [threading.Thread(target=worker, args=(i,), daemon=True)
               for i in range(SEED_THREADS)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join(timeout=180)
    created_total = sum(r[0] for r in results if r)
    all_errors = [error for r in results if r for error in r[1]]
    missing = [i for i, r in enumerate(results) if r is None]
    if missing:
        all_errors.append(f"workers did not finish: {missing}")
    group = Group(rec, "PERF-01")
    group.check(created_total == SEED_COUNT,
                f"created {created_total}/{SEED_COUNT}")
    group.check(not all_errors, f"errors: {' | '.join(all_errors[:5])}")
    group.finish()
    ctx["seed_codes"] = aliases

    group = Group(rec, "PERF-SEED-SPOT")
    for alias in ("seed0001", "seed1000"):
        resp = get_stats(rec, port, alias, label=f"seed-stats-{alias}")
        obj = resp.json_body()
        group.check(resp.status == 200 and isinstance(obj, dict)
                    and obj.get("url") == f"https://example.com/seed/{alias}",
                    f"{alias}: status={resp.status}")
    group.finish()


def phase_perf(ctx, rec, server):
    port = ctx["port"]

    # ---------------- fixture: 1000 links ------------------------------
    phase_seed(ctx, rec)

    # ---------------- jump warmup --------------------------------------
    codes = ctx["seed_codes"]
    warmup_errors = []
    conn = http.client.HTTPConnection(HOST, port, timeout=20)
    try:
        conn.connect()
        for i in range(JUMP_WARMUP):
            code = codes[i % len(codes)]
            conn.request("GET", "/" + code)
            resp = conn.getresponse()
            data = resp.read()
            if resp.status != 302:
                warmup_errors.append(f"{code}: status={resp.status}")
    except BaseException as exc:
        warmup_errors.append(f"exception={exc!r}")
    finally:
        conn.close()

    jump_rounds = []
    for round_index in range(JUMP_ROUNDS):
        latencies, errors, proto, missing = _jump_round(port, codes, round_index)
        p95 = nearest_rank_p95(latencies) if latencies else None
        jump_rounds.append({
            "round": round_index + 1,
            "request_count": len(latencies),
            "p95_ms": round(p95, 2) if p95 is not None else None,
            "errors": errors[:10],
            "protocol_issues": proto[:10],
            "missing_workers": missing,
            "latencies_ms": [round(value, 3) for value in latencies],
        })

    group = Group(rec, "PERF-02")
    group.check(not warmup_errors, f"warmup errors: {' | '.join(warmup_errors[:5])}")
    for row in jump_rounds:
        r = row["round"]
        group.check(row["request_count"] == JUMP_CLIENTS * JUMP_REQUESTS,
                    f"round {r}: requests={row['request_count']}")
        group.check(not row["errors"],
                    f"round {r}: errors={' | '.join(row['errors'][:3])}")
        group.check(not row["missing_workers"],
                    f"round {r}: missing workers={row['missing_workers']}")
    group.finish()

    jump_p95s = [row["p95_ms"] for row in jump_rounds if row["p95_ms"] is not None]
    jump_median = median3(jump_p95s) if len(jump_p95s) == 3 else None
    group = Group(rec, "PERF-03")
    group.check(jump_median is not None,
                f"p95 rounds={jump_p95s}")
    group.check(jump_median is not None and round(jump_median, 2) <= JUMP_P95_LIMIT_MS,
                f"median p95={jump_median}ms rounds={jump_p95s} limit={JUMP_P95_LIMIT_MS:.2f}ms")
    group.finish()

    jump_proto = [issue for row in jump_rounds for issue in row["protocol_issues"]]
    if jump_proto:
        for issue in jump_proto[:20]:
            rec._append(rec.protocol_issues, f"perf jump {issue}")

    # ---------------- create warmup ------------------------------------
    create_warmup_errors = []
    conn = http.client.HTTPConnection(HOST, port, timeout=20)
    try:
        conn.connect()
        for i in range(CREATE_WARMUP):
            url = f"https://example.com/perf/warmup/{i}"
            body = json.dumps({"url": url}).encode()
            conn.request("POST", "/api/links", body=body,
                         headers={"Content-Type": "application/json"})
            resp = conn.getresponse()
            data = resp.read()
            if resp.status != 201:
                create_warmup_errors.append(f"warmup {i}: status={resp.status}")
    except BaseException as exc:
        create_warmup_errors.append(f"exception={exc!r}")
    finally:
        conn.close()

    create_rounds = []
    for round_index in range(CREATE_ROUNDS):
        latencies, errors, proto, missing = _create_round(port, round_index)
        p95 = nearest_rank_p95(latencies) if latencies else None
        create_rounds.append({
            "round": round_index + 1,
            "request_count": len(latencies),
            "p95_ms": round(p95, 2) if p95 is not None else None,
            "errors": errors[:10],
            "protocol_issues": proto[:10],
            "missing_workers": missing,
            "latencies_ms": [round(value, 3) for value in latencies],
        })

    group = Group(rec, "PERF-04")
    group.check(not create_warmup_errors,
                f"warmup errors: {' | '.join(create_warmup_errors[:5])}")
    for row in create_rounds:
        r = row["round"]
        group.check(row["request_count"] == CREATE_CLIENTS * CREATE_REQUESTS,
                    f"round {r}: requests={row['request_count']}")
        group.check(not row["errors"],
                    f"round {r}: errors={' | '.join(row['errors'][:3])}")
        group.check(not row["missing_workers"],
                    f"round {r}: missing workers={row['missing_workers']}")
    group.finish()

    create_p95s = [row["p95_ms"] for row in create_rounds if row["p95_ms"] is not None]
    create_median = median3(create_p95s) if len(create_p95s) == 3 else None
    group = Group(rec, "PERF-05")
    group.check(create_median is not None, f"p95 rounds={create_p95s}")
    group.check(create_median is not None and round(create_median, 2) <= CREATE_P95_LIMIT_MS,
                f"median p95={create_median}ms rounds={create_p95s} limit={CREATE_P95_LIMIT_MS:.2f}ms")
    group.finish()

    create_proto = [issue for row in create_rounds for issue in row["protocol_issues"]]
    if create_proto:
        for issue in create_proto[:20]:
            rec._append(rec.protocol_issues, f"perf create {issue}")

    # ---------------- memory after the stress run ----------------------
    snapshot = service_snapshot(port, server.proc.pid if server.proc else None)
    group = Group(rec, "PERF-06")
    group.check(snapshot["process_count"] >= 1,
                f"no service process found: processes={snapshot['processes']}")
    group.check(snapshot["memory_kb"] <= VMHWM_LIMIT_KB,
                f"VmHWM sum={snapshot['memory_kb']}KiB limit={VMHWM_LIMIT_KB}KiB "
                f"processes={snapshot['processes']}")
    group.check(all(p["vmhwm_kb"] is not None for p in snapshot["processes"]),
                f"missing VmHWM: {snapshot['processes']}")
    group.finish()

    group = Group(rec, "PERF-07")
    group.check(snapshot["process_count"] == 1,
                f"service process count={snapshot['process_count']} "
                f"pids={snapshot['tree_pids']} "
                f"cmdlines={[p['cmdline'] for p in snapshot['processes']]}")
    group.finish()

    ctx["perf"] = {
        "parameters": {
            "jump_clients": JUMP_CLIENTS,
            "jump_requests_per_round": JUMP_CLIENTS * JUMP_REQUESTS,
            "jump_warmup": JUMP_WARMUP,
            "jump_rounds": JUMP_ROUNDS,
            "create_clients": CREATE_CLIENTS,
            "create_requests_per_round": CREATE_CLIENTS * CREATE_REQUESTS,
            "create_warmup": CREATE_WARMUP,
            "create_rounds": CREATE_ROUNDS,
            "p95_method": "nearest rank ceil(0.95*n)-1; median of 3 round p95",
            "limits_ms": {"jump": JUMP_P95_LIMIT_MS, "create": CREATE_P95_LIMIT_MS},
            "vmhwm_limit_kb": VMHWM_LIMIT_KB,
            "client_implementation": ("v3: 50 independent single-threaded "
                                      "processes (multiprocessing fork), "
                                      "persistent connections, timer covers "
                                      "request write -> response fully read; "
                                      "erroneous requests not sampled"),
        },
        "jump": {"rounds": jump_rounds, "p95_median_ms": jump_median},
        "create": {"rounds": create_rounds, "p95_median_ms": create_median},
        "memory": snapshot,
    }
    write_json(ctx["run_dir"] + "/perf.json", ctx["perf"])
