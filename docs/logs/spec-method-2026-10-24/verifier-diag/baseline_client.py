#!/usr/bin/env python3
"""Client-side baseline load generator (Task D).

Replicates the exact load shape and timing scope of the frozen
verifier/acc_perf.py jump phase:
  - 50 concurrent clients (threads), barrier-synchronized round start
  - each client: one persistent http.client connection, 10 sequential GETs
  - warmup 50 on a single serial connection (not measured)
  - 3 rounds x 500 measured requests
  - timer: time.perf_counter() around conn.request -> getresponse -> read
    (client-observed round trip, TCP connect excluded)
  - p95 = sorted[ceil(0.95*n)-1], rounded to 2 decimals per round,
    median of the 3 round p95 values (same as acc_perf.py / A15)
Only difference vs acc_perf.py: no status/Location assertions on a real
service (the echo server always answers 302 with a fixed Location).
"""
import http.client
import json
import math
import sys
import threading
import time

HOST = "127.0.0.1"
CLIENTS = 50
REQUESTS_PER_CLIENT = 10   # -> 500 per round
WARMUP = 50
ROUNDS = 3


def wait_barrier(barrier, errors, label):
    try:
        barrier.wait(timeout=60)
        return True
    except Exception as exc:
        errors.append(f"{label}: barrier {exc!r}")
        return False


def one_round(port, round_index):
    barrier = threading.Barrier(CLIENTS)
    results = [None] * CLIENTS

    def worker(index):
        latencies = []
        errors = []
        conn = http.client.HTTPConnection(HOST, port, timeout=20)
        try:
            conn.connect()
            wait_barrier(barrier, errors, f"r{round_index}-{index}")
            for step in range(REQUESTS_PER_CLIENT):
                started = time.perf_counter()
                conn.request("GET", f"/seed{1000 + step}")
                resp = conn.getresponse()
                resp.read()
                elapsed = (time.perf_counter() - started) * 1000.0
                latencies.append(elapsed)
                if resp.status != 302:
                    errors.append(f"status={resp.status}")
        except BaseException as exc:
            errors.append(f"exception={exc!r}")
        finally:
            conn.close()
        results[index] = (latencies, errors)

    threads = [threading.Thread(target=worker, args=(i,), daemon=True)
               for i in range(CLIENTS)]
    for t in threads:
        t.start()
    for t in threads:
        t.join(timeout=120)
    latencies = [v for r in results if r for v in r[0]]
    errors = [e for r in results if r for e in r[1]]
    missing = [i for i, r in enumerate(results) if r is None]
    return latencies, errors, missing


def nearest_rank_p95(samples):
    ordered = sorted(samples)
    return ordered[math.ceil(0.95 * len(ordered)) - 1]


def main():
    port = int(sys.argv[1])
    warmup_errors = []
    conn = http.client.HTTPConnection(HOST, port, timeout=20)
    try:
        conn.connect()
        for i in range(WARMUP):
            conn.request("GET", f"/warmup{i}")
            resp = conn.getresponse()
            resp.read()
            if resp.status != 302:
                warmup_errors.append(f"warmup {i}: status={resp.status}")
    except BaseException as exc:
        warmup_errors.append(f"exception={exc!r}")
    finally:
        conn.close()

    rounds = []
    for r in range(ROUNDS):
        latencies, errors, missing = one_round(port, r)
        p95 = nearest_rank_p95(latencies) if latencies else None
        rounds.append({
            "round": r + 1,
            "request_count": len(latencies),
            "p95_ms": round(p95, 2) if p95 is not None else None,
            "median_ms": round(sorted(latencies)[len(latencies) // 2], 2)
                         if latencies else None,
            "errors": errors[:10],
            "missing_workers": missing,
            "latencies_ms": [round(v, 3) for v in latencies],
        })
    p95s = [r["p95_ms"] for r in rounds if r["p95_ms"] is not None]
    result = {
        "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "shape": "50 threads x 10 req/round, warmup 50 serial, 3 rounds, "
                 "persistent connections, perf_counter request->body-read",
        "warmup_errors": warmup_errors,
        "rounds": rounds,
        "p95_median_ms": sorted(p95s)[1] if len(p95s) == 3 else None,
    }
    print(json.dumps({k: v for k, v in result.items()
                      if k not in ("rounds",)},
                     ensure_ascii=False))
    for r in rounds:
        print(f"round {r['round']}: n={r['request_count']} "
              f"p95={r['p95_ms']}ms median={r['median_ms']}ms "
              f"errors={r['errors']} missing={r['missing_workers']}")
    with open(sys.argv[2], "w") as fh:
        json.dump(result, fh, indent=1)


if __name__ == "__main__":
    main()
