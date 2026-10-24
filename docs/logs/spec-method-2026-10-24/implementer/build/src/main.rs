// shortlink - short link service per frozen spec spec-method-2026-10-24
// Single process, fixed worker thread pool, one shared SQLite connection
// guarded by a mutex (WAL, synchronous=NORMAL, commits before 201 responses).
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, ErrorCode, params};
use serde_json::{Value, json};

const WORKERS: usize = 64;
const WORKER_STACK: usize = 256 * 1024;
const MAX_HEADER_BYTES: usize = 64 * 1024;
const MAX_BODY_BYTES: usize = 1024 * 1024; // 1 MiB
const READ_TIMEOUT: Duration = Duration::from_secs(65);
const CODE_ALPHABET: &[u8; 62] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const RESERVED_ALIASES: [&str; 2] = ["api", "health"];

// ---------------- pure logic ----------------

#[derive(Debug)]
pub enum CreateErr {
    InvalidRequest,
    InvalidUrl,
    InvalidAlias,
}

pub struct CreateSpec {
    pub url: String,
    pub alias: Option<String>,
    pub ttl: Option<i64>,
}

fn is_hex(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

/// URL rules from spec 1.1: 1-2048 UTF-8 bytes, http/https scheme, non-empty
/// host (no space in host), no ASCII control chars, every `%` followed by two
/// hex digits.
pub fn validate_url(u: &str) -> bool {
    let bytes = u.as_bytes();
    if bytes.is_empty() || bytes.len() > 2048 {
        return false;
    }
    let rest = match u.strip_prefix("http://") {
        Some(r) => r,
        None => match u.strip_prefix("https://") {
            Some(r) => r,
            None => return false,
        },
    };
    // authority runs to the first '/', '?' or '#'
    let auth_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..auth_end];
    if authority.is_empty() {
        return false;
    }
    // host part before an optional ':' must be non-empty ("http://:80/" invalid)
    let host = match authority.split_once(':') {
        Some((h, _)) => h,
        None => authority,
    };
    if host.is_empty() || host.contains(' ') {
        return false;
    }
    // no ASCII control characters anywhere
    if bytes.iter().any(|&c| c < 0x20 || c == 0x7f) {
        return false;
    }
    // every '%' must be followed by exactly two hex digits
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() || !is_hex(bytes[i + 1]) || !is_hex(bytes[i + 2]) {
                return false;
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    true
}

/// alias: 4-32 chars of [A-Za-z0-9_-], case sensitive, "api"/"health" reserved
pub fn valid_alias(a: &str) -> bool {
    (4..=32).contains(&a.len())
        && a.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        && !RESERVED_ALIASES.contains(&a)
}

/// Parse and classify the POST body. Order of checks: request shape
/// (missing/mistyped fields, ttl bounds) -> invalid_request; url rules ->
/// invalid_url; alias rules -> invalid_alias.
pub fn parse_create_body(body: &[u8]) -> Result<CreateSpec, CreateErr> {
    let v: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return Err(CreateErr::InvalidRequest),
    };
    let obj = match v.as_object() {
        Some(o) => o,
        None => return Err(CreateErr::InvalidRequest),
    };
    let url = match obj.get("url") {
        Some(Value::String(s)) => s.clone(),
        _ => return Err(CreateErr::InvalidRequest),
    };
    let alias = match obj.get("alias") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(s.clone()),
        _ => return Err(CreateErr::InvalidRequest),
    };
    let ttl = match obj.get("ttl_seconds") {
        None | Some(Value::Null) => None,
        Some(Value::Number(n)) => match n.as_i64() {
            Some(i) => Some(i),
            None => return Err(CreateErr::InvalidRequest),
        },
        _ => return Err(CreateErr::InvalidRequest),
    };
    if let Some(t) = ttl {
        if !(1..=31536000).contains(&t) {
            return Err(CreateErr::InvalidRequest);
        }
    }
    if !validate_url(&url) {
        return Err(CreateErr::InvalidUrl);
    }
    if let Some(a) = &alias {
        if !valid_alias(a) {
            return Err(CreateErr::InvalidAlias);
        }
    }
    Ok(CreateSpec { url, alias, ttl })
}

const HEX_UPPER: &[u8; 16] = b"0123456789ABCDEF";

/// Location encoding per spec 1.3: existing `%HH` triples kept verbatim
/// (case preserved); ASCII reserved `:/?#[]@!$&'()*+,;=` and unreserved
/// `A-Za-z0-9-._~` kept; everything else percent-encoded as UTF-8 bytes.
pub fn encode_location(u: &str) -> String {
    let b = u.as_bytes();
    let mut out = String::with_capacity(b.len() + 16);
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'%' && i + 2 < b.len() && is_hex(b[i + 1]) && is_hex(b[i + 2]) {
            // keep the triple exactly as written
            out.push('%');
            out.push(b[i + 1] as char);
            out.push(b[i + 2] as char);
            i += 3;
            continue;
        }
        match c {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' => out.push(c as char),
            b'-' | b'.' | b'_' | b'~' => out.push(c as char),
            b':' | b'/' | b'?' | b'#' | b'[' | b']' | b'@' => out.push(c as char),
            b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'=' => {
                out.push(c as char)
            }
            _ => {
                out.push('%');
                out.push(HEX_UPPER[(c >> 4) as usize] as char);
                out.push(HEX_UPPER[(c & 0x0f) as usize] as char);
            }
        }
        i += 1;
    }
    out
}

