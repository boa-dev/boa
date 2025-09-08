{
  const date1 = new Date();
  const date2 = new Date(date1.getTime());

  // Make sure the world makes sense.
  assertNEq(date1, date2, "Two different date objects should not be equal");

  const x = {
    first: date1,
    second: date1,
    third: date2,
  };

  const xx = structuredClone(x);

  assertEq(xx.first, xx.second, "These should be the same object.");
  assertNEq(
    xx.second,
    xx.third,
    "Second and Third should NOT be the same object.",
  );
}
