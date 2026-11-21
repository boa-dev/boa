// Measures String.prototype.replace performance: substring search
// and per-replacement allocation cost on short strings.
const kIterationCount = 10_000;
const templates = [
  "the quick brown fox jumps over the lazy dog",
  "a fast red fox leaps across the slow cat",
  "one small step for man one giant leap for mankind",
];
const replacements = [
  ["fox", "cat"],
  ["quick", "slow"],
  ["the", "a"],
];

function main() {
  let result = "";
  for (let i = 0; i < kIterationCount; i++) {
    let s = templates[i % templates.length];
    const pair = replacements[i % replacements.length];
    s = s.replace(pair[0], pair[1]);
    result = s;
  }
  return result.length;
}
