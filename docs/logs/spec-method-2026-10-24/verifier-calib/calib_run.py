#!/usr/bin/env python3
"""One cell-repeat of the calibration matrix (CALIBRATION-PLAN section 1).

Cells: {T,P} x {E,A}
  T = thread client, reuses the FROZEN verifier/acc_perf.py code path
      (_jump_round / _create_round, read-only import)
  P = alternative client (50 independent single-threaded processes,
      alt_client.py)
  E = minimal echo server (echo_server, C)
  A = artifact under test (implementer/artifact/shortlink, fresh instance
      + fresh DB per repeat, 1000 seed links before measurement)

Per repeat: full frozen protocol for both protocols
  jump:  warmup 50 (serial single connection) -> 3 rounds x 500
  create: warmup 20 (serial) -> 3 rounds x 100
p95 = round(sorted[ceil(0.95n)-1], 2) per round; median of the 3 round p95s.
Raw output: verifier-calib/data/<CELL>-rep<N>.json
"""
from __future__ import annotations

import argparse
import http.client
import json
import os
import re
import signal
import subprocess
import sys
import threading
import time

HERE = os.path.dirname(os.path.abspath(__file__))
VERIFIER_DIR = os.path.abspath(os.path.join(HERE, "..", "verifier"))
sys.dont_write_bytecode = True          # never write into verifier/
sys.path.insert(0, VERIFIER_DIR)
from acc_core import median3, nearest_rank_p95, service_snapshot  # noqa: E402
import acc_perf as frozen_perf          # noqa: E402  (frozen thread client)

sys.path.insert(0, HERE)
import alt_client                        # noqa: E402

HOST = "127.0.0.1"
SEED_COUNT = 1000
SEED_THREADS = 20
CODES = [f"seed{i:04d}" for i in range(1, SEED_COUNT + 1)]
START_RE = re.compile(r"listening on http://127\.0\.0\.1:(\d+)")


def loadavg() -> str:
    with open("/proc/loadavg", "r", encoding="ascii") as f:
        return f.read().strip()


def utc_now() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def write_json(path, value):
    with open(path, "w", encoding="utf-8") as f:
        json.dump(value, f, ensure_ascii=False, indent=1)
        f.write("\n")


# ---------------------------------------------------------------------------
# servers
# ---------------------------------------------------------------------------
class Echo:
    def __init__(self, log_dir):
        self.bin = os.path.join(HERE, "echo_server")
        os.makedirs(log_dir, exist_ok=True)
        self.err = open(os.path.join(log_dir, "echo.err"), "wb")
        self.proc = subprocess.Popen([self.bin], stdin=subprocess.DEVNULL,
                                     stdout=subprocess.PIPE, stderr=self.err,
                                     start_new_session=True)
        line = self.proc.stdout.readline().decode("ascii").strip()
        self.port = int(line)
        threading.Thread(target=self.proc.stdout.read, daemon=True).start()

    def stop(self):
        try:
            os.killpg(os.getpgid(self.proc.pid), signal.SIGTERM)
        except OSError:
            pass
        try:
            self.proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(os.getpgid(self.proc.pid), signal.SIGKILL)
            except OSError:
                pass
            self.proc.wait(timeout=5)
        self.err.close()


class Artifact:
    def __init__(self, cwd, db_path, log_dir):
        os.makedirs(os.path.dirname(db_path), exist_ok=True)
        os.makedirs(log_dir, exist_ok=True)
        env = os.environ.copy()
        env["PORT"] = "0"
        env["DB_PATH"] = db_path
        self.out = open(os.path.join(log_dir, "boot.out"), "wb")
        self.err = open(os.path.join(log_dir, "boot.err"), "wb")
        self.proc = subprocess.Popen(["./shortlink"], cwd=cwd, env=env,
                                     stdin=subprocess.DEVNULL,
                                     stdout=subprocess.PIPE, stderr=self.err,
                                     start_new_session=True)
        line = self.proc.stdout.readline().decode("utf-8", "replace").strip()
        m = START_RE.fullmatch(line)
        if not m:
            self.stop()
            raise RuntimeError(f"bad startup line: {line!r}")
        self.port = int(m.group(1))
        threading.Thread(target=lambda: self.out.write(self.proc.stdout.read())
                         or None, daemon=True).start()

    def stop(self):
        started = time.monotonic()
        rc = None
        try:
            os.killpg(os.getpgid(self.proc.pid), signal.SIGTERM)
        except OSError:
            pass
        try:
            rc = self.proc.wait(timeout=15)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(os.getpgid(self.proc.pid), signal.SIGKILL)
            except OSError:
                pass
            self.proc.wait(timeout=5)
            rc = self.proc.returncode
        elapsed = time.monotonic() - started
        self.out.close()
        self.err.close()
        return {"elapsed_s": round(elapsed, 3), "returncode": rc}


