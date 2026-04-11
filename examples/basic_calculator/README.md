# Basic Calculator - LCP WebAssembly Module

This example demonstrates creating a compact WebAssembly module in Rust following the Liquid Context Protocol (LCP) paradigm. The module implements basic arithmetic operations and is optimized for minimal size.

## Features

- **no_std**: Does not use Rust standard library to minimize size
- **Compactness**: The resulting `.wasm` file is only a few kilobytes
- **Exported Functions**:
  - `add_u64(a, b)` - addition of two 64-bit numbers
  - `mul_u64(a, b)` - multiplication of two 64-bit numbers
  - `add_safe(a, b)` - addition with overflow check
  - `mul_safe(a, b)` - multiplication with overflow check
  - `get_version()` - returns module version

## Requirements

1. **Rust** (latest stable version):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **wasm32-unknown-unknown target platform**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **Optimization tools** (optional, for even smaller size):
   ```bash
   cargo install wasm-opt
   ```

## Compilation

### Basic Compilation

```bash
cd examples/basic_calculator
cargo build --target wasm32-unknown-unknown --release
```

The resulting file will be located at:
```
target/wasm32-unknown-unknown/release/basic_calculator.wasm
```

### Check Size

```bash
ls -lh target/wasm32-unknown-unknown/release/basic_calculator.wasm
```

Expected size: **~2-4 KB** (without additional optimization).

### Additional Optimization (via wasm-opt)

If `wasm-opt` from Binaryen is installed:

```bash
wasm-opt -Oz target/wasm32-unknown-unknown/release/basic_calculator.wasm -o basic_calculator.min.wasm
```

This can reduce size by another 20-30%.

## Usage in JavaScript

```javascript
// Load module
const wasmBytes = await fetch('basic_calculator.wasm').then(r => r.arrayBuffer());
const { instance } = await WebAssembly.instantiate(wasmBytes);

// Call functions
const sum = instance.exports.add_u64(10, 20);
console.log(sum); // 30

const product = instance.exports.mul_u64(5, 7);
console.log(product); // 35

const version = instance.exports.get_version();
console.log(version.toString(16)); // "10000" (version 1.0.0)
```

## Usage in Python (via wasmer)

```python
from wasmer import Store, Module, Instance
import wasmer

store = Store()
with open("basic_calculator.wasm", "rb") as f:
    module = Module(store, f.read())

instance = Instance(module)

result = instance.exports.add_u64(100, 200)
print(f"Result: {result}")  # Result: 300
```

## Architecture in LCP Context

This module demonstrates LCP principles:
1. **Atomicity**: Each function performs one clear task
2. **Portability**: Works in any environment with WebAssembly support
3. **Minimalism**: No dependencies, minimal size
4. **Interoperability**: Easily callable from JS, Python, Go, and other languages

## License

MIT