/// days since 1970-01-01 -> (year, month, day); Howard Hinnant's algorithm
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Unix seconds -> "YYYY-MM-DDTHH:MM:SSZ" (UTC)
pub fn iso8601(secs: i64) -> String {
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m,
        d,
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

static RNG_COUNTER: AtomicU64 = AtomicU64::new(0);

struct Rng(u64);

impl Rng {
    fn new() -> Self {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let pid = std::process::id() as u64;
        let ctr = RNG_COUNTER.fetch_add(1, Ordering::Relaxed) as u64;
        Rng(t ^ (pid << 32) ^ ctr.wrapping_mul(0x9E3779B97F4A7C15))
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    /// 7-char [A-Za-z0-9] code
    fn code7(&mut self) -> String {
        let mut s = String::with_capacity(7);
        let mut r = self.next();
        let mut n = 0;
        while n < 7 {
            if r == 0 {
                r = self.next();
            }
            s.push(CODE_ALPHABET[(r % 62) as usize] as char);
            r /= 62;
            n += 1;
        }
        s
    }
}

// ---------------- shared application state ----------------

struct App {
    tx: mpsc::Sender<Pending>,
    port: u16,
}

impl App {
    /// Submit one DB op and block until the writer thread's batch (which
    /// includes this op) has committed. Responses are only built after that.
    fn submit(&self, op: DbOp) -> DbResult {
        let slot = Arc::new(ReplySlot {
            state: Mutex::new(None),
            cv: std::sync::Condvar::new(),
        });
        let pending = Pending {
            op,
            now: now_secs(),
            slot: Some(Arc::clone(&slot)),
            result: None,
        };
        if self.tx.send(pending).is_err() {
            return DbResult::Internal;
        }
        let mut g = match slot.state.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        loop {
            if let Some(r) = g.take() {
                return r;
            }
            g = match slot.cv.wait(g) {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            };
        }
    }
}

// ---------------- group-commit DB op layer (round 2) ----------------
// All DB work goes through a single writer thread that drains the pending
// queue into one transaction per batch and commits once (group commit).
// Responses are only produced after the batch commit, so the durability
// guarantee is identical to per-request autocommit.

#[derive(Debug)]
enum DbOp {
    Create {
        url: String,
        alias: Option<String>,
        ttl: Option<i64>,
    },
    Stats {
        code: String,
    },
    Jump {
        code: String,
    },
}

#[derive(Debug)]
enum DbResult {
    Created {
        code: String,
        expires_at: Option<i64>,
    },
    StatsRow {
        url: String,
        clicks: i64,
        created_at: i64,
        expires_at: Option<i64>,
    },
    Jumped {
        url: String,
    },
    Missing,
    Expired,
    AliasTaken,
    Internal,
}

struct Pending {
    op: DbOp,
    now: i64,
    slot: Option<Arc<ReplySlot>>,
    result: Option<DbResult>,
}

fn do_create(
    conn: &Connection,
    rng: &mut Rng,
    url: &str,
    alias: Option<&str>,
    ttl: Option<i64>,
    now: i64,
) -> DbResult {
    let expires_at = ttl.map(|t| now + t);
    if let Some(alias) = alias {
        match exec_insert(conn, alias, url, now, expires_at) {
            Ok(()) => DbResult::Created {
                code: alias.to_string(),
                expires_at,
            },
            Err(DbErr::Constraint) => DbResult::AliasTaken,
            Err(DbErr::Other) => DbResult::Internal,
        }
    } else {
        for _ in 0..64 {
            let c = rng.code7();
            match exec_insert(conn, &c, url, now, expires_at) {
                Ok(()) => {
                    return DbResult::Created { code: c, expires_at };
                }
                Err(DbErr::Constraint) => continue,
                Err(DbErr::Other) => return DbResult::Internal,
            }
        }
        DbResult::Internal
    }
}

fn do_stats(conn: &Connection, code: &str) -> DbResult {
    match conn
        .prepare_cached("SELECT url, clicks, created_at, expires_at FROM links WHERE code = ?1")
        .and_then(|mut st| {
            st.query_row(params![code], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                ))
            })
        }) {
        Ok((url, clicks, created_at, expires_at)) => DbResult::StatsRow {
            url,
            clicks,
            created_at,
            expires_at,
        },
        Err(rusqlite::Error::QueryReturnedNoRows) => DbResult::Missing,
        Err(_) => DbResult::Internal,
    }
}

