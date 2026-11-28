//! Sol Next Public Edition (`ai.sol.next`) — Zero-Dependency Rust 2024 CLI & Stdio MCP Core.
//!
//! Provides deterministic `--json` ADB automation, FIPS 180-4 SHA-256 APK verification,
//! 1:1 Background VirtualDisplay alignment inspection, and an in-process Model Context
//! Protocol (`2024-11-05`) JSON-RPC 2.0 server with zero external crate dependencies and
//! zero Python runtime requirements.

use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const PACKAGE_ID: &str = "ai.sol.next";
pub const MAIN_ACTIVITY: &str = "ai.sol.next/ai.sol.next.canvas.SolCanvasActivity";
pub const RELEASE_TAG: &str = "v9.3.1-public";
pub const SERVER_VERSION: &str = "9.3.1-public";
pub const PROTOCOL_VERSION: &str = "2024-11-05";

pub struct ArtifactSpec {
    pub abi: &'static str,
    pub filename: &'static str,
    pub github_url: &'static str,
    pub china_url: &'static str,
    pub ghfast_url: &'static str,
    pub sha256: &'static str,
}

pub const ARTIFACTS: &[ArtifactSpec] = &[
    ArtifactSpec {
        abi: "arm64",
        filename: "Sol-Next-v9.3.1-public-arm64.apk",
        github_url: "https://github.com/chongyangzhang92-hub/Sol-Next/releases/download/v9.3.1-public/Sol-Next-v9.3.1-public-arm64.apk",
        china_url: "https://webui.67-230-181-51.sslip.io/assets/sol-public/v9.3.1/Sol-Next-v9.3.1-public-arm64.apk",
        ghfast_url: "https://ghfast.top/https://github.com/chongyangzhang92-hub/Sol-Next/releases/download/v9.3.1-public/Sol-Next-v9.3.1-public-arm64.apk",
        sha256: "e6a2c8d69909275dd2b255dcdb68fd75107bd471986af52404c9215879e2283c",
    },
    ArtifactSpec {
        abi: "universal",
        filename: "Sol-Next-v9.3.1-public-universal.apk",
        github_url: "https://github.com/chongyangzhang92-hub/Sol-Next/releases/download/v9.3.1-public/Sol-Next-v9.3.1-public-universal.apk",
        china_url: "https://webui.67-230-181-51.sslip.io/assets/sol-public/v9.3.1/Sol-Next-v9.3.1-public-universal.apk",
        ghfast_url: "https://ghfast.top/https://github.com/chongyangzhang92-hub/Sol-Next/releases/download/v9.3.1-public/Sol-Next-v9.3.1-public-universal.apk",
        sha256: "32ed3ab9ade612ed2559c7439b77b096c864cd1a2a52debf6eaa7faf6187bd0a",
    },
];

pub const RUNTIME_PERMISSIONS: &[&str] = &[
    "android.permission.POST_NOTIFICATIONS",
    "android.permission.RECORD_AUDIO",
    "android.permission.CALL_PHONE",
    "android.permission.READ_PHONE_STATE",
    "android.permission.ANSWER_PHONE_CALLS",
    "android.permission.READ_CONTACTS",
    "android.permission.READ_CALENDAR",
    "android.permission.WRITE_CALENDAR",
    "android.permission.CAMERA",
    "android.permission.ACCESS_FINE_LOCATION",
    "android.permission.ACCESS_COARSE_LOCATION",
    "android.permission.BLUETOOTH_CONNECT",
];

// ============================================================================
// Pure Zero-Dependency RFC 8259 JSON Value, Parser & Serializer
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    pub fn obj(entries: Vec<(&str, JsonValue)>) -> Self {
        JsonValue::Object(entries.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn to_compact_string(&self) -> String {
        let mut out = String::new();
        self.write_json(&mut out, false, 0);
        out
    }

    pub fn to_pretty_string(&self) -> String {
        let mut out = String::new();
        self.write_json(&mut out, true, 0);
        out
    }

    fn write_json(&self, out: &mut String, pretty: bool, depth: usize) {
        match self {
            JsonValue::Null => out.push_str("null"),
            JsonValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            JsonValue::Int(n) => out.push_str(&n.to_string()),
            JsonValue::Str(s) => write_escaped_str(out, s),
            JsonValue::Array(items) => {
                if items.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    if pretty {
                        out.push('\n');
                        push_indent(out, depth + 1);
                    }
                    item.write_json(out, pretty, depth + 1);
                }
                if pretty {
                    out.push('\n');
                    push_indent(out, depth);
                }
                out.push(']');
            }
            JsonValue::Object(entries) => {
                if entries.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push('{');
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    if pretty {
                        out.push('\n');
                        push_indent(out, depth + 1);
                    }
                    write_escaped_str(out, k);
                    out.push_str(if pretty { ": " } else { ":" });
                    v.write_json(out, pretty, depth + 1);
                }
                if pretty {
                    out.push('\n');
                    push_indent(out, depth);
                }
                out.push('}');
            }
        }
    }
}

