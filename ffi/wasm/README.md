# @boa-dev/boa_wasm

WebAssembly bindings for [Boa](https://github.com/boa-dev/boa), a Javascript engine written in Rust.

[![npm version](https://img.shields.io/npm/v/@boa-dev/boa_wasm)](https://www.npmjs.com/package/@boa-dev/boa_wasm)
[![license](https://img.shields.io/npm/l/@boa-dev/boa_wasm)](https://github.com/boa-dev/boa)

## Overview

This package provides WebAssembly bindings to run JavaScript code using the Boa engine directly in the browser or Node.js environments. Boa supports **more than 90%** of the latest ECMAScript specification.

## Installation

```bash
npm install @boa-dev/boa_wasm
```

Or with yarn:

```bash
yarn add @boa-dev/boa_wasm
```

## Usage

### Browser

```javascript
import init, { evaluate } from "@boa-dev/boa_wasm";

// Initialize the WASM module
await init();

// Evaluate JavaScript code
try {
  const result = evaluate("1 + 1");
  console.log(result); // "2"
} catch (error) {
  console.error("Evaluation error:", error);
}
```

### Node.js

```javascript
const { evaluate } = require("@boa-dev/boa_wasm");

try {
  const result = evaluate("1 + 1");
  console.log(result); // "2"
} catch (error) {
  console.error("Evaluation error:", error);
}
```

### Advanced Example

```javascript
import init, { evaluate } from "@boa-dev/boa_wasm";

await init();

const code = `
  function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
  }
  
  fibonacci(10)
`;

const result = evaluate(code);
console.log(result); // "55"
```

## API

### `evaluate(src: string): string`

Evaluates the given ECMAScript code and returns the result as a string.

**Parameters:**

- `src` - A string containing the JavaScript code to evaluate

**Returns:**

- A string representation of the evaluation result

**Throws:**

- A `JsValue` error if the execution throws an exception

## Features

Boa's WASM build includes:

- **Annex B**: Legacy ECMAScript features
- **Internationalization**: Full Intl API support
- **Experimental features**: Latest ECMAScript proposals

## Live Demo

Try Boa in your browser at our [live playground](https://boajs.dev/playground)!

## Conformance

Check out Boa's [ECMAScript Test262 conformance results](https://boajs.dev/conformance) to see our progress on implementing the ECMAScript specification.

## Documentation

- [Boa Repository](https://github.com/boa-dev/boa)
- [Boa Engine API Documentation](https://docs.rs/boa_engine/latest/boa_engine/)
- [Boa Website](https://boajs.dev/)

## Contributing

Contributions are welcome! Please check out the [contributing guide](https://github.com/boa-dev/boa/blob/main/CONTRIBUTING.md) to get started.

## Communication

- [Matrix](https://matrix.to/#/#boa:matrix.org)
- [Discord](https://discord.gg/tUFFk9Y)

## License

This project is licensed under the [Unlicense](https://github.com/boa-dev/boa/blob/main/LICENSE-UNLICENSE) or [MIT](https://github.com/boa-dev/boa/blob/main/LICENSE-MIT) licenses, at your option.