fn do_jump(conn: &Connection, code: &str, now: i64) -> DbResult {
    // Atomic click: increments only while not expired; expires_at <= now is expired.
    match conn
        .prepare_cached(
            "UPDATE links SET clicks = clicks + 1 \
             WHERE code = ?1 AND (expires_at IS NULL OR expires_at > ?2) \
             RETURNING url",
        )
        .and_then(|mut st| st.query_row(params![code, now], |row| row.get::<_, String>(0)))
    {
        Ok(url) => DbResult::Jumped { url },
        Err(rusqlite::Error::QueryReturnedNoRows) => match conn
            .prepare_cached("SELECT 1 FROM links WHERE code = ?1")
            .and_then(|mut st| st.query_row(params![code], |_| Ok(())))
        {
            Ok(()) => DbResult::Expired,
            Err(rusqlite::Error::QueryReturnedNoRows) => DbResult::Missing,
            Err(_) => DbResult::Internal,
        },
        Err(_) => DbResult::Internal,
    }
}

/// Execute one batch of ops in a single transaction; one commit per batch
/// (group commit). Per-statement constraint failures roll back only that
/// statement (SQLite ABORT), so the rest of the batch stays valid.
fn run_batch(conn: &mut Connection, batch: &mut [Pending]) -> bool {
    if conn.execute_batch("BEGIN;").is_err() {
        for p in batch.iter_mut() {
            p.result = Some(DbResult::Internal);
        }
        return false;
    }
    let mut rng = Rng::new();
    for p in batch.iter_mut() {
        p.result = Some(match &p.op {
            DbOp::Create { url, alias, ttl } => {
                do_create(conn, &mut rng, url, alias.as_deref(), *ttl, p.now)
            }
            DbOp::Stats { code } => do_stats(conn, code),
            DbOp::Jump { code } => do_jump(conn, code, p.now),
        });
    }
    if conn.execute_batch("COMMIT;").is_err() {
        let _ = conn.execute_batch("ROLLBACK;");
        for p in batch.iter_mut() {
            p.result = Some(DbResult::Internal);
        }
        return false;
    }
    true
}

struct ReplySlot {
    state: Mutex<Option<DbResult>>,
    cv: std::sync::Condvar,
}

/// Dedicated writer thread: blocks until work arrives, drains everything
/// pending (natural batching only, no artificial delay), runs it as one
/// transaction, commits, then releases all waiters.
fn db_writer(mut conn: Connection, rx: mpsc::Receiver<Pending>) {
    loop {
        let first = match rx.recv() {
            Ok(p) => p,
            Err(_) => return,
        };
        let mut batch: Vec<Pending> = Vec::with_capacity(64);
        batch.push(first);
        while batch.len() < 256 {
            match rx.try_recv() {
                Ok(p) => batch.push(p),
                Err(_) => break,
            }
        }
        let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_batch(&mut conn, &mut batch)
        }));
        if ok.is_err() {
            // panic recovery: make sure no transaction stays open
            let _ = conn.execute_batch("ROLLBACK;");
            for p in batch.iter_mut() {
                p.result = Some(DbResult::Internal);
            }
        }
        for mut p in batch.into_iter() {
            if p.result.is_none() {
                p.result = Some(DbResult::Internal);
            }
            if let Some(slot) = p.slot.take() {
                let mut g = match slot.state.lock() {
                    Ok(g) => g,
                    Err(e) => e.into_inner(),
                };
                *g = p.result;
                drop(g);
                slot.cv.notify_one();
            }
        }
    }
}

// ---------------- response builders ----------------

fn status_reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        302 => "Found",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        410 => "Gone",
        _ => "Internal Server Error",
    }
}

fn json_response(status: u16, body: &str, close: bool) -> Vec<u8> {
    let mut out = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\n",
        status,
        status_reason(status),
        body.as_bytes().len()
    );
    if close {
        out.push_str("Connection: close\r\n");
    }
    out.push_str("\r\n");
    let mut v = out.into_bytes();
    v.extend_from_slice(body.as_bytes());
    v
}

fn error_response(status: u16, code: &str, close: bool) -> Vec<u8> {
    json_response(status, &json!({ "error": code }).to_string(), close)
}

fn redirect_response(location: &str, close: bool) -> Vec<u8> {
    let mut out = format!(
        "HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\n",
        location
    );
    if close {
        out.push_str("Connection: close\r\n");
    }
    out.push_str("\r\n");
    out.into_bytes()
}

// ---------------- endpoint handlers ----------------

fn media_type_is_json(ct: &str) -> bool {
    match ct.split(';').next() {
        Some(mt) => mt.trim().eq_ignore_ascii_case("application/json"),
        None => false,
    }
}

fn handle_create(app: &App, content_type: Option<&str>, body: &[u8], close: bool) -> Vec<u8> {
    let ct_ok = content_type.map(media_type_is_json).unwrap_or(false);
    if !ct_ok {
        return error_response(400, "invalid_request", close);
    }
    let spec = match parse_create_body(body) {
        Ok(s) => s,
        Err(CreateErr::InvalidRequest) => return error_response(400, "invalid_request", close),
        Err(CreateErr::InvalidUrl) => return error_response(400, "invalid_url", close),
        Err(CreateErr::InvalidAlias) => return error_response(400, "invalid_alias", close),
    };
    match app.submit(DbOp::Create {
        url: spec.url,
        alias: spec.alias,
        ttl: spec.ttl,
    }) {
        DbResult::Created { code, expires_at } => {
            let resp = json!({
                "code": code,
                "short_url": format!("http://127.0.0.1:{}/{}", app.port, code),
                "expires_at": expires_at.map(iso8601),
            });
            json_response(201, &resp.to_string(), close)
        }
        DbResult::AliasTaken => error_response(409, "alias_taken", close),
        _ => error_response(500, "internal", close),
    }
}

