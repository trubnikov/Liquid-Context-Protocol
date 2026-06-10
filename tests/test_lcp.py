#!/usr/bin/env python3
"""
LCP conformance test runner.

Loads the compiled calculator.wasm and drives it THROUGH the real LCP ABI:
  1. call lcp_alloc to get the input buffer offset
  2. write the JSON request into linear memory there
  3. call lcp_invoke(ptr, len) -> packed i64
  4. unpack (out_ptr, out_len), read the result string back from memory
  5. assert it equals the expected JSON

This is not a mock. Every assertion executes real WASM in Wasmtime.
Exit code 0 = all pass, 1 = any failure.
"""
import sys, time
from wasmtime import Store, Module, Engine, Instance

WASM = sys.argv[1] if len(sys.argv) > 1 else \
    "examples/calculator-rust/target/wasm32-unknown-unknown/release/lcp_calculator.wasm"

engine = Engine()
store = Store(engine)
module = Module.from_file(engine, WASM)
inst = Instance(store, module, [])
ex = inst.exports(store)
lcp_alloc = ex["lcp_alloc"]
lcp_invoke = ex["lcp_invoke"]
memory = ex["memory"]


def invoke(req: str) -> str:
    data = req.encode()
    ptr = lcp_alloc(store, len(data))
    memory.write(store, data, ptr)
    packed = lcp_invoke(store, ptr, len(data))
    packed &= (1 << 64) - 1                       # treat as unsigned
    out_ptr, out_len = packed >> 32, packed & 0xFFFFFFFF
    return bytes(memory.read(store, out_ptr, out_ptr + out_len)).decode()


CASES = [
    ('{"op":"add","a":10,"b":32}',      '{"result":42}'),
    ('{"op":"sub","a":100,"b":58}',     '{"result":42}'),
    ('{"op":"mul","a":6,"b":7}',        '{"result":42}'),
    ('{"op":"div","a":84,"b":2}',       '{"result":42}'),
    ('{"op":"div","a":1,"b":0}',        '{"error":"division by zero"}'),
    ('{"op":"sqrt","a":144,"b":0}',     '{"result":12}'),
    ('{"op":"add","a":-5,"b":5}',       '{"result":0}'),
    ('{"op":"bogus","a":1,"b":1}',      '{"error":"unknown op"}'),
    ('{"op":"add","a":2.5,"b":1.5}',    '{"result":4}'),
]

passed = failed = 0
print(f"LCP ABI conformance — {WASM}\n" + "-" * 60)
for req, expected in CASES:
    try:
        got = invoke(req)
        ok = got == expected
    except Exception as e:                        # a crash is a failure, not a pass
        got, ok = f"EXCEPTION {type(e).__name__}: {e}", False
    mark = "PASS" if ok else "FAIL"
    if ok:
        passed += 1
    else:
        failed += 1
    print(f"  [{mark}] {req:<34} -> {got:<30} (want {expected})")

# latency micro-benchmark over the real module
N = 100_000
t0 = time.perf_counter()
for _ in range(N):
    invoke('{"op":"add","a":10,"b":32}')
per_us = (time.perf_counter() - t0) / N * 1e6

print("-" * 60)
print(f"  {passed} passed, {failed} failed")
print(f"  latency: {per_us:.2f} µs/call over {N} real invocations")
sys.exit(0 if failed == 0 else 1)
