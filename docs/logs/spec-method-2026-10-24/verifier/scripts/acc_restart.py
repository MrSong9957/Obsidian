"""Restart, SIGTERM and committed-data persistence checks."""
from __future__ import annotations

import signal
import time

from acc_core import (Group, get_redirect, get_stats, port_open, post_json,
                      truncate)

SIGTERM_LIMIT_S = 5.0  # spec: exit within 5 seconds of SIGTERM


def _stats_obj(rec, port, code, label):
    resp = get_stats(rec, port, code, label=label)
    return resp, resp.json_body()


def phase_restart(ctx, rec, server):
    port = ctx["port"]
    restart_url = "https://example.com/restart"
    ttl_url = "https://example.com/restart-ttl"
    restart_alias = "restart1"
    ttl_alias = "restart-ttl"

    # ---- prepare data on the first boot ---------------------------------
    r = post_json(rec, port, {"url": restart_url, "alias": restart_alias},
                  label="restart-create-link")
    if r.status != 201:
        raise RuntimeError(f"could not create restart link: {r.status} {truncate(r.body, 200)!r}")
    for index in range(2):
        jump = get_redirect(rec, port, restart_alias, label=f"restart-pre-jump-{index}")
        if jump.status != 302:
            raise RuntimeError(f"pre-restart jump failed: {jump.status}")
    _, before = _stats_obj(rec, port, restart_alias, "restart-pre-stats")
    if not isinstance(before, dict):
        raise RuntimeError("pre-restart stats body is not an object")

    r = post_json(rec, port, {"url": ttl_url, "alias": ttl_alias,
                             "ttl_seconds": 3600}, label="restart-create-ttl")
    if r.status != 201:
        raise RuntimeError(f"could not create restart TTL link: {r.status}")
    _, ttl_before = _stats_obj(rec, port, ttl_alias, "restart-ttl-pre-stats")
    if not isinstance(ttl_before, dict):
        raise RuntimeError("pre-restart TTL stats body is not an object")

    # ---- graceful stop ---------------------------------------------------
    stop = server.stop(signal.SIGTERM, timeout=SIGTERM_LIMIT_S + 5.0)
    closed = not port_open(port)
    group = Group(rec, "RESTART-01")
    group.check(not stop.timed_out,
                f"SIGTERM did not stop the service within {SIGTERM_LIMIT_S + 5.0:.0f}s")
    group.check(stop.elapsed_s <= SIGTERM_LIMIT_S,
                f"exit took {stop.elapsed_s:.3f}s (spec limit {SIGTERM_LIMIT_S:.1f}s)")
    group.check(closed, f"port {port} still accepting after exit")
    group.finish()

    # ---- second boot with the same DB_PATH ------------------------------
    boot = server.start(2)
    ctx["port"] = port = boot["port"]

    resp, after = _stats_obj(rec, port, restart_alias, "restart-post-stats")
    group = Group(rec, "RESTART-02")
    group.check(resp.status == 200, f"stats status={resp.status}")
    if isinstance(after, dict):
        group.check(after.get("code") == restart_alias, f"code={after.get('code')!r}")
        group.check(after.get("url") == restart_url, f"url={after.get('url')!r}")
        group.check(after.get("clicks") == 2, f"clicks={after.get('clicks')!r}")
        group.check(after.get("created_at") == before.get("created_at"),
                    f"created_at changed: {before.get('created_at')!r} -> {after.get('created_at')!r}")
        group.check(after.get("expires_at") == before.get("expires_at"),
                    f"expires_at changed: {before.get('expires_at')!r} -> {after.get('expires_at')!r}")
        group.check(after.get("expired") is False, f"expired={after.get('expired')!r}")
    else:
        group.check(False, f"stats body={truncate(resp.body, 200)!r}")
    group.finish()

    jump = get_redirect(rec, port, restart_alias, label="restart-post-jump")
    resp, after_jump = _stats_obj(rec, port, restart_alias, "restart-post-jump-stats")
    group = Group(rec, "RESTART-03")
    group.check(jump.status == 302, f"jump status={jump.status}")
    group.check(jump.get("Location") == restart_url, f"Location={jump.get('Location')!r}")
    group.check(isinstance(after_jump, dict) and after_jump.get("clicks") == 3,
                f"clicks={after_jump.get('clicks') if isinstance(after_jump, dict) else None!r}")
    group.finish()

    resp, ttl_after = _stats_obj(rec, port, ttl_alias, "restart-ttl-post-stats")
    group = Group(rec, "RESTART-04")
    group.check(resp.status == 200, f"TTL stats status={resp.status}")
    if isinstance(ttl_after, dict):
        group.check(ttl_after.get("expires_at") == ttl_before.get("expires_at"),
                    f"expires_at changed: {ttl_before.get('expires_at')!r} -> "
                    f"{ttl_after.get('expires_at')!r}")
        group.check(ttl_after.get("created_at") == ttl_before.get("created_at"),
                    "created_at changed")
        group.check(ttl_after.get("url") == ttl_url, f"url={ttl_after.get('url')!r}")
        group.check(ttl_after.get("expired") is False, f"expired={ttl_after.get('expired')!r}")
        group.check(ttl_after.get("clicks") == 0, f"clicks={ttl_after.get('clicks')!r}")
    else:
        group.check(False, f"TTL stats body={truncate(resp.body, 200)!r}")
    group.finish()

    # ---- 201 means committed: kill -9 right after the response ----------
    durable_url = "https://example.com/durable"
    created = post_json(rec, port, {"url": durable_url, "alias": "durable1"},
                        label="durable-create")
    group = Group(rec, "RESTART-05")
    group.check(created.status == 201,
                f"create status={created.status} body={truncate(created.body, 200)!r}")
    hard_stop = server.stop(signal.SIGKILL, timeout=10.0)
    group.check(not hard_stop.timed_out, "SIGKILL stop timed out")
    boot = server.start(3)
    ctx["port"] = port = boot["port"]
    resp, durable_stats = _stats_obj(rec, port, "durable1", "durable-post-stats")
    group.check(resp.status == 200,
                f"link created before SIGKILL is missing: status={resp.status}")
    if isinstance(durable_stats, dict):
        group.check(durable_stats.get("url") == durable_url,
                    f"url={durable_stats.get('url')!r}")
        group.check(durable_stats.get("clicks") == 0,
                    f"clicks={durable_stats.get('clicks')!r}")
    else:
        group.check(False, f"body={truncate(resp.body, 200)!r}")
    group.finish()

    # ---- data still consistent after the unclean shutdown ---------------
    resp, recovered = _stats_obj(rec, port, restart_alias, "recovered-stats")
    resp_ttl, recovered_ttl = _stats_obj(rec, port, ttl_alias, "recovered-ttl-stats")
    group = Group(rec, "RESTART-06")
    group.check(resp.status == 200, f"stats status={resp.status}")
    group.check(isinstance(recovered, dict) and recovered.get("clicks") == 3,
                f"clicks={recovered.get('clicks') if isinstance(recovered, dict) else None!r}")
    group.check(resp_ttl.status == 200, f"TTL stats status={resp_ttl.status}")
    group.check(isinstance(recovered_ttl, dict)
                and recovered_ttl.get("expires_at") == ttl_before.get("expires_at"),
                f"TTL expires_at={recovered_ttl.get('expires_at') if isinstance(recovered_ttl, dict) else None!r}")
    group.finish()

    ctx["boot_no"] = 3
    ctx["restart_done"] = True
    rec.trace({"ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
               "label": "restart",
               "sigterm_seconds": round(stop.elapsed_s, 3),
               "boot_ports": [b["port"] for b in server.boots]})