enum DbErr {
    Constraint,
    Other,
}

fn exec_insert(
    conn: &Connection,
    code: &str,
    url: &str,
    created_at: i64,
    expires_at: Option<i64>,
) -> Result<(), DbErr> {
    let r = conn
        .prepare_cached(
            "INSERT INTO links (code, url, clicks, created_at, expires_at) VALUES (?1, ?2, 0, ?3, ?4)",
        )
        .and_then(|mut st| {
            st.execute(params![code, url, created_at, expires_at])
        });
    match r {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(e, _))
            if e.code == ErrorCode::ConstraintViolation =>
        {
            Err(DbErr::Constraint)
        }
        Err(_) => Err(DbErr::Other),
    }
}

fn handle_stats(app: &App, code: &str, close: bool) -> Vec<u8> {
    match app.submit(DbOp::Stats { code: code.to_string() }) {
        DbResult::StatsRow {
            url,
            clicks,
            created_at,
            expires_at,
        } => {
            let now = now_secs();
            let expired = expires_at.map(|e| e <= now).unwrap_or(false);
            let resp = json!({
                "code": code,
                "url": url,
                "clicks": clicks,
                "created_at": iso8601(created_at),
                "expires_at": expires_at.map(iso8601),
                "expired": expired,
            });
            json_response(200, &resp.to_string(), close)
        }
        DbResult::Missing => error_response(404, "not_found", close),
        _ => error_response(500, "internal", close),
    }
}

fn handle_jump(app: &App, code: &str, close: bool) -> Vec<u8> {
    match app.submit(DbOp::Jump { code: code.to_string() }) {
        DbResult::Jumped { url } => redirect_response(&encode_location(&url), close),
        DbResult::Expired => error_response(410, "expired", close),
        DbResult::Missing => error_response(404, "not_found", close),
        _ => error_response(500, "internal", close),
    }
}

fn route(app: &App, method: &str, target: &str, content_type: Option<&str>, body: &[u8], close: bool) -> Vec<u8> {
    let path = match target.find('?') {
        Some(i) => &target[..i],
        None => target,
    };
    match (method, path) {
        ("POST", "/api/links") => handle_create(app, content_type, body, close),
        ("GET", p) if p.starts_with("/api/links/") => {
            let code = &p["/api/links/".len()..];
            if code.is_empty() || code.contains('/') {
                error_response(404, "not_found", close)
            } else {
                handle_stats(app, code, close)
            }
        }
        ("GET", p) => {
            let code = p.strip_prefix('/').unwrap_or(p);
            if code.contains('/') {
                error_response(404, "not_found", close)
            } else {
                handle_jump(app, code, close)
            }
        }
        _ => error_response(404, "not_found", close),
    }
}

// ---------------- connection handling ----------------

struct ReqHead {
    end: usize, // bytes consumed by headers
    method: String,
    path: String,
    keep_alive: bool,
    content_type: Option<String>,
    content_length: usize,
    bad: bool, // malformed request -> 400
}

