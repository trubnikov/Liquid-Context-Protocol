# Liquid Context Protocol (LCP)

> **Status: RFC — Request for Comments.** This is an architectural proposal seeking systems engineers for the first PoC implementation.

---

## The Problem

Every time an AI agent calls a tool under MCP or ACP, this happens:

```
LLM generates text → serialize to JSON → HTTP round-trip → 
wait for external process → parse JSON response → inject into context
```

Each step adds latency. For simple tool calls on a local model, the network overhead can exceed the inference time itself. For autonomous agents making dozens of tool calls per task, this compounds into a fundamental throughput ceiling.

This is not a configuration problem. It is an architectural one.

---

## The Proposal

**LCP moves tool execution into the same process as inference.**

Tools are compiled as `.wasm` binary modules. When the LLM needs a tool, the inference engine — running an embedded WASM runtime like [Wasmtime](https://wasmtime.dev/) — executes the module locally and injects the result directly into the context buffer before the next token is sampled.

No HTTP. No JSON serialization. No external process. No network.

```
LLM generates <|lcp_call|> token
    → inference loop intercepts
    → WASM module executes in-process (~microseconds)
    → result appended to context buffer
    → generation resumes
```

The bottleneck is eliminated at the architectural level.

---

## Architecture

### The Trigger Token

The model is fine-tuned (or prompted) to emit a structured `<|lcp_call|>` token when it needs a tool:

```
<|lcp_call|>{"tool": "calculator", "op": "sqrt", "input": 144}<|lcp_end|>
```

The inference loop intercepts this token sequence before passing it to the sampler. Standard practice — this is how function calling works in models like Llama 3.1 and Mistral, but with an external dispatcher. LCP replaces the dispatcher with an in-process WASM runtime.

### In-Process Execution

```
┌─────────────────────────────────────────────┐
│              Inference Engine               │
│                (llama.cpp)                  │
│                                             │
│  ┌─────────────┐    ┌────────────────────┐  │
│  │  Tokenizer  │    │   WASM Runtime     │  │
│  │  + Sampler  │◄───│   (Wasmtime)       │  │
│  └─────────────┘    │                    │  │
│                     │  calculator.wasm   │  │
│                     │  filesystem.wasm   │  │
│                     │  http-client.wasm  │  │
│                     └────────────────────┘  │
└─────────────────────────────────────────────┘
```

Tools run in adjacent memory. The result is written to the context buffer. No process boundary is crossed.

### Security Model

WASM provides memory sandboxing by design. LCP modules run in a restricted environment:

- No filesystem access unless explicitly granted via WASI
- No network access by default
- CPU timeout enforced — no infinite loops
- Each module runs in its own memory sandbox

---

## Why Not Just Use MCP?

MCP is the right choice for cloud-hosted models and multi-tenant systems. It provides network isolation, easy distribution, and works with any language or runtime.

LCP is the right choice when:

- You are running open-weight models locally (llama.cpp, vLLM, Ollama)
- Tool call latency is a bottleneck for your agent architecture
- You want tools to run without network access
- You are building for edge or embedded deployment

These are different problems. LCP is not a replacement for MCP — it is an answer to a different question.

---

## Current Status

| Component | Status |
|-----------|--------|
| Specification | ✅ v1.0 Draft |
| Reference tool (`calculator.wasm`) | ✅ Built, ABI-conformant, tested |
| Conformance test suite + CI | ✅ 9/9 passing, mutation-checked |
| In-process execution benchmark | ✅ Measured (see below) |
| llama.cpp integration | ⬜ Seeking contributor |
| Wasmtime hook in sampling loop | ⬜ Seeking contributor |
| Fine-tuning dataset for `<\|lcp_call\|>` | ⬜ Planned |
| vLLM integration | ⬜ Planned |

---

## Verified (v1.1.0)

The reference tool in [`examples/calculator-rust`](examples/calculator-rust) is
compiled to a 3 KB `.wasm` and driven through the real LCP ABI by
[`tests/test_lcp.py`](tests/test_lcp.py) using the [Wasmtime](https://wasmtime.dev/)
runtime. This is not a mock — every assertion executes the compiled module.

```
LCP ABI conformance — calculator.wasm (Wasmtime)
  [PASS] add / sub / mul / div / sqrt / negatives / fractions
  [PASS] div-by-zero  -> {"error":"division by zero"}
  [PASS] unknown op   -> {"error":"unknown op"}
  9 passed, 0 failed
  latency: ~60 µs/call over 100,000 real invocations
```

**In-process vs. subprocess** (same machine, identical tool result):

| Path | Latency / call |
|------|----------------|
| LCP in-process (Wasmtime) | ~0.03–0.06 ms |
| Subprocess (stdio-style) | ~1.2 ms |
| HTTP MCP (typical, per spec §1) | 10–500 ms |

The test suite is **mutation-checked**: CI deliberately breaks the tool
(`+` → `*`) and the build fails unless the suite catches it. A test that can't
fail on broken code proves nothing.

### Reproduce

```bash
rustup target add wasm32-unknown-unknown
pip install wasmtime
cargo build --release --target wasm32-unknown-unknown \
  --manifest-path examples/calculator-rust/Cargo.toml
python tests/test_lcp.py
```

---

## How to Contribute

We need systems engineers familiar with C++ or Rust to build the first integration.

**Target:** `llama.cpp`  
**Objective:** Integrate `wasmtime-c-api` into the main sampling loop. Intercept the `<|lcp_call|>` token sequence, execute the referenced `.wasm` file, append the result to the context, and resume generation.

The full technical spec is in [`WHITEPAPER.md`](WHITEPAPER.md).  
The implementation roadmap is in [`ROADMAP.md`](ROADMAP.md).

Open an issue to discuss or reach out directly.

---

## Related Work

- **[Physics Engine](https://github.com/trubnikov/design-physics-engine)** — same philosophy applied to design systems. Tools are laws, not dictionaries. Context is structure, not prose.
- [llama.cpp](https://github.com/ggml-org/llama.cpp) — the primary integration target
- [Wasmtime](https://github.com/bytecodealliance/wasmtime) — the embedded WASM runtime
- [MCP](https://modelcontextprotocol.io) — the network-based standard this complements

---


---

## Part of the Exo-Somatic research program

This repository is one layer of a single research program on verifiable cognition:

**[Exo-Somatic](https://github.com/trubnikov/Exo-Somatic)** (theory: substrate-independent minds)
→ **[SES](https://github.com/trubnikov/SES)** (contract: signed identity snapshots)
→ **[qca-cycle](https://github.com/trubnikov/qca-cycle)** (mechanism: the cognitive loop)
→ **[Evidence](https://github.com/NousResearch/hermes-agent/pull/43306)** (substrate transition test)

Adjacent track: **[Liquid-Context-Protocol](https://github.com/trubnikov/Liquid-Context-Protocol)** — the same contract-first idea applied to LLM tool execution.

---

## License

MIT © Dima Trubnikov
