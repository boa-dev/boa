# Issue: Refactor parser error context — implement ErrorContext for Error and remove dead-code markers

## Title (for GitHub)

**refactor(parser): implement ErrorContext for Error, remove redundant methods and allow(unused)**

## Description (for GitHub issue body)

### Summary

The parser's `Error` type had `set_context` and `context` as direct methods with an `#[allow(unused)]` and a TODO suggesting the `context()` method was unused and a candidate for removal. In reality, both methods are used: `ParseResult<T>`'s `ErrorContext` implementation delegates to them, and tests call `err.context()` and `err.set_context()` on `Error` directly. This issue tracks a small refactor that cleans this up without changing behavior.

### Current state

- **Trait:** `ErrorContext` is implemented only for `ParseResult<T>`. It has `set_context` and `context`; the trait's `context()` was marked `#[allow(dead_code)]`.
- **Error:** `Error` has its own `set_context(self, ...)` and `context(&self)` with `#[allow(unused)]` and a TODO: *"context method is unused, candidate for removal?"*
- **Usage:** `ParseResult::set_context` / `ParseResult::context` call `Error::set_context` and `Error::context`. Tests also call `err.set_context()` and `err.context()` on an `Error` value.

So the methods are not dead code; the allow and TODO are misleading.

### Proposed change

1. **Implement `ErrorContext` for `Error`**  
   Move the logic from `Error::set_context` and `Error::context` into `impl ErrorContext for Error`. This gives a single, consistent API: both `ParseResult<T>` and `Error` implement the same trait.

2. **Remove redundant methods from `impl Error`**  
   Delete the standalone `set_context` and `context` methods from `impl Error`, since they are now provided by the trait.

3. **Remove obsolete attributes and comments**  
   - Remove `#[allow(unused)]` and the TODO from the (now removed) `Error::context`.
   - Remove `#[allow(dead_code)]` from the trait's `context()` method, as it is used.

### Benefits

- **Clearer API:** Callers can use `ErrorContext` for both `Error` and `ParseResult<T>`.
- **Less noise:** No `#[allow(unused)]` or `#[allow(dead_code)]` for methods that are actually used.
- **No behavioral change:** Same behavior and same tests; refactor only.

### Files

- `core/parser/src/error/mod.rs`

### Testing

- Existing tests in `core/parser/src/error/tests.rs` (e.g. `context()`) should pass unchanged, as they already use `result.context()`, `result.set_context()`, `err.context()`, and `err.set_context()`.
