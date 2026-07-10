class SomeClass {}

[...new Array(100_000).map(() => new SomeClass())].length;
