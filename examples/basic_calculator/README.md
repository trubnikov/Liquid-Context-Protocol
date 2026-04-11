# Basic Calculator - LCP WebAssembly Module

Этот пример демонстрирует создание компактного WebAssembly модуля на Rust в парадигме Liquid Context Protocol (LCP). Модуль реализует базовые арифметические операции и оптимизирован для минимального размера.

## Особенности

- **no_std**: Не использует стандартную библиотеку Rust для минимизации размера
- **Компактность**: Итоговый `.wasm` файл весит всего несколько килобайт
- **Экспортируемые функции**:
  - `add_u64(a, b)` - сложение двух 64-битных чисел
  - `mul_u64(a, b)` - умножение двух 64-битных чисел
  - `add_safe(a, b)` - сложение с проверкой переполнения
  - `mul_safe(a, b)` - умножение с проверкой переполнения
  - `get_version()` - возвращает версию модуля

## Требования

1. **Rust** (последняя стабильная версия):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Целевая платформа wasm32-unknown-unknown**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **Инструменты для оптимизации** (опционально, для ещё меньшего размера):
   ```bash
   cargo install wasm-opt
   ```

## Компиляция

### Базовая компиляция

```bash
cd examples/basic_calculator
cargo build --target wasm32-unknown-unknown --release
```

Итоговый файл будет находиться по пути:
```
target/wasm32-unknown-unknown/release/basic_calculator.wasm
```

### Проверка размера

```bash
ls -lh target/wasm32-unknown-unknown/release/basic_calculator.wasm
```

Ожидаемый размер: **~2-4 КБ** (без дополнительной оптимизации).

### Дополнительная оптимизация (через wasm-opt)

Если установлен `wasm-opt` из Binaryen:

```bash
wasm-opt -Oz target/wasm32-unknown-unknown/release/basic_calculator.wasm -o basic_calculator.min.wasm
```

Это может уменьшить размер ещё на 20-30%.

## Использование в JavaScript

```javascript
// Загрузка модуля
const wasmBytes = await fetch('basic_calculator.wasm').then(r => r.arrayBuffer());
const { instance } = await WebAssembly.instantiate(wasmBytes);

// Вызов функций
const sum = instance.exports.add_u64(10, 20);
console.log(sum); // 30

const product = instance.exports.mul_u64(5, 7);
console.log(product); // 35

const version = instance.exports.get_version();
console.log(version.toString(16)); // "10000" (версия 1.0.0)
```

## Использование в Python (через wasmer)

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

## Архитектура в контексте LCP

Этот модуль демонстрирует принципы LCP:
1. **Атомарность**: Каждая функция выполняет одну четкую задачу
2. **Переносимость**: Работает в любой среде с поддержкой WebAssembly
3. **Минимализм**: Нет зависимостей, минимальный размер
4. **Интероперабельность**: Легко вызывается из JS, Python, Go и других языков

## Лицензия

MIT
