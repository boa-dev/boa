# fix(parser): replace expect_expression panic with try_into_expression error handling

## Summary

Replaces all uses of `expect_expression()` with `try_into_expression()` in the parser, and removes the panicking `expect_expression()` method. When the parser encounters arrow-function parameter list syntax in a context that requires an expression (e.g. conditional operator `a ? b : c`, call expression `foo()`, optional chaining `a?.b`), it now returns a recoverable parse error instead of panicking.

## Motivation

The `FormalParameterListOrExpression` type represents the grammar ambiguity between `(a, b)` as a comma expression vs `(a, b)` as arrow-function parameters. In contexts like the conditional operator (`x ? y : z`), call expressions (`f()`), and optional chaining (`a?.b`), the left-hand side must be an expression—arrow params are invalid. Previously, call sites used `expect_expression()`, which panics with "Unexpected arrow-function arguments" when the parser produced a `FormalParameterList` instead of an `Expression`. This can occur with edge-case input such as `(a) ? 1 : 2` when `(a)` is parsed as a single arrow param. For Boa as an embedded engine or in tooling, panics are unacceptable; the host expects a parse error. The `try_into_expression()` method already existed and returns `Err(Error::General { ... })` with a helpful message ("invalid arrow-function arguments (parentheses around the arrow-function may help)"). Replacing `expect_expression()` with `try_into_expression()?` propagates that error instead of panicking, aligning with Boa's stability goals.

## Changes

| Category     | Description                                                                                           |
| ------------ | ----------------------------------------------------------------------------------------------------- |
| **Replaced** | `lhs.expect_expression()` with `lhs.try_into_expression()?` in conditional expression parser          |
| **Replaced** | `member.expect_expression()` with `member.try_into_expression()?` in left-hand side (call expression) |
| **Replaced** | `lhs.expect_expression()` with `lhs.try_into_expression()?` in left-hand side (optional expression)   |
| **Removed**  | `expect_expression()` method from `FormalParameterListOrExpression` to prevent future misuse          |

## Technical Details

- **Files modified:** `core/parser/src/parser/expression/assignment/conditional.rs`, `core/parser/src/parser/expression/left_hand_side/mod.rs`, `core/parser/src/parser/expression/fpl_or_exp.rs`
- **Lines changed:** ~15 lines across 3 files
- **Behavioral impact:** Invalid input that previously caused a panic now produces `Err(Error::General { message: "invalid arrow-function arguments (parentheses around the arrow-function may help)", position })`. Valid input is unchanged.

## Testing

- [x] `cargo test -p boa_parser` — all 296 tests pass
- [x] `cargo test -p boa_engine --lib` — all 921 tests pass
- [x] `cargo clippy` — no warnings