fn read_head(stream: &mut TcpStream, buf: &mut Vec<u8>) -> ReqHead {
    loop {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut preq = httparse::Request::new(&mut headers);
        match preq.parse(buf) {
            Ok(httparse::Status::Complete(end)) => {
                let method = preq.method.unwrap_or("").to_string();
                let path = preq.path.unwrap_or("").to_string();
                let version = preq.version.unwrap_or(1);
                let mut content_length: Option<usize> = None;
                let mut cl_conflict = false;
                let mut conn_close = version == 0;
                let mut conn_keep = false;
                let mut has_te = false;
                let mut content_type: Option<String> = None;
                for h in preq.headers.iter() {
                    let name = h.name.as_bytes();
                    let val = h.value;
                    if name.eq_ignore_ascii_case(b"content-length") {
                        match std::str::from_utf8(val)
                            .ok()
                            .and_then(|s| s.trim().parse::<usize>().ok())
                        {
                            Some(v) => {
                                if content_length.map(|p| p != v).unwrap_or(false) {
                                    cl_conflict = true;
                                }
                                content_length = Some(v);
                            }
                            None => cl_conflict = true,
                        }
                    } else if name.eq_ignore_ascii_case(b"connection") {
                        let s = String::from_utf8_lossy(val).to_ascii_lowercase();
                        if s.contains("close") {
                            conn_close = true;
                        }
                        if s.contains("keep-alive") {
                            conn_keep = true;
                        }
                    } else if name.eq_ignore_ascii_case(b"transfer-encoding") {
                        has_te = true;
                    } else if name.eq_ignore_ascii_case(b"content-type") {
                        content_type = Some(String::from_utf8_lossy(val).into_owned());
                    }
                }
                let mut head = ReqHead {
                    end,
                    method,
                    path,
                    keep_alive: !conn_close && (version == 1 || conn_keep),
                    content_type,
                    content_length: content_length.unwrap_or(0),
                    bad: false,
                };
                if has_te || cl_conflict {
                    head.bad = true;
                }
                return head;
            }
            Ok(httparse::Status::Partial) => {
                if buf.len() > MAX_HEADER_BYTES {
                    return ReqHead {
                        end: 0,
                        method: String::new(),
                        path: String::new(),
                        keep_alive: false,
                        content_type: None,
                        content_length: 0,
                        bad: true,
                    };
                }
                let mut tmp = [0u8; 4096];
                match stream.read(&mut tmp) {
                    Ok(0) => {
                        return ReqHead {
                            end: 0,
                            method: String::new(),
                            path: String::new(),
                            keep_alive: false,
                            content_type: None,
                            content_length: 0,
                            bad: false,
                        }
                    }
                    Ok(k) => buf.extend_from_slice(&tmp[..k]),
                    Err(_) => {
                        return ReqHead {
                            end: 0,
                            method: String::new(),
                            path: String::new(),
                            keep_alive: false,
                            content_type: None,
                            content_length: 0,
                            bad: false,
                        }
                    }
                }
            }
            Err(_) => {
                return ReqHead {
                    end: 0,
                    method: String::new(),
                    path: String::new(),
                    keep_alive: false,
                    content_type: None,
                    content_length: 0,
                    bad: true,
                }
            }
        }
    }
}

fn handle_conn(app: &Arc<App>, mut stream: TcpStream) {
    let _ = stream.set_nodelay(true);
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let mut buf: Vec<u8> = Vec::with_capacity(2048);
    loop {
        let head = read_head(&mut stream, &mut buf);
        if head.bad {
            let _ = stream.write_all(&error_response(400, "invalid_request", true));
            return;
        }
        if head.end == 0 {
            return; // clean EOF / read error before a full request
        }
        if head.content_length > MAX_BODY_BYTES {
            let _ = stream.write_all(&error_response(400, "invalid_request", true));
            // Drain the declared body (bounded) so the peer can finish sending
            // and read the 400 before we close; avoids a RST truncating it.
            let already = buf.len().saturating_sub(head.end);
            let mut remaining = head
                .content_length
                .saturating_sub(already)
                .min(16 * MAX_BODY_BYTES);
            let mut tmp = [0u8; 16384];
            while remaining > 0 {
                let want = remaining.min(tmp.len());
                match stream.read(&mut tmp[..want]) {
                    Ok(0) => break,
                    Ok(k) => remaining -= k,
                    Err(_) => break,
                }
            }
            return;
        }
        // buffer the full body
        let need = head.end + head.content_length;
        while buf.len() < need {
            let mut tmp = [0u8; 8192];
            match stream.read(&mut tmp) {
                Ok(0) => return,
                Ok(k) => buf.extend_from_slice(&tmp[..k]),
                Err(_) => return,
            }
        }
        let close = !head.keep_alive;
        let resp = route(
            app,
            &head.method,
            &head.path,
            head.content_type.as_deref(),
            &buf[head.end..need],
            close,
        );
        if stream.write_all(&resp).is_err() {
            return;
        }
        buf.drain(..need);
        if close {
            return;
        }
    }
}

// ---------------- signals ----------------

static TERMINATED: AtomicBool = AtomicBool::new(false);

extern "C" fn on_signal(_sig: libc::c_int) {
    TERMINATED.store(true, Ordering::SeqCst);
}

fn install_signal_handlers() {
    unsafe {
        libc::signal(libc::SIGTERM, on_signal as libc::sighandler_t);
        libc::signal(libc::SIGINT, on_signal as libc::sighandler_t);
    }
}

// ---------------- startup ----------------

fn open_db(path: &str) -> Connection {
    let conn = Connection::open(path).expect("open sqlite db");
    conn.execute_batch(
        "PRAGMA page_size = 512;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA cache_size = -256;
         PRAGMA wal_autocheckpoint = 8000;
         CREATE TABLE IF NOT EXISTS links (
             code TEXT PRIMARY KEY,
             url TEXT NOT NULL,
             clicks INTEGER NOT NULL DEFAULT 0,
             created_at INTEGER NOT NULL,
             expires_at INTEGER
         ) WITHOUT ROWID;",
    )
    .expect("init schema");
    conn
}

