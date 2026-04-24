# LCP Roadmap

## Phase 0 — RFC (current)

**Goal:** Define the protocol. Gather feedback from the systems engineering community.

- [x] Architectural proposal published
- [x] WHITEPAPER v1.0 Draft
- [x] Tool interface specification (`lcp_invoke`)
- [x] Trigger token format defined (`<|lcp_call|>`)
- [ ] Community feedback on open questions (see WHITEPAPER §6)
- [ ] Finalize tool interface ABI

## Phase 1 — Proof of Concept

**Goal:** A working demo. One tool. One model. Measurable latency improvement.

- [ ] Integrate `wasmtime-c-api` into llama.cpp build
- [ ] Implement `<|lcp_call|>` token interceptor in sampling loop
- [ ] Build `calculator.wasm` reference tool (arithmetic operations)
- [ ] Benchmark: LCP vs MCP latency on identical tool calls
- [ ] Publish benchmark results

**Success criteria:** Tool call round-trip under 1ms on consumer hardware.

**Seeking:** C++/Rust engineer familiar with llama.cpp internals.

## Phase 2 — Alpha

**Goal:** Multiple tools. Developer experience. Documentation.

- [ ] Tool registry (load all `.wasm` from configured directory)
- [ ] WASI capability grants per-tool (filesystem, network opt-in)
- [ ] CPU cycle limit enforcement
- [ ] `lcptool` CLI for packaging and testing tools
- [ ] Reference tools: filesystem read, HTTP client (restricted), JSON query
- [ ] Developer guide: "Build your first LCP tool in Rust"

## Phase 3 — Standard

**Goal:** Adoption. Integration with other runtimes.

- [ ] vLLM integration
- [ ] Ollama integration
- [ ] Fine-tuning dataset for `<|lcp_call|>` token emission
- [ ] LoRA adapter for existing function-calling models
- [ ] Formal specification (IETF-style RFC document)
- [ ] WASM Component Model compatibility

---

## How to Help

The biggest unblocking action right now is **Phase 1** — a PoC in llama.cpp.

If you are a systems engineer familiar with C++ or Rust and interested in LLM inference internals, open an issue or reach out. The WHITEPAPER has a full implementation guide for the llama.cpp integration.
