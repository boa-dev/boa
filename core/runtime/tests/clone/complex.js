class SomeClass {
    constructor() {
        this.a = 42n;
        this.x = 1;
        this.repeat = this;
        this.repeatArray = [this, this, this];
    }
}

const buffer = new Uint8Array([1, 2, 3, 4]);
const buffer2 = new Uint8Array([5, 6, 7, 8]);

const original = {
    some: new SomeClass(),
    buffer,
    bufferArray: [buffer, buffer, buffer, buffer],
    buffer2,
    buffer2Array: [buffer2, buffer2, buffer2, buffer2],
};

let dolly = structuredClone(original, {transfer: [buffer]})

assertEq(buffer.byteLength, 0);
assertEq(buffer2.byteLength, 4);
assertThrows(() => {
    new Int32Array(buffer);
});
assertEq(dolly.buffer2.byteLength, 4);
assertArrayEqual(dolly.buffer, [1, 2, 3, 4]);
assertArrayEqual(dolly.bufferArray[0], [1, 2, 3, 4]);

assertEq(dolly.bufferArray[0], dolly.bufferArray[1]);
assertEq(dolly.bufferArray[0], dolly.bufferArray[2]);
assertEq(dolly.bufferArray[0], dolly.bufferArray[3]);

assertEq(dolly.buffer, dolly.bufferArray[0]);
assertArrayEqual(dolly.buffer2, [5, 6, 7, 8]);
assertEq(dolly.buffer2Array[0], dolly.buffer2Array[1]);
assertEq(dolly.buffer2Array[0], dolly.buffer2Array[2]);
assertEq(dolly.buffer2Array[0], dolly.buffer2Array[3]);
