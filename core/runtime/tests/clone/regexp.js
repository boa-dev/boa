{
  const re1 = new RegExp("abc", "i");
  const re2 = /abc/i;

  // Make sure the world makes sense.
  assertNEq(re1, re2, "Two different regexp objects should not be equal");

  const x = {
    first: re1,
    second: re1,
    third: re2,
    fourth: re2,
  };

  const xx = structuredClone(x);

  assertEq(xx.first, xx.second, "These should be the same object.");
  assertNEq(
    xx.second,
    xx.third,
    "Second and Third should NOT be the same object.",
  );
  assertEq(xx.third, xx.fourth, "These should be the same object.");

  assert(xx.first.test("def ABC def"));
  assertEq(
    Object.getPrototypeOf(xx.first),
    Object.getPrototypeOf(xx.second),
    "These should be have the same prototype.",
  );
}
