// Snapshot pins the bytecode for a function call whose callee is resolved
// through an object Environment Record (a `with` statement). PR #5507 makes
// the compiler emit `GetNameAndLocator` to resolve the binding/locator once,
// then `ThisForObjectEnvironmentName dst:rNN` (dst-only; the old `index`
// operand was removed) to derive the call's `this` (WithBaseObject) from that
// same resolved locator, so HasBinding/[[HasProperty]] runs exactly once.
with (
  {
    fn() {
      return this;
    },
  }
) {
  fn();
}