# ---------------------------------------------------------------------------
# fixture: 1000 seed links, 20 threads (A21 shape, mirrors phase_seed)
# ---------------------------------------------------------------------------
def seed_links(port):
    per_thread = SEED_COUNT // SEED_THREADS
    barrier = threading.Barrier(SEED_THREADS)
    results: list = [None] * SEED_THREADS

    def worker(index):
        errors = []
        created = 0
        conn = http.client.HTTPConnection(HOST, port, timeout=20)
        try:
            conn.connect()
            barrier.wait(timeout=60)
            for offset in range(per_thread):
                alias = f"seed{index * per_thread + offset + 1:04d}"
                body = json.dumps({"url": f"https://example.com/seed/{alias}",
                                   "alias": alias}).encode()
                conn.request("POST", "/api/links", body=body,
                             headers={"Content-Type": "application/json"})
                resp = conn.getresponse()
                data = resp.read()
                if resp.status != 201:
                    errors.append(f"{alias}: status={resp.status}")
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
    for t in threads:
        t.start()
    for t in threads:
        t.join(timeout=180)
    created_total = sum(r[0] for r in results if r)
    all_errors = [e for r in results if r for e in r[1]]
    missing = [i for i, r in enumerate(results) if r is None]
    return {"created": created_total, "expected": SEED_COUNT,
            "errors": all_errors[:10], "error_count": len(all_errors),
            "missing_workers": missing}