fn main() {
    let port_str = env::var("PORT").unwrap_or_else(|_| {
        eprintln!("shortlink: PORT environment variable is required");
        std::process::exit(1);
    });
    let port: u16 = match port_str.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("shortlink: invalid PORT value {:?}", port_str);
            std::process::exit(1);
        }
    };
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| {
        eprintln!("shortlink: DB_PATH environment variable is required");
        std::process::exit(1);
    });
    if db_path.is_empty() {
        eprintln!("shortlink: DB_PATH must not be empty");
        std::process::exit(1);
    }
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("shortlink: cannot create parent dir for DB_PATH: {}", e);
                std::process::exit(1);
            }
        }
    }
    let db = open_db(&db_path);

    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("shortlink: cannot bind 127.0.0.1:{}: {}", port, e);
            std::process::exit(1);
        }
    };
    let actual_port = listener.local_addr().expect("local addr").port();

    let (db_tx, db_rx) = mpsc::channel::<Pending>();
    thread::Builder::new()
        .stack_size(WORKER_STACK)
        .spawn(move || db_writer(db, db_rx))
        .expect("spawn db writer");
    let app = Arc::new(App {
        tx: db_tx,
        port: actual_port,
    });

    // Startup line first (bind done, port known), flushed immediately.
    println!("listening on http://127.0.0.1:{}", actual_port);
    let _ = std::io::stdout().flush();

    install_signal_handlers();
    thread::spawn(|| loop {
        if TERMINATED.load(Ordering::SeqCst) {
            std::process::exit(0);
        }
        thread::sleep(Duration::from_millis(50));
    });

    let (tx, rx) = mpsc::channel::<TcpStream>();
    let rx = Arc::new(Mutex::new(rx));
    for _ in 0..WORKERS {
        let rx = Arc::clone(&rx);
        let app = Arc::clone(&app);
        if thread::Builder::new()
            .stack_size(WORKER_STACK)
            .spawn(move || loop {
                let stream = {
                    let guard = match rx.lock() {
                        Ok(g) => g,
                        Err(_) => return,
                    };
                    guard.recv()
                };
                match stream {
                    Ok(s) => handle_conn(&app, s),
                    Err(_) => return,
                }
            })
            .is_err()
        {
            eprintln!("shortlink: failed to spawn worker thread");
            std::process::exit(1);
        }
    }

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                if tx.send(s).is_err() {
                    break;
                }
            }
            Err(_) => continue,
        }
    }
}

