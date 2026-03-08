// test_crash.js
const adapter = await navigator.gpu.requestAdapter();
const device = await adapter.requestDevice();
const input = new Float32Array([1, 3, 5]);

console.log("input =", input);
console.log("input instanceof Float32Array =", input instanceof Float32Array);

const workBuffer = device.createBuffer({
  size: input.byteLength,
 // usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
});

console.log("workBuffer =", workBuffer);
console.log("workBuffer instanceof GPUBuffer =", workBuffer instanceof GPUBuffer);

device.queue.writeBuffer(workBuffer, 0, input);
console.log("writeBuffer success");
console.log("end");
