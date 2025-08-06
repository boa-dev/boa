class SomeClass {
    constructor() {
        this.a = 42n;
        this.x = 1;
        this.repeat = this;
        this.repeatArray = [this, this, this];
    }
}

const buffer
    = new Uint8Array([1, 2, 3, 4]);
const buffer2 = new Uint8Array([5, 6, 7, 8]);
const buffer3 = new Uint32Array([9, 10, 11, 12]);
const buffer4 = new Uint32Array([13, 14, 15, 16]);

const original = {
    some: new SomeClass(),
    buffer,
    bufferTArray: [buffer, buffer, buffer, buffer],
    buffer2,
    buffer2Array: [buffer2, buffer2, buffer2, buffer2],
    buffer3,
    buffer4,
};

let dolly = structuredClone(original, {transfer: [buffer, buffer3]})

assertEq(buffer.byteLength, 0);
assertEq(buffer2.byteLength, 4);
assertEq(buffer3.byteLength, 0);
assertThrows(() => {
    new Uint8Array(buffer);
});
assertThrows(() => {
    new Uint32Array(buffer3);
});

assertEq(dolly.buffer.constructor, Uint8Array);
assertEq(dolly.buffer.byteLength, 4);
assertEq(dolly.buffer2.constructor, Uint8Array);
assertEq(dolly.buffer2.byteLength, 4);
assertEq(dolly.buffer3.constructor, Uint32Array);
assertEq(dolly.buffer3.byteLength, 16);
assertEq(dolly.buffer4.constructor, Uint32Array);
assertEq(dolly.buffer4.byteLength, 16);

assertArrayEqual(dolly.buffer, [1, 2, 3, 4]);
assertEq(dolly.buffer, dolly.bufferTArray[0]);
assertEq(dolly.bufferTArray[0], dolly.bufferTArray[1]);
assertEq(dolly.bufferTArray[0], dolly.bufferTArray[2]);
assertEq(dolly.bufferTArray[0], dolly.bufferTArray[3]);

assertArrayEqual(dolly.buffer2, [5, 6, 7, 8]);
assertEq(dolly.buffer2Array[0], dolly.buffer2Array[1]);
assertEq(dolly.buffer2Array[0], dolly.buffer2Array[2]);
assertEq(dolly.buffer2Array[0], dolly.buffer2Array[3]);

assertArrayEqual(dolly.buffer3, [9, 10, 11, 12]);
assertArrayEqual(dolly.buffer4, [13, 14, 15, 16]);

assertNEq(dolly.some.constructor, SomeClass);
assertEq(dolly.some.a, 42n);
assertEq(dolly.some.x, 1);
assertEq(dolly.some.repeat, dolly.some);
