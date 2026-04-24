# Liquid Context Protocol — Technical Specification v1.0

**Status:** Draft RFC  
**Author:** Dima Trubnikov  
**Target Runtime:** llama.cpp (primary), vLLM (planned)  
**WASM Runtime:** Wasmtime / WasmEdge

---

## Abstract

LCP (Liquid Context Protocol) is an architectural proposal for eliminating the network layer between an LLM inference engine and its tools. Current tool-use protocols (MCP, ACP) rely on JSON-RPC over HTTP or stdio, creating a process boundary that adds latency on every tool call. LCP replaces this boundary with in-process WebAssembly execution: tools are compiled `.wasm` binaries loaded directly into the inference engine's address space at startup. Tool execution time drops from network-bound (10–500ms) to memory-bound (microseconds).

---

## 1. The Bottleneck

A standard MCP tool call looks like this:

```
[LLM] → generate tool_use block (text)
[Engine] → serialize to JSON
[Engine] → send over HTTP or stdio to tool server
[Server] → deserialize JSON
[Server] → execute tool logic
[Server] → serialize result to JSON
[Engine] → receive response
[Engine] → inject result text into context
[LLM] → resume generation
```

Eight steps. Each crosses a process or network boundary. For an autonomous agent executing 20–50 tool calls per task, this latency accumulates significantly — especially on local hardware where the tool server and inference engine compete for the same CPU.

The fundamental issue is that the tool result must travel as text through a serialization/deserialization pipeline to reach the context. This is a protocol artifact, not a necessity.

---

## 2. The LCP Architecture

LCP removes the protocol boundary. Tools run in the same process as the inference engine.

### 2.1 Tool Format

An LCP tool is a standard WebAssembly binary (`.wasm`) compiled from any language (Rust, C, C++, Go). It exposes a single entry point:

```rust
// Rust example — compiled to calculator.wasm
#[no_mangle]
pub extern "C" fn lcp_invoke(input_ptr: i32, input_len: i32) -> i64 {
    let input = read_string(input_ptr, input_len);
    let args: serde_json::Value = serde_json::from_str(&input).unwrap();
    
    let result = match args["op"].as_str().unwrap() {
        "sqrt" => (args["input"].as_f64().unwrap()).sqrt(),
        "abs"  => (args["input"].as_f64().unwrap()).abs(),
        _      => return encode_error("unknown op"),
    };
    
    encode_result(&result.to_string())
}
```

The interface is minimal: one function, string in, string out. Any language that compiles to WASM can implement it.

### 2.2 The Trigger Token

The model emits a structured token sequence when it needs a tool. This token format is defined by LCP and the model is trained or prompted to produce it:

```
<|lcp_call|>{"tool":"calculator","op":"sqrt","input":144}<|lcp_end|>
```

The inference engine intercepts this sequence at the sampling stage — before the tokens are written to the output buffer.

This is not novel: Llama 3.1, Mistral, and Qwen models already use special token sequences to trigger function calling. LCP defines the same mechanism but routes execution to local WASM instead of an external server.

### 2.3 Execution Pipeline

```
Token stream: ... "what is √144?" ...
                              ↓
                   <|lcp_call|> detected
                              ↓
                   Parse: { tool: "calculator", op: "sqrt", input: 144 }
                              ↓
                   Lookup: calculator.wasm (already loaded in WASM runtime)
                              ↓
                   lcp_invoke(input_json) → "12"
                              ↓
                   Append to context buffer: "The answer is 12"
                              ↓
                   Resume generation
```

No network. No subprocess. No serialization round-trip beyond the initial WASM call argument (a string in linear memory — same address space).

### 2.4 Context Injection

The WASM module returns a result string. The inference engine appends this to the active context buffer as a system-role message:

```
<|system|>Tool result [calculator]: 12<|end|>
```

The model then continues generating with this result already in context. From the model's perspective, the tool call and result are part of the same token stream — there is no observable pause or protocol boundary.

This is distinct from directly modifying the K/V cache (which would require re-running the attention computation over the injected tokens). Context injection triggers normal forward processing of the new tokens on the next generation step, which is both correct and simpler to implement.

---

## 3. Security Model

WebAssembly's memory isolation is LCP's security foundation. Each module:

