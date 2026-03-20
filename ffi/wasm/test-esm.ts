import init from "./pkg/boa_wasm.js";

const boa: any = await init();

const result = boa.evaluate("1 + 2");

console.log("Type of result:", typeof result);
