// Measures property resolution through prototype chains of varying
// depth (1, 5, and 10 levels) to quantify chain-walking overhead.
const kIterationCount = 100_000;

// Build a prototype chain of the given depth with a base property.
function buildChain(depth) {
  let proto = { baseValue: 42, type: "root" };
  for (let i = 0; i < depth; i++) {
    const child = Object.create(proto);
    child["level" + i] = i;
    child["name" + i] = "node_" + i;
    proto = child;
  }
  return proto;
}

const shallow = buildChain(1);
const medium = buildChain(5);
const deep = buildChain(10);

function main() {
  let sum = 0;

  // Shallow chain: 1 prototype hop to reach baseValue.
  for (let i = 0; i < kIterationCount; i++) {
    sum += shallow.baseValue;
  }

  // Medium chain: 5 prototype hops.
  for (let i = 0; i < kIterationCount; i++) {
    sum += medium.baseValue;
  }

  // Deep chain: 10 prototype hops.
  for (let i = 0; i < kIterationCount; i++) {
    sum += deep.baseValue;
  }

  return sum;
}