- **Cannot read or write outside its linear memory** — enforced by the WASM runtime, not by policy
- **Has no filesystem access** by default — WASI capabilities must be explicitly granted per module
- **Has no network access** by default — same WASI capability model
- **Is bounded by CPU time** — the engine enforces a cycle limit; modules that exceed it are terminated

This gives LCP stronger isolation guarantees than a subprocess-based tool server at lower overhead.

### 3.1 Threat Model

| Threat | Mitigation |
|--------|-----------|
| Malicious tool reads model weights | WASM linear memory is isolated from engine memory |
| Tool enters infinite loop | CPU cycle limit enforced by runtime |
| Tool escapes sandbox | Not possible without WASM runtime vulnerability |
| Tool exfiltrates data via network | Network capability not granted by default |

---

## 4. Comparison with MCP

| Property | MCP | LCP |
|----------|-----|-----|
| Tool location | External process / server | In-process WASM module |
| Transport | HTTP / stdio | Direct function call |
| Latency | 10–500ms per call | ~microseconds per call |
| Language support | Any (server process) | Any (WASM target) |
| Security boundary | Process isolation | WASM memory isolation |
| Network required | Yes | No |
| Best for | Cloud models, multi-tenant | Local models, edge, latency-critical |

LCP and MCP solve different problems. A production system may use both: MCP for cloud-integrated tools, LCP for latency-critical local operations.

---

## 5. Implementation Guide (llama.cpp)

This section defines the target integration for the first LCP PoC.

### 5.1 Dependencies

```cmake
# Add to llama.cpp CMakeLists.txt
find_package(wasmtime-c-api REQUIRED)
target_link_libraries(llama wasmtime)
```

### 5.2 Initialization

At engine startup, scan the configured `lcp_tools_dir` for `.wasm` files and load each into the Wasmtime runtime:

```cpp
// Pseudocode
wasm_engine_t* engine = wasm_engine_new();
wasm_store_t*  store  = wasm_store_new(engine);

for (const auto& path : tools_dir) {
    auto bytes = read_file(path);
    wasm_module_t* mod = wasm_module_new(store, &bytes);
    tool_registry[tool_name(path)] = mod;
}
```

### 5.3 Token Interception

In the sampling loop, after logits are computed but before the next token is committed:

```cpp
// Pseudocode — insert after logit processing
if (is_lcp_call_sequence(current_tokens)) {
    auto call = parse_lcp_call(current_tokens);
    auto* mod  = tool_registry[call.tool_name];
    
    std::string result = invoke_wasm_tool(store, mod, call.args_json);
    
    // Inject result as system message tokens
    auto result_tokens = tokenize(format_tool_result(call.tool_name, result));
    context_buffer.insert(result_tokens);
    
    // Clear the lcp_call tokens — they are not written to output
    current_tokens.clear_lcp_sequence();
}
```

### 5.4 WASM Tool Interface

Every LCP-compatible tool must export:

```wat
(func (export "lcp_invoke")
  (param $input_ptr i32)
  (param $input_len i32)
  (result i64))   ;; returns ptr<<32 | len
```

Input and output are UTF-8 JSON strings passed through WASM linear memory.

---

## 6. Open Questions

These are the unresolved design decisions where community input is needed:

**Q1: Token format.** Should `<|lcp_call|>` be a single special token (requires tokenizer modification) or a multi-token sentinel sequence (works with any tokenizer)? Multi-token is easier to implement; single token is cleaner.

**Q2: Streaming results.** Can a WASM module stream partial results into the context during generation, or must it return a complete result before generation resumes?

**Q3: Stateful tools.** Should modules be instantiated once and persist state between calls, or instantiated fresh per call? Fresh instantiation is safer; persistent state enables tools like an in-memory database.

**Q4: Fine-tuning dataset.** What is the minimum dataset size to reliably teach a model to emit well-formed `<|lcp_call|>` sequences? Can this be done with LoRA on an existing function-calling model?

---

## 7. Prior Art

- **llama.cpp tool-use** — implements function calling via text parsing, routes to external servers
- **Extism** — plugin system for embedding WASM in applications; closest existing implementation to LCP's model
- **WebAssembly Component Model** — W3C proposal for WASM module interfaces; LCP's tool interface is compatible
- **MCP** — the network-based standard LCP complements for local deployment

---

*This document is a living RFC. Sections marked with ⚠️ are pending community validation.*
