# `boa_engine` Public-Fields Audit Slice (`#4524`)

Date: 2026-03-01
Scope: first narrow slice for API hardening before `1.0`

## Method

- Source scan in `core/engine/src` for explicit `pub` fields.
- Cross-check with `cargo +nightly public-api -p boa_engine -ss`.
- Classify each item by API stability risk and migration complexity.

## Public struct fields (direct exposure)

| Type | Source | Field(s) | Recommendation | Priority |
|---|---|---|---|---|
| `OptimizerStatistics` | `core/engine/src/optimizer/mod.rs` | `constant_folding_run_count`, `constant_folding_pass_count` | Add getters; make fields private after migration; consider `#[non_exhaustive]` on struct for future counters. | High |
| `JsNativeError` | `core/engine/src/error/mod.rs` | `kind` | Add `kind(&self)` accessor and migrate docs/callers away from direct field access; then privatize before `1.0`. | High |
| `JsErasedNativeError` | `core/engine/src/error/mod.rs` | `kind` | Same approach as `JsNativeError`. | High |
| `ResolvingFunctions` | `core/engine/src/builtins/promise/mod.rs` | `resolve`, `reject` | Keep public for now (core ergonomic API), but add `#[non_exhaustive]` to allow future expansion without locking constructor shape. | Medium |
| `RecursionLimiter` | `core/engine/src/object/jsobject.rs` | `visited`, `live` | Prefer making internals private and expose `visited()` / `live()` methods if external access is needed. Consider `pub(crate)` if no external use is intended. | Medium |
| `JsPartialTime` | `core/engine/src/builtins/temporal/plain_time/mod.rs` | `hour`, `minute`, `second`, `millisecond`, `microsecond`, `nanosecond` | This looks like internal representation. Prefer `pub(crate)`; if kept public, use private fields + accessors/builder. | Medium |

## Public enum structural fields (match-shape exposure)

| Enum | Source | Exposed variant fields | Recommendation | Priority |
|---|---|---|---|---|
| `TryNativeError` | `core/engine/src/error/mod.rs` | `InaccessibleProperty::{property, source}`, `InvalidErrorsIndex::{index, source}`, `InaccessibleRealm::{source}`, `EngineError::{source}` | Add `#[non_exhaustive]` to enum and prefer helper accessors over destructuring in docs/examples. | High |
| `DescriptorKind` | `core/engine/src/property/mod.rs` | `Data::{value, writable}`, `Accessor::{get, set}` | Keep data model, but add `#[non_exhaustive]` and nudge users toward `PropertyDescriptor` accessor methods. | Medium |
| `PrivateElement` | `core/engine/src/object/mod.rs` | `Accessor::{getter, setter}` | Evaluate restricting visibility (`pub(crate)`) if not intended for embedders. Otherwise add `#[non_exhaustive]`. | Medium |

## Recommended first PR from this slice

1. `OptimizerStatistics`: add getters + privatize fields.
2. `JsNativeError` and `JsErasedNativeError`: add `kind()` accessors + docs migration.
3. `TryNativeError`: add `#[non_exhaustive]`.

This keeps the first change set small, reviewable, and high-impact for `1.0` API stability.