# ---------------------------------------------------------------------------
# measurement
# ---------------------------------------------------------------------------
def measure_jump(client, port):
    warm_samples, warm_errors, _ = alt_client.jump_warmup(port, CODES, 50)
    rounds = []
    for r in range(3):
        if client == "T":
            lat, errors, proto, missing = frozen_perf._jump_round(port, CODES, r)
            samples = lat  # frozen path samples every completed response
        else:
            samples, errors, proto, missing = alt_client.jump_round(port, CODES, r)
        p95 = nearest_rank_p95(samples) if samples else None
        rounds.append({
            "round": r + 1,
            "sample_count": len(samples),
            "error_count": len(errors),
            "errors": errors[:10],
            "protocol_issues": proto[:10],
            "missing_workers": missing,
            "p95_ms": round(p95, 2) if p95 is not None else None,
            "median_ms": round(sorted(samples)[len(samples) // 2], 3)
                         if samples else None,
            "latencies_ms": [round(v, 3) for v in samples],
        })
    p95s = [r["p95_ms"] for r in rounds if r["p95_ms"] is not None]
    return {"warmup_error_count": len(warm_errors),
            "warmup_errors": warm_errors[:10],
            "rounds": rounds,
            "p95_median_ms": median3(p95s) if len(p95s) == 3 else None}


def measure_create(client, port):
    warm_samples, warm_errors, _ = alt_client.create_warmup(port, 20)
    rounds = []
    for r in range(3):
        if client == "T":
            lat, errors, proto, missing = frozen_perf._create_round(port, r)
            samples = lat
        else:
            samples, errors, proto, missing = alt_client.create_round(port, r)
        p95 = nearest_rank_p95(samples) if samples else None
        rounds.append({
            "round": r + 1,
            "sample_count": len(samples),
            "error_count": len(errors),
            "errors": errors[:10],
            "protocol_issues": proto[:10],
            "missing_workers": missing,
            "p95_ms": round(p95, 2) if p95 is not None else None,
            "median_ms": round(sorted(samples)[len(samples) // 2], 3)
                         if samples else None,
            "latencies_ms": [round(v, 3) for v in samples],
        })
    p95s = [r["p95_ms"] for r in rounds if r["p95_ms"] is not None]
    return {"warmup_error_count": len(warm_errors),
            "warmup_errors": warm_errors[:10],
            "rounds": rounds,
            "p95_median_ms": median3(p95s) if len(p95s) == 3 else None}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cell", required=True,
                        choices=["T-E", "P-E", "T-A", "P-A"])
    parser.add_argument("--repeat", type=int, required=True)
    parser.add_argument("--data-dir", default=os.path.join(HERE, "data"))
    args = parser.parse_args()

    client, server = args.cell.split("-")
    os.makedirs(args.data_dir, exist_ok=True)
    rep_dir = os.path.join(args.data_dir, f"{args.cell}-rep{args.repeat}")
    os.makedirs(rep_dir, exist_ok=True)

    out = {
        "cell": args.cell,
        "client": ("thread (frozen acc_perf code path)" if client == "T"
                   else "process (alt_client, 50 single-threaded processes)"),
        "server": ("minimal echo (C, fork-per-connection)" if server == "E"
                   else "artifact shortlink (fresh instance + DB)"),
        "repeat": args.repeat,
        "started_utc": utc_now(),
        "loadavg_before": loadavg(),
        "parameters": {
            "jump": "50 clients x 10 req/round, warmup 50 serial, 3 rounds",
            "create": "50 clients x 2 req/round, warmup 20 serial, 3 rounds, fresh URLs per round",
            "p95": "round(sorted[ceil(0.95*n)-1], 2); median of 3 round p95",
            "timing_scope": "perf_counter around request write -> response fully read; connect excluded",
            "seed_fixture": "1000 seed links via 20 threads (artifact cells only; echo is stateless)",
        },
    }

    artifact = None
    echo = None
    pid_for_memory = None
    try:
        if server == "E":
            echo = Echo(rep_dir)
            port = echo.port
            pid_for_memory = echo.proc.pid
        else:
            db_path = os.path.join(rep_dir, "db", "nested", "links.db")
            artifact = Artifact(
                os.path.abspath(os.path.join(HERE, "..", "implementer",
                                             "artifact")),
                db_path, rep_dir)
            port = artifact.port
            pid_for_memory = artifact.proc.pid
            out["startup_port"] = port
            out["seed"] = seed_links(port)

        out["jump"] = measure_jump(client, port)
        out["create"] = measure_create(client, port)

        if server == "A":
            out["memory"] = service_snapshot(port, pid_for_memory)
    finally:
        if server == "E" and echo is not None:
            echo.stop()
        if server == "A" and artifact is not None:
            out["stop"] = artifact.stop()

    out["finished_utc"] = utc_now()
    out["loadavg_after"] = loadavg()
    path = os.path.join(args.data_dir, f"{args.cell}-rep{args.repeat}.json")
    write_json(path, out)

    jump = out["jump"]
    create = out["create"]
    seed_info = out.get("seed", {}).get("created", "n/a (echo)")
    print(f"[{args.cell} rep{args.repeat}] seed={seed_info} "
          f"jump p95 rounds={[r['p95_ms'] for r in jump['rounds']]} "
          f"median={jump['p95_median_ms']} "
          f"errors={sum(r['error_count'] for r in jump['rounds']) + jump['warmup_error_count']} | "
          f"create p95 rounds={[r['p95_ms'] for r in create['rounds']]} "
          f"median={create['p95_median_ms']} "
          f"errors={sum(r['error_count'] for r in create['rounds']) + create['warmup_error_count']} "
          f"| loadavg {out['loadavg_before']} -> {out['loadavg_after']}", flush=True)
    if server == "A":
        mem = out.get("memory", {})
        print(f"    memory: VmHWM sum={mem.get('memory_kb')} KiB "
              f"processes={mem.get('process_count')} "
              f"stop={out.get('stop')}", flush=True)


if __name__ == "__main__":
    main()