fn push_indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn write_escaped_str(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser {
        bytes: input.as_bytes(),
        pos: 0,
    };
    let val = parser.parse_value()?;
    parser.skip_ws();
    if parser.pos != parser.bytes.len() {
        return Err(format!("trailing characters at byte {}", parser.pos));
    }
    Ok(val)
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\r' | b'\n' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'n') => self.parse_literal(b"null", JsonValue::Null),
            Some(b't') => self.parse_literal(b"true", JsonValue::Bool(true)),
            Some(b'f') => self.parse_literal(b"false", JsonValue::Bool(false)),
            Some(b'"') => self.parse_string().map(JsonValue::Str),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(b) => Err(format!("unexpected byte '{}' at {}", b as char, self.pos)),
            None => Err("unexpected end of JSON input".to_string()),
        }
    }

    fn parse_literal(&mut self, lit: &[u8], val: JsonValue) -> Result<JsonValue, String> {
        if self.bytes[self.pos..].starts_with(lit) {
            self.pos += lit.len();
            Ok(val)
        } else {
            Err(format!("invalid literal at byte {}", self.pos))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while let Some(b'0'..=b'9') = self.peek() {
            self.pos += 1;
        }
        let mut is_float = false;
        if self.peek() == Some(b'.') {
            is_float = true;
            self.pos += 1;
            while let Some(b'0'..=b'9') = self.peek() {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            while let Some(b'0'..=b'9') = self.peek() {
                self.pos += 1;
            }
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| "invalid UTF-8 in number".to_string())?;
        if is_float {
            let f: f64 = s.parse().map_err(|e| format!("invalid number '{s}': {e}"))?;
            Ok(JsonValue::Int(f.round() as i64))
        } else {
            let n: i64 = s.parse().map_err(|e| format!("invalid integer '{s}': {e}"))?;
            Ok(JsonValue::Int(n))
        }
    }

    fn parse_hex4(&mut self) -> Result<u16, String> {
        if self.pos + 4 > self.bytes.len() {
            return Err("truncated \\uXXXX escape".to_string());
        }
        let hex = std::str::from_utf8(&self.bytes[self.pos..self.pos + 4])
            .map_err(|_| "invalid hex escape".to_string())?;
        self.pos += 4;
        u16::from_str_radix(hex, 16).map_err(|_| format!("invalid hex '{hex}'"))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        if self.peek() != Some(b'"') {
            return Err(format!("expected '\"' at byte {}", self.pos));
        }
        self.pos += 1;
        let mut out = String::new();
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'"' {
                self.pos += 1;
                return Ok(out);
            }
            if b == b'\\' {
                self.pos += 1;
                let esc = self.peek().ok_or_else(|| "unterminated escape".to_string())?;
                self.pos += 1;
                match esc {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{08}'),
                    b'f' => out.push('\u{0C}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => {
                        let code = self.parse_hex4()?;
                        if (0xD800..=0xDBFF).contains(&code) {
                            if self.bytes[self.pos..].starts_with(b"\\u") {
                                self.pos += 2;
                                let low = self.parse_hex4()?;
                                if (0xDC00..=0xDFFF).contains(&low) {
                                    let combined = 0x10000
                                        + (((code as u32 - 0xD800) << 10) | (low as u32 - 0xDC00));
                                    if let Some(ch) = char::from_u32(combined) {
                                        out.push(ch);
                                    }
                                }
                            }
                        } else if let Some(ch) = char::from_u32(code as u32) {
                            out.push(ch);
                        }
                    }
                    other => return Err(format!("invalid escape '\\{}'", other as char)),
                }
            } else {
                let tail = std::str::from_utf8(&self.bytes[self.pos..])
                    .map_err(|_| "invalid UTF-8 in string".to_string())?;
                let ch = tail.chars().next().ok_or_else(|| "empty UTF-8 char".to_string())?;
                out.push(ch);
                self.pos += ch.len_utf8();
            }
        }
        Err("unterminated JSON string".to_string())
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => return Err(format!("expected ',' or ']' at byte {}", self.pos)),
            }
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1;
        let mut entries = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(format!("expected ':' at byte {}", self.pos));
            }
            self.pos += 1;
            let val = self.parse_value()?;
            entries.push((key, val));
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(entries));
                }
                _ => return Err(format!("expected ',' or '}}' at byte {}", self.pos)),
            }
        }
    }
}

// ============================================================================
// Pure Zero-Dependency FIPS 180-4 SHA-256
// ============================================================================

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub struct Sha256 {
    state: [u32; 8],
    buf: [u8; 64],
    buf_len: usize,
    total_len: u64,
}

