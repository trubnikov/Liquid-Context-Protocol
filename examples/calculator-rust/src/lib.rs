//! LCP reference tool — calculator.wasm
//!
//! Conforms to the LCP ABI defined in WHITEPAPER.md §2.1:
//!   - single entry point `lcp_invoke(input_ptr: i32, input_len: i32) -> i64`
//!   - input:  UTF-8 JSON string at (input_ptr, input_len) in linear memory
//!   - output: i64 packed as (output_ptr << 32) | output_len
//!   - host reads the result string from linear memory at output_ptr
//!
//! Also exports `lcp_alloc` so the host can obtain a buffer to write input into.
//! no_std + no allocator: a fixed static arena lives in the module's own memory.
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// ---- static arenas in linear memory -------------------------------------- //
// Input buffer the host writes into (offset returned by lcp_alloc).
static mut INPUT: [u8; 4096] = [0; 4096];
// Output buffer the result string is written into.
static mut OUTPUT: [u8; 256] = [0; 256];

/// Host calls this first to learn where to write the input bytes.
#[no_mangle]
pub extern "C" fn lcp_alloc(_size: i32) -> i32 {
    unsafe { INPUT.as_ptr() as i32 }
}

/// Entry point. Parses {"op": "...","a":N,"b":N} and returns {"result":N}.
#[no_mangle]
pub extern "C" fn lcp_invoke(input_ptr: i32, input_len: i32) -> i64 {
    let input: &[u8] =
        unsafe { core::slice::from_raw_parts(input_ptr as *const u8, input_len as usize) };

    let op = json_str(input, b"op");
    let a = json_num(input, b"a");
    let b = json_num(input, b"b");

    let result: f64 = match op {
        Some(b"add") => a + b,
        Some(b"sub") => a - b,
        Some(b"mul") => a * b,
        Some(b"div") => {
            if b == 0.0 {
                return write_out(b"{\"error\":\"division by zero\"}");
            }
            a / b
        }
        Some(b"sqrt") => fsqrt(a),
        _ => return write_out(b"{\"error\":\"unknown op\"}"),
    };

    // Build {"result":<number>} into a small scratch then OUTPUT.
    let mut buf = [0u8; 256];
    let mut n = 0;
    for &c in b"{\"result\":" {
        buf[n] = c;
        n += 1;
    }
    n += write_f64(&mut buf[n..], result);
    buf[n] = b'}';
    n += 1;
    write_out(&buf[..n])
}

// ---- minimal JSON helpers (no_std, no alloc) ----------------------------- //

/// Find `"key"` and return the quoted string value after the following colon.
fn json_str<'a>(s: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    let i = find_key(s, key)?;
    let mut j = i;
    while j < s.len() && s[j] != b':' {
        j += 1;
    }
    j += 1;
    while j < s.len() && (s[j] == b' ' || s[j] == b'"') {
        j += 1;
    }
    let start = j;
    while j < s.len() && s[j] != b'"' {
        j += 1;
    }
    Some(&s[start..j])
}

/// Find `"key"` and parse the numeric value after the following colon.
fn json_num(s: &[u8], key: &[u8]) -> f64 {
    let i = match find_key(s, key) {
        Some(v) => v,
        None => return 0.0,
    };
    let mut j = i;
    while j < s.len() && s[j] != b':' {
        j += 1;
    }
    j += 1;
    while j < s.len() && (s[j] == b' ' || s[j] == b'"') {
        j += 1;
    }
    parse_f64(&s[j..])
}

fn find_key(s: &[u8], key: &[u8]) -> Option<usize> {
    if key.is_empty() || s.len() < key.len() + 2 {
        return None;
    }
    let mut i = 0;
    while i + key.len() + 2 <= s.len() {
        if s[i] == b'"' && &s[i + 1..i + 1 + key.len()] == key && s[i + 1 + key.len()] == b'"' {
            return Some(i + 1 + key.len());
        }
        i += 1;
    }
    None
}

fn parse_f64(s: &[u8]) -> f64 {
    let mut i = 0;
    let mut sign = 1.0;
    if i < s.len() && s[i] == b'-' {
        sign = -1.0;
        i += 1;
    }
    let mut val = 0.0;
    while i < s.len() && s[i].is_ascii_digit() {
        val = val * 10.0 + (s[i] - b'0') as f64;
        i += 1;
    }
    if i < s.len() && s[i] == b'.' {
        i += 1;
        let mut frac = 0.1;
        while i < s.len() && s[i].is_ascii_digit() {
            val += (s[i] - b'0') as f64 * frac;
            frac *= 0.1;
            i += 1;
        }
    }
    sign * val
}

/// Newton's method sqrt — no std, no intrinsics.
fn fsqrt(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut g = x;
    let mut i = 0;
    while i < 40 {
        g = 0.5 * (g + x / g);
        i += 1;
    }
    g
}

/// Write an f64 as text. Integers print without a decimal point.
fn write_f64(out: &mut [u8], v: f64) -> usize {
    let mut n = 0;
    let mut x = v;
    if x < 0.0 {
        out[n] = b'-';
        n += 1;
        x = -x;
    }
    let int_part = x as u64;
    n += write_u64(&mut out[n..], int_part);
    let mut frac = x - int_part as f64;
    if frac > 1e-9 {
        out[n] = b'.';
        n += 1;
        let mut guard = 0;
        while frac > 1e-9 && guard < 6 {
            frac *= 10.0;
            let d = frac as u8;
            out[n] = b'0' + d;
            n += 1;
            frac -= d as f64;
            guard += 1;
        }
    }
    n
}

fn write_u64(out: &mut [u8], mut v: u64) -> usize {
    if v == 0 {
        out[0] = b'0';
        return 1;
    }
    let mut tmp = [0u8; 20];
    let mut len = 0;
    while v > 0 {
        tmp[len] = b'0' + (v % 10) as u8;
        v /= 10;
        len += 1;
    }
    let mut i = 0;
    while i < len {
        out[i] = tmp[len - 1 - i];
        i += 1;
    }
    len
}

fn write_out(bytes: &[u8]) -> i64 {
    unsafe {
        let out_ptr = OUTPUT.as_mut_ptr();
        let n = if bytes.len() > 256 { 256 } else { bytes.len() };
        let mut i = 0;
        while i < n {
            *out_ptr.add(i) = bytes[i];
            i += 1;
        }
        ((out_ptr as i64) << 32) | (n as i64)
    }
}
