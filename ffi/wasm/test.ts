import init from "./pkg/boa_wasm.js";

const wasm = await init();
console.log("WASM loaded:", wasm);