// ---------------- tests (written BEFORE implementation; see red logs) ----------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_valid_cases() {
        for u in [
            "http://a",
            "https://example.com/",
            "https://example.com/path with space",
            "http://127.0.0.1:9/x",
            "https://example.com/a%2Fb?x=%20&y=1",
            "https://example.com/a%2fb",
            "https://example.com:8443/x?y=1#z",
        ] {
            assert!(validate_url(u), "expected valid: {:?}", u);
        }
        let base = "https://example.com/";
        let u2048 = format!("{}{}", base, "a".repeat(2048 - base.len()));
        assert_eq!(u2048.as_bytes().len(), 2048);
        assert!(validate_url(&u2048));
    }

    #[test]
    fn url_invalid_cases() {
        for u in [
            "",
            "example.com",
            "http://",
            "https://",
            "http:///path",
            "ftp://example.com",
            "http://exa mple.com/path",
            "https://exa mple.com",
            "http://:80/path",
            "http://example.com/%2",
            "http://example.com/%zz",
            "http://example.com/100%",
            "http://example.com/\u{0}",
            "http://example.com/\u{1}x",
            "http://example.com/\u{1f}",
            "http://example.com/\u{7f}",
            "http://exam\nple.com/",
            "http://example.com/a%2G1",
        ] {
            assert!(!validate_url(u), "expected invalid: {:?}", u);
        }
        let base = "https://example.com/";
        let u2049 = format!("{}{}", base, "a".repeat(2049 - base.len()));
        assert!(!validate_url(&u2049));
        // 3-byte UTF-8 char counting: must count bytes not chars
        // prefix "https://example.com/" is 20 bytes
        let wide2048 = format!("https://example.com/{}", "中".repeat(676)); // 20 + 2028 = 2048
        assert_eq!(wide2048.as_bytes().len(), 2048);
        assert!(validate_url(&wide2048));
        let wide2051 = format!("https://example.com/{}", "中".repeat(677)); // 20 + 2031 = 2051
        assert_eq!(wide2051.as_bytes().len(), 2051);
        assert!(!validate_url(&wide2051));
    }

    #[test]
    fn alias_rules() {
        for a in ["Abc1", "abc1", "aBc_", "a-b9", "a000", &"A".repeat(32), &"z".repeat(4)] {
            assert!(valid_alias(a), "expected valid: {:?}", a);
        }
        for a in [
            "abc",
            &"a".repeat(33),
            "ab c1",
            "abc!",
            "abc/1",
            "中文ab",
            "api",
            "health",
            "",
        ] {
            assert!(!valid_alias(a), "expected invalid: {:?}", a);
        }
    }

    #[test]
    fn location_encoding() {
        let cases = [
            ("https://example.com/a%2Fb?x=%20&y=1", "https://example.com/a%2Fb?x=%20&y=1"),
            ("https://example.com/a%2fb", "https://example.com/a%2fb"),
            (
                "https://example.com/路径?x=中 文#片段",
                "https://example.com/%E8%B7%AF%E5%BE%84?x=%E4%B8%AD%20%E6%96%87#%E7%89%87%E6%AE%B5",
            ),
            ("https://example.com/path with space", "https://example.com/path%20with%20space"),
            ("http://127.0.0.1:9/x", "http://127.0.0.1:9/x"),
            ("https://example.com/a?b=c&d=e#f", "https://example.com/a?b=c&d=e#f"),
            ("https://example.com/x?a=1&b=2", "https://example.com/x?a=1&b=2"),
            (
                "https://example.com/~user!$&'()*+,;=:@[]",
                "https://example.com/~user!$&'()*+,;=:@[]",
            ),
        ];
        for (src, want) in cases {
            assert_eq!(encode_location(src), want, "src={:?}", src);
        }
    }

    #[test]
    fn iso_format() {
        assert_eq!(iso8601(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601(946684800), "2000-01-01T00:00:00Z");
        assert_eq!(iso8601(1234567890), "2009-02-13T23:31:30Z");
        assert_eq!(iso8601(2000000000), "2033-05-18T03:33:20Z");
        assert_eq!(iso8601(951782400), "2000-02-29T00:00:00Z"); // leap day
    }

    fn url_err(r: Result<CreateSpec, CreateErr>) -> bool {
        matches!(r, Err(CreateErr::InvalidUrl))
    }
    fn req_err(r: Result<CreateSpec, CreateErr>) -> bool {
        matches!(r, Err(CreateErr::InvalidRequest))
    }
    fn alias_err(r: Result<CreateSpec, CreateErr>) -> bool {
        matches!(r, Err(CreateErr::InvalidAlias))
    }

    #[test]
    fn create_body_ok() {
        let s = parse_create_body(br#"{"url":"https://example.com"}"#).unwrap();
        assert_eq!(s.url, "https://example.com");
        assert!(s.alias.is_none());
        assert!(s.ttl.is_none());

        let s = parse_create_body(br#"{"url":"https://example.com","alias":null,"ttl_seconds":null}"#)
            .unwrap();
        assert!(s.alias.is_none());
        assert!(s.ttl.is_none());

        let s = parse_create_body(br#"{"url":"https://e.com","alias":"Abc1","ttl_seconds":31536000}"#)
            .unwrap();
        assert_eq!(s.alias.as_deref(), Some("Abc1"));
        assert_eq!(s.ttl, Some(31536000));

        let s = parse_create_body(br#"{"ttl_seconds":1,"url":"https://e.com","unknown":{"a":[1,true,null,"x"]}}"#)
            .unwrap();
        assert_eq!(s.ttl, Some(1));
    }

    #[test]
    fn create_body_invalid_request() {
        let bodies: [&[u8]; 27] = [
            b"",
            b"{",
            b"[]",
            b"null",
            b"\"x\"",
            b"{}{}",
            b"{\"url\":123}",
            b"{\"url\":null}",
            b"{\"url\":true}",
            b"{}",
            b"{\"alias\":\"abcde\"}",
            b"{\"url\":\"https://example.com\",\"alias\":123}",
            b"{\"url\":\"https://example.com\",\"alias\":true}",
            b"{\"url\":\"https://example.com\",\"alias\":[\"a\"]}",
            b"{\"url\":\"https://example.com\",\"ttl_seconds\":true}",
            b"{\"url\":\"https://example.com\",\"ttl_seconds\":false}",
            b"{\"url\":\"https://example.com\",\"ttl_seconds\":1.5}",
            b"{\"url\":\"https://example.com\",\"ttl_seconds\":1.0}",
            b"{\"url\":\"https://example.com\",\"ttl_seconds\":\"5\"}",
            b"{\"url\":\"https://example.com\",\"ttl_seconds\":0}",
            b"{\"url\":\"https://example.com\",\"ttl_seconds\":-1}",
            b"{\"url\":\"https://example.com\",\"ttl_seconds\":31536001}",
            b"{\"url\":\"https://example.com\",\"ttl_seconds\":999999999999999999999}",
            b"{\"url\":\"https://example.com\",}",
            b"{\"url\":\"https://example.com\"",
            b"{'url':'https://example.com'}",
            b"{\"url\":\"https://example.com/\\q\"}",
        ];
        for b in bodies {
            assert!(req_err(parse_create_body(b)), "expected invalid_request: {:?}", b);
        }
        assert!(req_err(parse_create_body(b"{\"url\":\xff}")));
    }

    #[test]
    fn create_body_invalid_url_and_alias() {
        assert!(url_err(parse_create_body(br#"{"url":"ftp://example.com"}"#)));
        assert!(url_err(parse_create_body(br#"{"url":"http://%2"}"#)));
        assert!(alias_err(parse_create_body(
            br#"{"url":"https://example.com","alias":"abc"}"#
        )));
        assert!(alias_err(parse_create_body(
            br#"{"url":"https://example.com","alias":"api"}"#
        )));
        assert!(alias_err(parse_create_body(
            br#"{"url":"https://example.com","alias":"health"}"#
        )));
        assert!(alias_err(parse_create_body(
            br#"{"url":"https://example.com","alias":"abc!"}"#
        )));
    }

    // ------- round 2: group-commit batch semantics (written first; red) -------

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE links (
                code TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                clicks INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                expires_at INTEGER
            ) WITHOUT ROWID;",
        )
        .unwrap();
        conn
    }

    fn pending_ops(ops: Vec<(DbOp, i64)>) -> Vec<Pending> {
        ops.into_iter()
            .map(|(op, now)| Pending { op, now, slot: None, result: None })
            .collect()
    }

    fn stats_of(conn: &mut Connection, code: &str) -> DbResult {
        let mut b = pending_ops(vec![(DbOp::Stats { code: code.into() }, 1_000_000)]);
        run_batch(conn, &mut b);
        b[0].result.take().unwrap()
    }

    #[test]
    fn batch_alias_race_one_winner() {
        let mut conn = mem_conn();
        let mut batch = pending_ops((0..20).map(|i| {
            (
                DbOp::Create {
                    url: format!("https://example.com/c/{i}"),
                    alias: Some("conc1".into()),
                    ttl: None,
                },
                1000,
            )
        }).collect::<Vec<_>>());
        assert!(run_batch(&mut conn, &mut batch));
        let n_created = batch
            .iter()
            .filter(|p| matches!(p.result, Some(DbResult::Created { .. })))
            .count();
        let n_taken = batch
            .iter()
            .filter(|p| matches!(p.result, Some(DbResult::AliasTaken)))
            .count();
        assert_eq!((n_created, n_taken), (1, 19));
        match stats_of(&mut conn, "conc1") {
            DbResult::StatsRow { clicks, .. } => assert_eq!(clicks, 0),
            other => panic!("expected StatsRow, got {:?}", other),
        }
    }

    #[test]
    fn batch_jump_counts_exact_and_missing() {
        let mut conn = mem_conn();
        let mut seed = pending_ops(vec![(
            DbOp::Create {
                url: "https://example.com/j".into(),
                alias: Some("jjjj1".into()),
                ttl: None,
            },
            1000,
        )]);
        assert!(run_batch(&mut conn, &mut seed));
        let mut batch = pending_ops((0..50).map(|_| (DbOp::Jump { code: "jjjj1".into() }, 1001)).collect::<Vec<_>>());
        assert!(run_batch(&mut conn, &mut batch));
        assert!(batch.iter().all(|p| matches!(p.result, Some(DbResult::Jumped { .. }))));
        match stats_of(&mut conn, "jjjj1") {
            DbResult::StatsRow { clicks, .. } => assert_eq!(clicks, 50),
            other => panic!("expected StatsRow, got {:?}", other),
        }
        // unknown code -> Missing
        let mut miss = pending_ops(vec![(DbOp::Jump { code: "nope1".into() }, 1001)]);
        assert!(run_batch(&mut conn, &mut miss));
        assert!(matches!(miss[0].result, Some(DbResult::Missing)));
    }

    #[test]
    fn batch_expiry_boundary() {
        let mut conn = mem_conn();
        // ttl=1 created at t=1000 -> expires_at=1001; "exactly expired" at now>=1001
        let mut seed = pending_ops(vec![(
            DbOp::Create {
                url: "https://example.com/t".into(),
                alias: Some("tttl1".into()),
                ttl: Some(1),
            },
            1000,
        )]);
        assert!(run_batch(&mut conn, &mut seed));
        match seed[0].result.take().unwrap() {
            DbResult::Created { expires_at, .. } => assert_eq!(expires_at, Some(1001)),
            other => panic!("expected Created, got {:?}", other),
        }
        let mut before = pending_ops(vec![(DbOp::Jump { code: "tttl1".into() }, 1000)]);
        assert!(run_batch(&mut conn, &mut before));
        assert!(matches!(before[0].result, Some(DbResult::Jumped { .. })));
        let mut at_boundary = pending_ops(vec![(DbOp::Jump { code: "tttl1".into() }, 1001)]);
        assert!(run_batch(&mut conn, &mut at_boundary));
        assert!(matches!(at_boundary[0].result, Some(DbResult::Expired)));
        // clicks frozen at 1 after expiry
        match stats_of(&mut conn, "tttl1") {
            DbResult::StatsRow { clicks, .. } => assert_eq!(clicks, 1),
            other => panic!("expected StatsRow, got {:?}", other),
        }
    }

    #[test]
    fn batch_auto_code_unique_per_batch() {
        let mut conn = mem_conn();
        let mut batch = pending_ops((0..100).map(|i| {
            (
                DbOp::Create {
                    url: format!("https://example.com/u/{i}"),
                    alias: None,
                    ttl: None,
                },
                1000,
            )
        }).collect::<Vec<_>>());
        assert!(run_batch(&mut conn, &mut batch));
        assert!(batch
            .iter()
            .all(|p| matches!(&p.result, Some(DbResult::Created { code, .. }) if code.len() == 7)));
        let mut codes: Vec<String> = batch
            .iter()
            .map(|p| match &p.result {
                Some(DbResult::Created { code, .. }) => code.clone(),
                _ => String::new(),
            })
            .collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), 100, "auto codes must be unique within a batch");
    }
}
