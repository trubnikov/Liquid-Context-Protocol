# Changelog

## v1.1.0 — Verified reference tool

This release turns LCP from a documents-only RFC into an RFC with a working,
tested core.

### Added
- **`examples/calculator-rust`** — ABI-conformant reference tool. Implements the
  `lcp_invoke(ptr, len) -> i64` entry point exactly as specified in
  WHITEPAPER.md §2.1, with a `no_std` hand-rolled JSON parser (no external deps).
  Supports `add`, `sub`, `mul`, `div` (with div-by-zero handling), `sqrt`.
  Compiles to a 3 KB `.wasm`.
- **`tests/test_lcp.py`** — conformance suite that loads the compiled module in
  Wasmtime and drives it through the real ABI (alloc → write input → invoke →
  read packed result). 9 cases, all executing real WASM.
- **`.github/workflows/ci.yml`** — builds the wasm and runs the suite on every
  push. Includes a **mutation check**: the build fails unless a deliberately
  broken tool is caught by the tests.
- **Verified section in README** — measured in-process latency (~60 µs/call over
  100k invocations) vs. subprocess (~1.2 ms) and HTTP MCP (10–500 ms).

### Fixed
- The old `examples/basic_calculator` exported ad-hoc functions
  (`add_u64`, `mul_u64`) that did **not** follow the LCP ABI — a contradiction
  between the spec and its own reference code. It is now deprecated and points
  to the conformant tool.

### Notes
- A real cache pitfall surfaced during testing: `cargo build` can report
  `Finished` without recompiling after a source edit. CI uses `cargo clean`
  before each build so the mutation check is meaningful.
- Phase 1 (llama.cpp integration) remains open and is the project's biggest
  unblocking action. The tool ABI it must target is now proven to work.
