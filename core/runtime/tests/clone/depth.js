// Deeply nested but acyclic graphs must be cloned below the conversion depth
// limit, and must throw a catchable error above it instead of overflowing the
// native stack and aborting the process.
{
  function makeDeepObject(depth) {
    const root = {};
    let current = root;
    for (let i = 0; i < depth; i++) {
      current.next = {};
      current = current.next;
    }
    return root;
  }

  const shallowObject = makeDeepObject(100);
  const shallowObjectClone = structuredClone(shallowObject);
  let node = shallowObjectClone;
  for (let i = 0; i < 100; i++) {
    node = node.next;
  }
  assertEq(typeof node, "object");

  // The previous crash threshold was script-controlled, so verify that a much
  // larger depth is rejected cleanly as well.
  assertThrows(() => structuredClone(makeDeepObject(300)));
  assertThrows(() => structuredClone(makeDeepObject(10000)));

  function makeDeepArray(depth) {
    const root = [];
    let current = root;
    for (let i = 0; i < depth; i++) {
      const next = [];
      current.push(next);
      current = next;
    }
    return root;
  }

  const shallowArray = makeDeepArray(100);
  const shallowArrayClone = structuredClone(shallowArray);
  let arrayNode = shallowArrayClone;
  for (let i = 0; i < 100; i++) {
    arrayNode = arrayNode[0];
  }
  assertEq(Array.isArray(arrayNode), true);

  assertThrows(() => structuredClone(makeDeepArray(300)));
  assertThrows(() => structuredClone(makeDeepArray(10000)));

  // The engine must still be usable after the error was thrown and caught.
  const afterError = structuredClone({ ok: true });
  assertEq(afterError.ok, true);
}
