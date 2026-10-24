"""Negative-input checks and unknown-code handling."""
from __future__ import annotations

import json

from acc_core import (Group, expect_error, get_redirect, get_stats, post_json,
                      truncate)

URL_PREFIX = "https://example.com/"


def _post_raw(rec, port, body, content_type="application/json", label="",
              check_error_body=True):
    return post_json(rec, port, body, content_type=content_type, label=label,
                     check_error_body=check_error_body)


def phase_validation(ctx, rec):
    port = ctx["port"]

    # VALID-01: Content-Type missing or not application/json
    group = Group(rec, "VALID-01")
    body = json.dumps({"url": "https://example.com/ct-missing"}).encode()
    resp = _post_raw(rec, port, body, content_type=None, label="ct-missing")
    expect_error(rec, group, resp, 400, "invalid_request", "missing Content-Type")
    resp = _post_raw(rec, port, body, content_type="text/plain", label="ct-text-plain")
    expect_error(rec, group, resp, 400, "invalid_request", "text/plain")
    resp = _post_raw(rec, port, body, content_type="application/x-www-form-urlencoded",
                     label="ct-form")
    expect_error(rec, group, resp, 400, "invalid_request", "form content type")
    group.finish()

    # VALID-02: malformed JSON
    group = Group(rec, "VALID-02")
    for label, body in (("empty", b""), ("open-brace", b"{"),
                        ("trailing-comma", b'{"url":"https://example.com/x",}'),
                        ("plain-text", b"not json")):
        resp = _post_raw(rec, port, body, label=f"json-{label}")
        expect_error(rec, group, resp, 400, "invalid_request", f"json {label}")
    group.finish()

    # VALID-03: valid JSON but not an object
    group = Group(rec, "VALID-03")
    for label, body in (("array", b"[]"), ("string", b'"hello"'),
                        ("number", b"123"), ("null", b"null"), ("true", b"true")):
        resp = _post_raw(rec, port, body, label=f"nonobject-{label}")
        expect_error(rec, group, resp, 400, "invalid_request", f"non-object {label}")
    group.finish()

    # VALID-04: url wrong type or missing
    group = Group(rec, "VALID-04")
    resp = _post_raw(rec, port, {"url": 123}, label="url-number")
    expect_error(rec, group, resp, 400, "invalid_request", "url number")
    resp = _post_raw(rec, port, {"url": None}, label="url-null")
    expect_error(rec, group, resp, 400, "invalid_request", "url null")
    resp = _post_raw(rec, port, {"url": ["https://example.com/"]}, label="url-array")
    expect_error(rec, group, resp, 400, "invalid_request", "url array")
    # Missing url: the spec does not say whether this is invalid_request or
    # invalid_url; either is accepted and recorded as an ambiguity.
    resp = _post_raw(rec, port, {}, label="url-missing")
    expect_error(rec, group, resp, 400, "invalid_request", "url missing",
                 allowed_errors={"invalid_request", "invalid_url"})
    resp = _post_raw(rec, port, {"alias": "missingUrl"}, label="url-missing-alias")
    expect_error(rec, group, resp, 400, "invalid_request", "url missing with alias",
                 allowed_errors={"invalid_request", "invalid_url"})
    group.finish()

    # VALID-05: scheme, host, and spaces in scheme/host
    group = Group(rec, "VALID-05")
    for label, url in (("ftp", "ftp://example.com/x"),
                       ("empty-http", "http://"),
                       ("empty-https", "https://"),
                       ("triple-slash", "https:///path"),
                       ("query-no-host", "http://?x=1"),
                       ("space-only", " "),
                       ("scheme-space", "ht tp://example.com/x"),
                       ("host-space", "https://exa mple.com/x"),
                       ("host-leading-space", "https:// example.com/x"),
                       ("host-trailing-space", "https://example.com /x")):
        resp = post_json(rec, port, {"url": url}, label=f"url-{label}")
        expect_error(rec, group, resp, 400, "invalid_url", f"url {label}")
    group.finish()

    # VALID-06: 1-2048 UTF-8 bytes
    group = Group(rec, "VALID-06")
    url_2048 = URL_PREFIX + "a" * (2048 - len(URL_PREFIX))
    url_2049 = URL_PREFIX + "a" * (2049 - len(URL_PREFIX))
    cn_2048 = URL_PREFIX + "中" * ((2048 - len(URL_PREFIX)) // 3)
    cn_over = URL_PREFIX + "中" * 1000
    group.check(len(url_2048.encode()) == 2048, "constructed 2048-byte URL is wrong")
    group.check(len(url_2049.encode()) == 2049, "constructed 2049-byte URL is wrong")
    group.check(len(cn_2048.encode()) == 2048, "constructed 2048-byte Chinese URL is wrong")
    group.check(len(cn_over.encode()) > 2048, "constructed oversized Chinese URL is wrong")
    resp = post_json(rec, port, {"url": url_2048}, label="url-2048")
    group.check(resp.status == 201,
                f"2048-byte URL: status={resp.status} body={truncate(resp.body, 120)!r}")
    resp = post_json(rec, port, {"url": url_2049}, label="url-2049")
    expect_error(rec, group, resp, 400, "invalid_url", "2049-byte URL")
    resp = post_json(rec, port, {"url": cn_2048}, label="url-2048-cn")
    group.check(resp.status == 201,
                f"2048-byte Chinese URL: status={resp.status} body={truncate(resp.body, 120)!r}")
    resp = post_json(rec, port, {"url": cn_over}, label="url-over-cn")
    expect_error(rec, group, resp, 400, "invalid_url",
                 "URL whose UTF-8 length exceeds 2048 bytes")
    group.finish()

    # VALID-07: ASCII control characters (0x00-0x1F, 0x7F) inside URL
    group = Group(rec, "VALID-07")
    for label, url in (("nul", "https://example.com/a\x00b"),
                       ("tab", "https://example.com/a\tb"),
                       ("newline", "https://example.com/a\nb"),
                       ("carriage-return", "https://example.com/a\rb"),
                       ("unit-separator", "https://example.com/a\x1fb"),
                       ("del", "https://example.com/a\x7fb")):
        resp = post_json(rec, port, {"url": url}, label=f"url-control-{label}")
        expect_error(rec, group, resp, 400, "invalid_url", f"url with {label}")
    group.finish()

    # VALID-08: '%' must be followed by two hex digits
    group = Group(rec, "VALID-08")
    for label, url in (("trailing", "https://example.com/100%"),
                       ("one-digit", "https://example.com/%2"),
                       ("non-hex", "https://example.com/%zz"),
                       ("non-hex-2", "https://example.com/%2G")):
        resp = post_json(rec, port, {"url": url}, label=f"url-pct-{label}")
        expect_error(rec, group, resp, 400, "invalid_url", f"url percent {label}")
    group.finish()

    # VALID-09: ttl_seconds type and range
    group = Group(rec, "VALID-09")
    for label, value in (("zero", 0), ("negative", -1), ("too-big", 31536001),
                         ("float", 1.0), ("string", "60"), ("bool-true", True),
                         ("bool-false", False)):
        resp = post_json(rec, port, {"url": "https://example.com/ttl-invalid",
                                    "ttl_seconds": value},
                         label=f"ttl-{label}")
        expect_error(rec, group, resp, 400, "invalid_request", f"ttl {label}")
    group.finish()

    # VALID-10: alias length/charset
    group = Group(rec, "VALID-10")
    for label, alias in (("too-short", "abc"), ("too-long", "a" * 33),
                         ("dot", "ab.cd"), ("space", "ab cd"),
                         ("empty", ""), ("slash", "ab/cd"),
                         ("non-ascii", "别名abc")):
        resp = post_json(rec, port, {"url": "https://example.com/alias-invalid",
                                    "alias": alias},
                         label=f"alias-{label}")
        expect_error(rec, group, resp, 400, "invalid_alias", f"alias {label}")
    group.finish()

    # VALID-11: reserved aliases
    group = Group(rec, "VALID-11")
    for alias in ("api", "health"):
        resp = post_json(rec, port, {"url": "https://example.com/reserved",
                                    "alias": alias},
                         label=f"alias-reserved-{alias}")
        expect_error(rec, group, resp, 400, "invalid_alias", f"reserved alias {alias}")
    group.finish()

    # VALID-12: duplicate alias -> 409 (sequential)
    group = Group(rec, "VALID-12")
    first = post_json(rec, port, {"url": "https://example.com/dup-one",
                                 "alias": "dup-alias"}, label="dup-first")
    second = post_json(rec, port, {"url": "https://example.com/dup-two",
                                  "alias": "dup-alias"}, label="dup-second")
    group.check(first.status == 201, f"first create status={first.status}")
    expect_error(rec, group, second, 409, "alias_taken", "duplicate alias")
    group.finish()

    # VALID-14: bodies larger than 1 MiB return exactly 400 invalid_request
    group = Group(rec, "VALID-14")
    oversized = json.dumps({"url": "https://example.com/too-big",
                            "pad": "x" * (1 << 20)}).encode()
    group.check(len(oversized) > 1 << 20, f"body length={len(oversized)}")
    try:
        resp = _post_raw(rec, port, oversized, label="body-over-1mib")
    except Exception as exc:
        group.check(False,
                    f"oversized body was not answered with 400 invalid_request: {exc!r}")
    else:
        expect_error(rec, group, resp, 400, "invalid_request", "body over 1 MiB")
    group.finish()
    # service must still answer after the oversized request
    try:
        alive = get_stats(rec, port, ctx["primary"], label="alive-after-oversize")
    except Exception as exc:
        rec.check("VALID-15", False,
                  f"stats after oversized body raised: {exc!r}")
    else:
        rec.check("VALID-15", alive.status == 200,
                  f"stats after oversized body: status={alive.status}")

    # STATS-07 / REDIR-03: unknown codes
    group = Group(rec, "STATS-07")
    for code in ("nosuch1", "no-such-code"):
        resp = get_stats(rec, port, code, label=f"stats-unknown-{code}")
        expect_error(rec, group, resp, 404, "not_found", f"unknown code {code}")
    group.finish()

    group = Group(rec, "REDIR-03")
    for code in ("nosuch1", "no-such-code"):
        resp = get_redirect(rec, port, code, label=f"redirect-unknown-{code}")
        expect_error(rec, group, resp, 404, "not_found", f"unknown code {code}")
    group.finish()

    # VALID-13 is evaluated after all phases (acceptance.py) so that error
    # bodies produced by later phases are covered as well.
