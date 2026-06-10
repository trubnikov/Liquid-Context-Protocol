# DEPRECATED — see ../calculator-rust

This early example exported ad-hoc functions (`add_u64`, `mul_u64`) that did
**not** follow the LCP ABI defined in `WHITEPAPER.md` §2.1.

The conformant, tested reference tool now lives in
[`../calculator-rust`](../calculator-rust) — it implements the required
`lcp_invoke(ptr, len) -> i64` entry point and passes the conformance suite in
`tests/test_lcp.py`.
