const kIterationCount = 100;
const segmenter = new Intl.Segmenter("en");
const text = "The quick brown fox jumps over the lazy dog.";

function main() {
  for (let i = 0; i < kIterationCount; i++) {
    [...segmenter.segment(text)];
  }
}
