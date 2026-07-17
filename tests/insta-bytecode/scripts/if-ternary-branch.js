// Relational conditions in `if` statements and ternary expressions should
// compile to a single fused compare-and-branch opcode (e.g. `JumpIfNotLessThan`)
// instead of computing a boolean into a register (`LessThan`) and then testing
// it (`JumpIfFalse`). See `compile_if_else` / `compile_condition_and_branch`.
//
// `a`/`b` are identifiers (not literals) so constant folding leaves the
// relational comparisons in place.
let a = 1;
let b = 2;
let r = 0;

// `if`/`else` with `<` -> fused `JumpIfNotLessThan`.
if (a < b) {
  r = 1;
} else {
  r = 2;
}

// Parenthesized ternary with `>` -> fused `JumpIfNotGreaterThan`
// (also exercises `Expression::flatten` stripping the parentheses).
r = (a > b) ? 10 : 20;

// Non-relational condition -> `JumpIfFalse` fallback is preserved.
if (r) {
  r = 3;
}

r;