impl Sha256 {
    pub fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],
            buf: [0u8; 64],
            buf_len: 0,
            total_len: 0,
        }
    }

    pub fn update(&mut self, mut data: &[u8]) {
        self.total_len += data.len() as u64;
        if self.buf_len > 0 {
            let take = (64 - self.buf_len).min(data.len());
            self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];
            if self.buf_len == 64 {
                let block = self.buf;
                self.compress(&block);
                self.buf_len = 0;
            }
        }
        while data.len() >= 64 {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[..64]);
            self.compress(&block);
            data = &data[64..];
        }
        if !data.is_empty() {
            self.buf[..data.len()].copy_from_slice(data);
            self.buf_len = data.len();
        }
    }

    pub fn finalize_hex(mut self) -> String {
        let bit_len = self.total_len.wrapping_mul(8);
        self.update(&[0x80]);
        if self.buf_len > 56 {
            let pad = [0u8; 64];
            self.update(&pad[..64 - self.buf_len]);
        }
        if self.buf_len < 56 {
            let pad = [0u8; 56];
            self.update(&pad[..56 - self.buf_len]);
        }
        self.update(&bit_len.to_be_bytes());
        let mut hex = String::with_capacity(64);
        for word in self.state {
            hex.push_str(&format!("{word:08x}"));
        }
        hex
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            let j = i * 4;
            w[i] = u32::from_be_bytes([block[j], block[j + 1], block[j + 2], block[j + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize_hex()
}

pub fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 256 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize_hex())
}

// ============================================================================
// ADB Discovery, Display Spec Parser & Deterministic Command Handlers
// ============================================================================

#[derive(Debug, Clone)]
pub struct AdbDevice {
    pub serial: String,
    pub state: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplaySpec {
    pub width: i64,
    pub height: i64,
    pub density_dpi: i64,
    pub scale_one: bool,
}

impl DisplaySpec {
    fn to_json(&self) -> JsonValue {
        JsonValue::obj(vec![
            ("width", JsonValue::Int(self.width)),
            ("height", JsonValue::Int(self.height)),
            ("densityDpi", JsonValue::Int(self.density_dpi)),
            ("scaleOne", JsonValue::Bool(self.scale_one)),
        ])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayAlignment {
    pub display0: Option<DisplaySpec>,
    pub virtual_display: Option<DisplaySpec>,
    pub exact_1to1: bool,
}

impl DisplayAlignment {
    pub fn to_json(&self) -> JsonValue {
        JsonValue::obj(vec![
            (
                "display0",
                self.display0.as_ref().map(DisplaySpec::to_json).unwrap_or(JsonValue::Null),
            ),
            (
                "virtualDisplay",
                self.virtual_display
                    .as_ref()
                    .map(DisplaySpec::to_json)
                    .unwrap_or(JsonValue::Null),
            ),
            ("exact1to1", JsonValue::Bool(self.exact_1to1)),
        ])
    }
}

pub fn find_adb() -> Option<PathBuf> {
    if let Ok(path_var) = env::var("PATH") {
        let exe_name = if cfg!(windows) { "adb.exe" } else { "adb" };
        for dir in env::split_paths(&path_var) {
            let cand = dir.join(exe_name);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    let home = env::var("HOME").unwrap_or_default();
    let candidates = [
        PathBuf::from("/opt/homebrew/share/android-commandlinetools/platform-tools/adb"),
        PathBuf::from(format!("{home}/Library/Android/sdk/platform-tools/adb")),
        PathBuf::from(format!("{home}/Android/Sdk/platform-tools/adb")),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

pub fn run_adb(args: &[&str], serial: Option<&str>) -> Result<(i32, String, String), String> {
    let adb = find_adb().ok_or_else(|| {
        "adb binary not found in PATH or standard Android SDK locations".to_string()
    })?;
    let mut cmd = Command::new(adb);
    if let Some(s) = serial {
        if !s.is_empty() {
            cmd.arg("-s").arg(s);
        }
    }
    cmd.args(args);
    let out = cmd.output().map_err(|e| format!("failed to execute adb: {e}"))?;
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    Ok((code, stdout, stderr))
}

pub fn list_devices() -> Result<Vec<AdbDevice>, String> {
    let (_, stdout, _) = run_adb(&["devices", "-l"], None)?;
    let mut devices = Vec::new();
    for line in stdout.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            devices.push(AdbDevice {
                serial: parts[0].to_string(),
                state: parts[1].to_string(),
                detail: parts[2..].join(" "),
            });
        }
    }
    Ok(devices)
}

pub fn resolve_serial(explicit: Option<&str>) -> Result<String, String> {
    if let Some(s) = explicit {
        let clean = s.trim();
        if !clean.is_empty() {
            return Ok(clean.to_string());
        }
    }
    if let Ok(env_serial) = env::var("SOL_ADB_SERIAL") {
        let clean = env_serial.trim();
        if !clean.is_empty() {
            return Ok(clean.to_string());
        }
    }
    let online: Vec<String> = list_devices()?
        .into_iter()
        .filter(|d| d.state == "device")
        .map(|d| d.serial)
        .collect();
    match online.len() {
        0 => Err("No online ADB Android device found. Connect via USB or `adb connect <ip>:<port>`.".to_string()),
        1 => Ok(online[0].clone()),
        _ => Err(format!(
            "Multiple ADB devices online ({}). Pass --serial <id>.",
            online.join(", ")
        )),
    }
}

pub fn parse_display_specs(dumpsys_text: &str) -> DisplayAlignment {
    let mut display0: Option<DisplaySpec> = None;
    let mut virtual_display: Option<DisplaySpec> = None;

    for line in dumpsys_text.lines() {
        if !line.contains("DisplayDeviceInfo{") {
            continue;
        }
        let Some((width, height)) = extract_wh(line) else {
            continue;
        };
        let Some(density_dpi) = extract_density(line) else {
            continue;
        };
        let spec = DisplaySpec {
            width,
            height,
            density_dpi,
            scale_one: line.contains("scale=1.0"),
        };
        if line.contains("\"Built-in Screen\"") && display0.is_none() {
            display0 = Some(spec);
        } else if line.contains("\"sol-next-bg\"") {
            virtual_display = Some(spec);
        }
    }

    let exact_1to1 = match (&display0, &virtual_display) {
        (Some(d0), Some(vd)) => {
            d0.width == vd.width && d0.height == vd.height && d0.density_dpi == vd.density_dpi
        }
        _ => false,
    };
    DisplayAlignment {
        display0,
        virtual_display,
        exact_1to1,
    }
}

fn extract_wh(line: &str) -> Option<(i64, i64)> {
    let tokens: Vec<&str> = line.split(|c: char| !c.is_ascii_alphanumeric()).collect();
    for win in tokens.windows(3) {
        if win[1] == "x" {
            if let (Ok(w), Ok(h)) = (win[0].parse::<i64>(), win[2].parse::<i64>()) {
                if w >= 240 && h >= 240 {
                    return Some((w, h));
                }
            }
        }
    }
    None
}

fn extract_density(line: &str) -> Option<i64> {
    let idx = line.find("density ")?;
    let rest = &line[idx + "density ".len()..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse::<i64>().ok()
}

fn extract_kv_token(text: &str, prefix: &str) -> Option<String> {
    let idx = text.find(prefix)?;
    let rest = &text[idx + prefix.len()..];
    let val: String = rest.chars().take_while(|c| !c.is_whitespace()).collect();
    if val.is_empty() { None } else { Some(val) }
}

pub fn cmd_status(serial: Option<&str>) -> JsonValue {
    let Some(adb_path) = find_adb() else {
        return JsonValue::obj(vec![
            ("ok", JsonValue::Bool(false)),
            ("error", JsonValue::Str("adb not found".to_string())),
            ("packageId", JsonValue::Str(PACKAGE_ID.to_string())),
        ]);
    };
    let devices = match list_devices() {
        Ok(d) => d,
        Err(e) => {
            return JsonValue::obj(vec![
                ("ok", JsonValue::Bool(false)),
                ("error", JsonValue::Str(e)),
                ("packageId", JsonValue::Str(PACKAGE_ID.to_string())),
            ]);
        }
    };
    let online: Vec<&AdbDevice> = devices.iter().filter(|d| d.state == "device").collect();
    let target = serial
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| env::var("SOL_ADB_SERIAL").ok().filter(|s| !s.trim().is_empty()))
        .or_else(|| (online.len() == 1).then(|| online[0].serial.clone()));

    let device_vals: Vec<JsonValue> = devices
        .iter()
        .map(|d| {
            JsonValue::obj(vec![
                ("serial", JsonValue::Str(d.serial.clone())),
                ("state", JsonValue::Str(d.state.clone())),
                ("detail", JsonValue::Str(d.detail.clone())),
            ])
        })
        .collect();

    let mut entries = vec![
        ("ok", JsonValue::Bool(true)),
        ("adb", JsonValue::Str(adb_path.display().to_string())),
        ("packageId", JsonValue::Str(PACKAGE_ID.to_string())),
        ("releaseTag", JsonValue::Str(RELEASE_TAG.to_string())),
        ("devices", JsonValue::Array(device_vals)),
    ];

    if let Some(target_serial) = target {
        let pkg_stdout = run_adb(&["shell", "dumpsys", "package", PACKAGE_ID], Some(&target_serial))
            .map(|(_, out, _)| out)
            .unwrap_or_default();
        let installed = pkg_stdout.contains("versionName=");
        let version_name = extract_kv_token(&pkg_stdout, "versionName=");
        let version_code = extract_kv_token(&pkg_stdout, "versionCode=")
            .and_then(|v| v.parse::<i64>().ok());
        let disp_stdout = run_adb(&["shell", "dumpsys", "display"], Some(&target_serial))
            .map(|(_, out, _)| out)
            .unwrap_or_default();
        let displays = parse_display_specs(&disp_stdout);

        entries.push(("selectedSerial", JsonValue::Str(target_serial)));
        entries.push(("installed", JsonValue::Bool(installed)));
        entries.push((
            "versionName",
            version_name.map(JsonValue::Str).unwrap_or(JsonValue::Null),
        ));
        entries.push((
            "versionCode",
            version_code.map(JsonValue::Int).unwrap_or(JsonValue::Null),
        ));
        entries.push(("displays", displays.to_json()));
    }

    JsonValue::obj(entries)
}

pub struct InstallOptions {
    pub serial: Option<String>,
    pub abi: String,
    pub mirror: String,
    pub apk_path: Option<String>,
    pub verify_sha256: bool,
    pub grant_permissions: bool,
}

pub fn cmd_install(opts: &InstallOptions) -> Result<JsonValue, String> {
    let serial = resolve_serial(opts.serial.as_deref())?;
    let abi_name = if opts.abi.trim().is_empty() {
        env::var("SOL_APK_ABI").unwrap_or_else(|_| "arm64".to_string())
    } else {
        opts.abi.clone()
    };
    let spec = ARTIFACTS
        .iter()
        .find(|a| a.abi == abi_name)
        .ok_or_else(|| format!("Unsupported ABI '{abi_name}'. Choose from: arm64, universal"))?;

    let (apk_path, downloaded) = if let Some(ref local) = opts.apk_path {
        let path = PathBuf::from(local);
        if !path.is_file() {
            return Err(format!("APK file not found: {}", path.display()));
        }
        (path, false)
    } else {
        let cache_dir = env::temp_dir().join("sol-next-public-cache");
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("failed to create cache dir: {e}"))?;
        let target = cache_dir.join(spec.filename);
        let need_download = !target.is_file()
            || sha256_file(&target).map(|s| s != spec.sha256).unwrap_or(true);
        if need_download {
            let dl_url = match opts.mirror.as_str() {
                "china" => spec.china_url,
                "ghfast" => spec.ghfast_url,
                _ => spec.github_url,
            };
            let status = Command::new("curl")
                .args(["-fsSL", "--retry", "3", "-o"])
                .arg(&target)
                .arg(dl_url)
                .status()
                .map_err(|e| format!("failed to invoke curl: {e}"))?;
            if !status.success() {
                return Err(format!("download failed from {dl_url} (exit {:?})", status.code()));
            }
            (target, true)
        } else {
            (target, false)
        }
    };

    let actual_sha = sha256_file(&apk_path)
        .map_err(|e| format!("failed to hash {}: {e}", apk_path.display()))?;
    if opts.verify_sha256 && opts.apk_path.is_none() && actual_sha != spec.sha256 {
        return Err(format!(
            "SHA-256 mismatch! expected={} actual={actual_sha}",
            spec.sha256
        ));
    }

    let apk_str = apk_path.to_string_lossy().to_string();
    let (code, stdout, stderr) = run_adb(&["install", "-r", "-d", &apk_str], Some(&serial))?;
    let combined = format!("{stdout}{stderr}");
    if code != 0 || !combined.contains("Success") {
        return Err(format!("adb install failed: {}", combined.trim()));
    }

    let mut granted_count = 0i64;
    if opts.grant_permissions {
        for perm in RUNTIME_PERMISSIONS {
            if let Ok((0, _, _)) = run_adb(&["shell", "pm", "grant", PACKAGE_ID, perm], Some(&serial)) {
                granted_count += 1;
            }
        }
    }

    Ok(JsonValue::obj(vec![
        ("ok", JsonValue::Bool(true)),
        ("serial", JsonValue::Str(serial)),
        ("packageId", JsonValue::Str(PACKAGE_ID.to_string())),
        ("abi", JsonValue::Str(spec.abi.to_string())),
        ("apkPath", JsonValue::Str(apk_str)),
        ("downloaded", JsonValue::Bool(downloaded)),
        ("sha256", JsonValue::Str(actual_sha.clone())),
        (
            "sha256Verified",
            if opts.apk_path.is_none() {
                JsonValue::Bool(actual_sha == spec.sha256)
            } else {
                JsonValue::Null
            },
        ),
        ("permissionsGrantedCount", JsonValue::Int(granted_count)),
        (
            "nextStep",
            JsonValue::Str("Open Sol Next v9.3.1 -> works immediately with free built-in Rust-encrypted DeepSeek V4.1 Flash, or override in Settings with your own BYOK API Key.".to_string()),
        ),
    ]))
}

pub fn cmd_launch(
    serial: Option<&str>,
    tab: Option<&str>,
    open_calls: bool,
    open_virtual_phone: bool,
) -> Result<JsonValue, String> {
    let target = resolve_serial(serial)?;
    let mut args = vec!["shell", "am", "start", "-n", MAIN_ACTIVITY];
    if let Some(t) = tab {
        args.push("--es");
        args.push("sol_next_open_tab");
        args.push(t);
    }
    if open_calls {
        args.push("--ez");
        args.push("sol_next_open_calls");
        args.push("true");
    }
    if open_virtual_phone {
        args.push("--ez");
        args.push("sol_next_open_virtual_phone");
        args.push("true");
    }
    let (code, stdout, stderr) = run_adb(&args, Some(&target))?;
    Ok(JsonValue::obj(vec![
        ("ok", JsonValue::Bool(code == 0)),
        ("serial", JsonValue::Str(target)),
        ("packageId", JsonValue::Str(PACKAGE_ID.to_string())),
        ("output", JsonValue::Str(format!("{stdout}{stderr}").trim().to_string())),
    ]))
}

pub fn cmd_start_helper(serial: Option<&str>) -> Result<JsonValue, String> {
    let target = resolve_serial(serial)?;
    let _ = run_adb(&["shell", "am", "start", "-n", MAIN_ACTIVITY], Some(&target));
    let script_path = format!("/sdcard/Android/data/{PACKAGE_ID}/files/usb-start.sh");
    let (code, stdout, stderr) = run_adb(&["shell", "sh", &script_path], Some(&target))?;
    Ok(JsonValue::obj(vec![
        ("ok", JsonValue::Bool(code == 0)),
        ("serial", JsonValue::Str(target)),
        ("scriptPath", JsonValue::Str(script_path)),
        ("output", JsonValue::Str(format!("{stdout}{stderr}").trim().to_string())),
    ]))
}

pub fn cmd_run(serial: Option<&str>, intent: &str) -> Result<JsonValue, String> {
    let target = resolve_serial(serial)?;
    let (code, stdout, stderr) = run_adb(
        &["shell", "am", "start", "-n", MAIN_ACTIVITY, "--es", "sol_next_initial_intent", intent],
        Some(&target),
    )?;
    Ok(JsonValue::obj(vec![
        ("ok", JsonValue::Bool(code == 0)),
        ("serial", JsonValue::Str(target)),
        ("intent", JsonValue::Str(intent.to_string())),
        ("output", JsonValue::Str(format!("{stdout}{stderr}").trim().to_string())),
    ]))
}

pub fn cmd_verify_vdisplay(serial: Option<&str>) -> Result<JsonValue, String> {
    let target = resolve_serial(serial)?;
    let (_, stdout, _) = run_adb(&["shell", "dumpsys", "display"], Some(&target))?;
    let specs = parse_display_specs(&stdout);
    Ok(JsonValue::obj(vec![
        ("ok", JsonValue::Bool(true)),
        ("serial", JsonValue::Str(target)),
        (
            "display0",
            specs.display0.as_ref().map(DisplaySpec::to_json).unwrap_or(JsonValue::Null),
        ),
        (
            "virtualDisplay",
            specs.virtual_display.as_ref().map(DisplaySpec::to_json).unwrap_or(JsonValue::Null),
        ),
        ("exact1to1", JsonValue::Bool(specs.exact_1to1)),
    ]))
}

// ============================================================================
// In-Process Stdio Model Context Protocol (JSON-RPC 2.0) Dispatcher
// ============================================================================

pub fn mcp_tools_schema() -> JsonValue {
    JsonValue::Array(vec![
        JsonValue::obj(vec![
            ("name", JsonValue::Str("sol_device_status".to_string())),
            ("description", JsonValue::Str("Inspect connected ADB Android devices, check whether Sol Next Public Edition (ai.sol.next) is installed, and verify 1:1 VirtualDisplay alignment.".to_string())),
            ("inputSchema", JsonValue::obj(vec![
                ("type", JsonValue::Str("object".to_string())),
                ("properties", JsonValue::obj(vec![
                    ("serial", JsonValue::obj(vec![
                        ("type", JsonValue::Str("string".to_string())),
                        ("description", JsonValue::Str("Optional ADB device serial".to_string())),
                    ])),
                ])),
            ])),
        ]),
        JsonValue::obj(vec![
            ("name", JsonValue::Str("sol_install_public_apk".to_string())),
            ("description", JsonValue::Str("Download the official Sol Next v9.3.1-public APK from GitHub Releases or China direct mirror, verify SHA-256, install via ADB, and optionally grant runtime permissions.".to_string())),
            ("inputSchema", JsonValue::obj(vec![
                ("type", JsonValue::Str("object".to_string())),
                ("properties", JsonValue::obj(vec![
                    ("serial", JsonValue::obj(vec![
                        ("type", JsonValue::Str("string".to_string())),
                        ("description", JsonValue::Str("Optional ADB device serial".to_string())),
                    ])),
                    ("abi", JsonValue::obj(vec![
                        ("type", JsonValue::Str("string".to_string())),
                        ("enum", JsonValue::Array(vec![JsonValue::Str("arm64".to_string()), JsonValue::Str("universal".to_string())])),
                        ("default", JsonValue::Str("arm64".to_string())),
                    ])),
                    ("mirror", JsonValue::obj(vec![
                        ("type", JsonValue::Str("string".to_string())),
                        ("enum", JsonValue::Array(vec![JsonValue::Str("github".to_string()), JsonValue::Str("china".to_string()), JsonValue::Str("ghfast".to_string())])),
                        ("default", JsonValue::Str("github".to_string())),
                    ])),
                    ("grant_permissions", JsonValue::obj(vec![
                        ("type", JsonValue::Str("boolean".to_string())),
                        ("default", JsonValue::Bool(true)),
                    ])),
                ])),
            ])),
        ]),
        JsonValue::obj(vec![
            ("name", JsonValue::Str("sol_start_virtual_display_helper".to_string())),
            ("description", JsonValue::Str("Start the uid=2000 1:1 Background VirtualDisplay shell helper on the connected Android phone via ADB.".to_string())),
            ("inputSchema", JsonValue::obj(vec![
                ("type", JsonValue::Str("object".to_string())),
                ("properties", JsonValue::obj(vec![
                    ("serial", JsonValue::obj(vec![
                        ("type", JsonValue::Str("string".to_string())),
                        ("description", JsonValue::Str("Optional ADB device serial".to_string())),
                    ])),
                ])),
            ])),
        ]),
        JsonValue::obj(vec![
            ("name", JsonValue::Str("sol_run_phone_task".to_string())),
            ("description", JsonValue::Str("Dispatch a natural-language task intent to Sol Next (ai.sol.next) running on the connected Android phone.".to_string())),
            ("inputSchema", JsonValue::obj(vec![
                ("type", JsonValue::Str("object".to_string())),
                ("required", JsonValue::Array(vec![JsonValue::Str("intent".to_string())])),
                ("properties", JsonValue::obj(vec![
                    ("intent", JsonValue::obj(vec![
                        ("type", JsonValue::Str("string".to_string())),
                        ("description", JsonValue::Str("Natural-language task instruction for the Android phone".to_string())),
                    ])),
                    ("serial", JsonValue::obj(vec![
                        ("type", JsonValue::Str("string".to_string())),
                        ("description", JsonValue::Str("Optional ADB device serial".to_string())),
                    ])),
                ])),
            ])),
        ]),
    ])
}

fn call_mcp_tool(name: &str, arguments: &JsonValue) -> JsonValue {
    let serial = arguments
        .get("serial")
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let data = match name {
        "sol_device_status" => cmd_status(serial),
        "sol_install_public_apk" => {
            let abi = arguments
                .get("abi")
                .and_then(JsonValue::as_str)
                .unwrap_or("arm64")
                .to_string();
            let mirror = arguments
                .get("mirror")
                .and_then(JsonValue::as_str)
                .unwrap_or("github")
                .to_string();
            let grant = arguments
                .get("grant_permissions")
                .and_then(JsonValue::as_bool)
                .unwrap_or(true);
            let opts = InstallOptions {
                serial: serial.map(str::to_string),
                abi,
                mirror,
                apk_path: None,
                verify_sha256: true,
                grant_permissions: grant,
            };
            cmd_install(&opts).unwrap_or_else(|e| {
                JsonValue::obj(vec![
                    ("ok", JsonValue::Bool(false)),
                    ("error", JsonValue::Str(e)),
                ])
            })
        }
        "sol_start_virtual_display_helper" => cmd_start_helper(serial).unwrap_or_else(|e| {
            JsonValue::obj(vec![
                ("ok", JsonValue::Bool(false)),
                ("error", JsonValue::Str(e)),
            ])
        }),
        "sol_run_phone_task" => {
            let intent = arguments
                .get("intent")
                .and_then(JsonValue::as_str)
                .map(str::trim)
                .unwrap_or("");
            if intent.is_empty() {
                JsonValue::obj(vec![
                    ("ok", JsonValue::Bool(false)),
                    ("error", JsonValue::Str("intent is required".to_string())),
                ])
            } else {
                cmd_run(serial, intent).unwrap_or_else(|e| {
                    JsonValue::obj(vec![
                        ("ok", JsonValue::Bool(false)),
                        ("error", JsonValue::Str(e)),
                    ])
                })
            }
        }
        other => {
            return JsonValue::obj(vec![
                ("isError", JsonValue::Bool(true)),
                (
                    "content",
                    JsonValue::Array(vec![JsonValue::obj(vec![
                        ("type", JsonValue::Str("text".to_string())),
                        ("text", JsonValue::Str(format!("Unknown tool: {other}"))),
                    ])]),
                ),
            ]);
        }
    };

    let is_error = !data.get("ok").and_then(JsonValue::as_bool).unwrap_or(false);
    JsonValue::obj(vec![
        ("isError", JsonValue::Bool(is_error)),
        (
            "content",
            JsonValue::Array(vec![JsonValue::obj(vec![
                ("type", JsonValue::Str("text".to_string())),
                ("text", JsonValue::Str(data.to_pretty_string())),
            ])]),
        ),
    ])
}

pub fn handle_mcp_request(req: &JsonValue) -> Option<JsonValue> {
    let method = req.get("method").and_then(JsonValue::as_str).unwrap_or("");
    let req_id = req.get("id").cloned();
    let empty_obj = JsonValue::obj(vec![]);
    let params = req.get("params").unwrap_or(&empty_obj);

    match method {
        "initialize" => Some(JsonValue::obj(vec![
            ("jsonrpc", JsonValue::Str("2.0".to_string())),
            ("id", req_id.unwrap_or(JsonValue::Null)),
            (
                "result",
                JsonValue::obj(vec![
                    ("protocolVersion", JsonValue::Str(PROTOCOL_VERSION.to_string())),
                    (
                        "capabilities",
                        JsonValue::obj(vec![("tools", JsonValue::obj(vec![]))]),
                    ),
                    (
                        "serverInfo",
                        JsonValue::obj(vec![
                            ("name", JsonValue::Str("sol-next-mcp".to_string())),
                            ("version", JsonValue::Str(SERVER_VERSION.to_string())),
                        ]),
                    ),
                ]),
            ),
        ])),
        "notifications/initialized" => None,
        "ping" => Some(JsonValue::obj(vec![
            ("jsonrpc", JsonValue::Str("2.0".to_string())),
            ("id", req_id.unwrap_or(JsonValue::Null)),
            ("result", JsonValue::obj(vec![])),
        ])),
        "tools/list" => Some(JsonValue::obj(vec![
            ("jsonrpc", JsonValue::Str("2.0".to_string())),
            ("id", req_id.unwrap_or(JsonValue::Null)),
            ("result", JsonValue::obj(vec![("tools", mcp_tools_schema())])),
        ])),
        "tools/call" => {
            let name = params.get("name").and_then(JsonValue::as_str).unwrap_or("");
            let args = params.get("arguments").unwrap_or(&empty_obj);
            Some(JsonValue::obj(vec![
                ("jsonrpc", JsonValue::Str("2.0".to_string())),
                ("id", req_id.unwrap_or(JsonValue::Null)),
                ("result", call_mcp_tool(name, args)),
            ]))
        }
        other => {
            let id = req_id?;
            Some(JsonValue::obj(vec![
                ("jsonrpc", JsonValue::Str("2.0".to_string())),
                ("id", id),
                (
                    "error",
                    JsonValue::obj(vec![
                        ("code", JsonValue::Int(-32601)),
                        ("message", JsonValue::Str(format!("Method not found: {other}"))),
                    ]),
                ),
            ]))
        }
    }
}

pub fn handle_mcp_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    match parse_json(trimmed) {
        Ok(req) => handle_mcp_request(&req).map(|resp| resp.to_compact_string()),
        Err(err) => Some(
            JsonValue::obj(vec![
                ("jsonrpc", JsonValue::Str("2.0".to_string())),
                ("id", JsonValue::Null),
                (
                    "error",
                    JsonValue::obj(vec![
                        ("code", JsonValue::Int(-32700)),
                        ("message", JsonValue::Str(err)),
                    ]),
                ),
            ])
            .to_compact_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_nist_test_vectors() {
        assert_eq!(
            sha256_bytes(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_bytes(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_bytes(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn json_roundtrip_handles_unicode_and_escapes() {
        let raw = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"sol_run_phone_task","arguments":{"intent":"打开设置\n查看\"电量\""}}}"#;
        let parsed = parse_json(raw).expect("valid json");
        let intent = parsed
            .get("params")
            .and_then(|p| p.get("arguments"))
            .and_then(|a| a.get("intent"))
            .and_then(JsonValue::as_str)
            .expect("intent str");
        assert_eq!(intent, "打开设置\n查看\"电量\"");
        let re_parsed = parse_json(&parsed.to_pretty_string()).expect("pretty roundtrip");
        assert_eq!(parsed, re_parsed);
    }

    #[test]
    fn parse_display_specs_detects_exact_1to1_virtual_display() {
        let sample = r#"
          DisplayDeviceInfo{"Built-in Screen": uniqueId="local:0", 1080 x 2280, modeId 1, density 440, scale=1.0}
          DisplayDeviceInfo{"sol-next-bg": uniqueId="virtual:ai.sol.next", 1080 x 2280, modeId 2, density 440, scale=1.0}
        "#;
        let parsed = parse_display_specs(sample);
        assert!(parsed.exact_1to1);
        assert_eq!(parsed.display0.as_ref().unwrap().width, 1080);
        assert_eq!(parsed.virtual_display.as_ref().unwrap().density_dpi, 440);
    }

    #[test]
    fn mcp_protocol_handles_initialize_list_errors_and_validation() {
        let init = handle_mcp_line(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
            .expect("init response");
        assert!(init.contains("sol-next-mcp"));
        assert!(init.contains(SERVER_VERSION));

        let notif = handle_mcp_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
        assert!(notif.is_none());

        let list = handle_mcp_line(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#)
            .expect("tools/list");
        assert!(list.contains("sol_device_status"));
        assert!(list.contains("sol_install_public_apk"));
        assert!(list.contains("sol_run_phone_task"));

        let missing_intent = handle_mcp_line(
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"sol_run_phone_task","arguments":{}}}"#,
        )
        .expect("validation error");
        assert!(missing_intent.contains("\"isError\":true"));
        assert!(missing_intent.contains("intent is required"));

        let bad_method = handle_mcp_line(r#"{"jsonrpc":"2.0","id":4,"method":"unknown/op"}"#)
            .expect("unknown method");
        assert!(bad_method.contains("-32601"));

        let bad_json = handle_mcp_line("{broken").expect("parse error");
        assert!(bad_json.contains("-32700"));
    }
}
