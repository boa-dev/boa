// Regression test for the try/finally jump table (see PR #5381 / issue #5369).
// A `break`/`continue` that is syntactically present but not executed inside a
// `try` whose `finally` runs must fall through past the jump table instead of
// being taken once the finally completes. The emitted table reserves entry 0
// for this fallthrough case.
let total = 0;
for (let i = 0; i < 5; ++i) {
  try {
    if (i === 2) continue;
    if (i === 4) break;
  } finally {
    total += i;
  }
  total += 100;
}
total;
