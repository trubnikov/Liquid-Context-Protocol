;; calculator.wat — Reference LCP Tool Implementation
;;
;; This is the simplest possible LCP tool, written in WebAssembly Text format.
;; In production, use Rust or C (compiled to WASM) for more complex tools.
;;
;; Compile to .wasm:
;;   wat2wasm calculator.wat -o calculator.wasm
;;
;; Interface contract (all LCP tools must follow this):
;;   Input:  UTF-8 JSON string in linear memory (ptr, len)
;;   Output: i64 = (result_ptr << 32) | result_len
;;
;; Example input:  {"op":"add","a":10,"b":32}
;; Example output: {"result":42}

(module
  ;; Linear memory: 64KB, shared with the host (inference engine)
  (memory (export "memory") 1)

  ;; Output buffer starts at offset 1024
  ;; Input is passed by the host at offset 0
  (data (i32.const 0) "")

  ;; lcp_invoke: entry point for all LCP tool calls
  ;; @param input_ptr  - offset into linear memory where JSON input begins
  ;; @param input_len  - byte length of input JSON
  ;; @returns i64      - (output_ptr << 32) | output_len
  ;;
  ;; NOTE: This .wat example returns a static response for illustration.
  ;; A real tool would parse the JSON input and compute a result.
  ;; Use the Rust template (examples/calculator-rust/) for a real implementation.
  (func (export "lcp_invoke")
    (param $input_ptr i32)
    (param $input_len i32)
    (result i64)

    ;; Write a static JSON result to offset 1024
    ;; Real implementation would parse input and compute
    (i32.store8 (i32.const 1024) (i32.const 123))   ;; {
    (i32.store8 (i32.const 1025) (i32.const 34))    ;; "
    (i32.store8 (i32.const 1026) (i32.const 114))   ;; r
    (i32.store8 (i32.const 1027) (i32.const 101))   ;; e
    (i32.store8 (i32.const 1028) (i32.const 115))   ;; s
    (i32.store8 (i32.const 1029) (i32.const 117))   ;; u
    (i32.store8 (i32.const 1030) (i32.const 108))   ;; l
    (i32.store8 (i32.const 1031) (i32.const 116))   ;; t
    (i32.store8 (i32.const 1032) (i32.const 34))    ;; "
    (i32.store8 (i32.const 1033) (i32.const 58))    ;; :
    (i32.store8 (i32.const 1034) (i32.const 52))    ;; 4
    (i32.store8 (i32.const 1035) (i32.const 50))    ;; 2
    (i32.store8 (i32.const 1036) (i32.const 125))   ;; }

    ;; Return: ptr=1024 shifted into high 32 bits, len=13 in low 32 bits
    ;; Encoded as: (1024 << 32) | 13
    (i64.or
      (i64.shl (i64.const 1024) (i64.const 32))
      (i64.const 13)
    )
  )
)
