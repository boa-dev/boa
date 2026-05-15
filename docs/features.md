# ECMAScript Feature Support

Boa aims to implement the full ECMAScript specification. This document tracks
which features are supported, partially implemented, or not yet available.

Conformance is measured against the official
[Test262](https://github.com/tc39/test262) test suite. Current results are
published at **[boajs.dev/conformance](https://boajs.dev/conformance)**.

## Overall Conformance

| Metric | Value |
|--------|-------|
| **Test262 conformance** | ~94.5% |
| **Spec editions** | ES5 through ES16 (ES2025) |
| **Active features** | ~200 tracked in the test262 suite |

## Spec Edition Support

| Edition | Status |
|---------|--------|
| ES5 (2009) | ✅ Full |
| ES6/ES2015 | ✅ Full |
| ES7/ES2016 | ✅ Full |
| ES8/ES2017 | ✅ Full |
| ES9/ES2018 | ✅ Full |
| ES10/ES2019 | ✅ Full |
| ES11/ES2020 | ✅ Full |
| ES12/ES2021 | ✅ Full |
| ES13/ES2022 | ✅ Full |
| ES14/ES2023 | ✅ Full |
| ES15/ES2024 | ✅ Full |
| ES16/ES2025 | ✅ Mostly implemented |

## Built-in Objects

### Fundamental Objects

| Object | Status | Notes |
|--------|--------|-------|
| `Object` | ✅ | With `ObjectInitializer` and `ConstructorBuilder` |
| `Function` | ✅ | With `NativeFunction`, closures, async support |
| `Boolean` | ✅ |
| `Symbol` | ✅ | Well-known symbols, `Symbol.for`, `Symbol.keyFor` |

### Numbers and Dates

| Object | Status | Notes |
|--------|--------|-------|
| `Number` | ✅ | Includes `toExponential`, `toFixed`, `toPrecision` |
| `BigInt` | ✅ |
| `Math` | ✅ | Comprehensive math operations |
| `Date` | ✅ | With timezone support via ICU4X / Temporal |
| `Temporal` | ⚠️ | Available behind `temporal` Cargo feature |

### Text Processing

| Object | Status | Notes |
|--------|--------|-------|
| `String` | ✅ | Full including UTF-16, `repeat`, `matchAll` |
| `RegExp` | ✅ | Via `regress` crate |

### Indexed Collections

| Object | Status | Notes |
|--------|--------|-------|
| `Array` | ✅ | Full including typed species, immutables |
| `Int8Array` | ✅ | All TypedArray views |
| `Uint8Array` | ✅ |
| `Uint8ClampedArray` | ✅ |
| `Int16Array` | ✅ |
| `Uint16Array` | ✅ |
| `Int32Array` | ✅ |
| `Uint32Array` | ✅ |
| `Float32Array` | ✅ |
| `Float64Array` | ✅ |
| `Float16Array` | ⚠️ | Behind `float16` Cargo feature |
| `BigInt64Array` | ✅ |
| `BigUint64Array` | ✅ |

### Keyed Collections

| Object | Status | Notes |
|--------|--------|-------|
| `Map` | ✅ |
| `Set` | ✅ |
| `WeakMap` | ✅ | Via `JsWeakMap` wrapper |
| `WeakSet` | ✅ | Via `JsWeakSet` wrapper |

### Structured Data

| Object | Status | Notes |
|--------|--------|-------|
| `ArrayBuffer` | ✅ | With `transfer`, `resize` |
| `SharedArrayBuffer` | ✅ |
| `DataView` | ✅ |
| `Atomics` | ✅ | `wait`, `waitAsync`, `pause`, operations |
| `JSON` | ✅ | Parse/stringify with cycle detection |
| `structuredClone` | ✅ |

### Control Abstraction

| Object | Status | Notes |
|--------|--------|-------|
| `Promise` | ✅ |
| `GeneratorFunction` | ✅ |
| `AsyncGeneratorFunction` | ✅ |
| `AsyncFunction` | ✅ |
| `Iterator` | ✅ | Iterator helpers |

### Reflection

| Object | Status | Notes |
|--------|--------|-------|
| `Proxy` | ✅ |
| `Reflect` | ✅ |

### Internationalization

| Object | Status | Notes |
|--------|--------|-------|
| `Intl` | ⚠️ | Basic support via `intl` feature |
| `Intl.Collator` | ✅ | Via ICU4X |
| `Intl.DateTimeFormat` | ✅ |
| `Intl.NumberFormat` | ✅ | Includes `^` notation |
| `Intl.ListFormat` | ✅ |
| `Intl.Locale` | ⚠️ | Partial (`Intl.Locale-info` not implemented) |
| `Intl.DisplayNames` | ❌ | Not yet implemented |
| `Intl.RelativeTimeFormat` | ❌ | Not yet implemented |
| `Intl.DurationFormat` | ❌ | Not yet implemented |
| `Intl.Segmenter` | ⚠️ | Partial |
| `Intl.PluralRules` | ✅ |

### Module System

| Feature | Status | Notes |
|---------|--------|-------|
| ESM modules | ✅ | With async module loaders |
| Dynamic `import()` | ✅ | Async module loading |
| `import.meta` | ✅ |
| Source phase imports | ❌ | Not yet implemented |
| Import defer | ❌ | Not yet implemented |
| JSON modules | ✅ |

## ECMAScript Proposals and Recent Features

### Stage 4 (Finished Proposals)

| Proposal | Status | Notes |
|----------|--------|-------|
| Resizable ArrayBuffer | ✅ |
| Array Grouping | ✅ |
| `Promise.withResolvers` | ✅ |
| Decorators | ❌ | Not yet implemented |
| Duplicate named capture groups | ⚠️ | PR in progress |
| Immutable ArrayBuffer | ❌ | Not yet implemented |
| `Uint8Array` base64 | ❌ | Not yet implemented |
| `Map.prototype.emplace` | ✅ |
| `Array.fromAsync` | ✅ |
| `Atomics.pause` | ✅ |
| Explicit Resource Management | ❌ | Not yet implemented |
| Joint iteration | ❌ | Not yet implemented |

### Supported via Cargo Features

| Feature flag | What it enables |
|-------------|-----------------|
| `intl` | Internationalization (ICU4X-based) |
| `annex-b` | Annex B (web compatibility) features |
| `temporal` | `Temporal` proposal implementation |
| `experimental` | In-progress proposals |
| `float16` | `Float16Array` |

## Execution Limits

| Limit | Via `$boa.limits` | Default |
|-------|-------------------|---------|
| Loop iterations | `$boa.limits.loop` | Configurable |
| Recursion depth | `$boa.limits.recursion` | Configurable |
| Value stack | `$boa.limits.stack` | Configurable |
| Backtrace frames | `$boa.limits.backtrace` | Configurable |
| Instruction count | Via `ContextBuilder::instructions_remaining()` | Unlimited |

## Unimplemented Features (from test262 config)

The following features are listed as ignored in the test262 runner:

```
symbols-as-weakmap-keys
regexp-duplicate-named-groups
explicit-resource-management
joint-iteration
import-bytes
legacy-regexp
import-defer
ShadowRealm
decorators
uint8array-base64
source-phase-imports-module-source
immutable-arraybuffer
caller
Intl.DisplayNames
Intl.RelativeTimeFormat
Intl-enumeration
Intl.DurationFormat
Intl.Locale-info
```

## Feature Flags in the Engine

Boa's `core/engine/Cargo.toml` defines these feature flags:

| Feature | Description |
|---------|-------------|
| `default` | Standard ES features including `annex-b` |
| `experimental` | In-progress proposals |
| `temporal` | Temporal proposal implementation |
| `intl` | Internationalization (requires ICU4X data) |
| `annex-b` | Web compatibility extras |
| `float16` | Float16Array support |
| `xsum` | Cross-symbol-store merging |

## Testing Your Feature

To verify a specific feature's status:

```bash
# Run a specific test262 sub-suite
cargo run --release --bin boa_tester -- run \
    -s test/built-ins/Array

# Run with verbose output
cargo run --release --bin boa_tester -- run \
    -vv -s test/language/expressions

# Compare with the main branch results
cargo run --release --bin boa_tester -- compare \
    ./test-results-main/latest.json \
    ./test-results-feature/latest.json
```

## References

- [ECMAScript Language Specification](https://tc39.es/ecma262/)
- [Boa conformance dashboard](https://boajs.dev/conformance)
- [Test262 test suite](https://github.com/tc39/test262)
- [test262_config.toml](../test262_config.toml) — ignored features list
- [Boa issue tracker](https://github.com/boa-dev/boa/issues)
