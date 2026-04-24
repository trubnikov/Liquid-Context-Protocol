# LCP Tool Examples

This directory contains reference implementations of LCP tools.

## calculator.wat

A minimal LCP tool written in raw WebAssembly Text format. Illustrates the tool interface contract. For educational purposes.

**To compile:**
```bash
# Install wat2wasm (part of WABT toolkit)
brew install wabt        # macOS
apt install wabt         # Ubuntu

# Compile
wat2wasm calculator.wat -o calculator.wasm
```

**LCP tool interface contract:**

```
Input:  UTF-8 JSON string in WASM linear memory (ptr: i32, len: i32)
Output: i64 = (output_ptr << 32) | output_len
```

Example call:
```json
{"op": "sqrt", "input": 144}
```

Expected result:
```json
{"result": 12}
```

---

## Coming in Phase 1

- `calculator/` — Full Rust implementation with real JSON parsing
- `fs-read/` — Filesystem read tool (WASI-based, sandboxed)
- `json-query/` — JSONPath query tool

See the [ROADMAP](../ROADMAP.md) for implementation timeline.
