let s = "\u{1234}--synchronized-----";
for (let i = 0; i < 30; i++) {
    try {
        s = s + s;
        console.log(`Iteration ${i}: length = ${s.length}`);
    } catch (e) {
        console.log(`Caught error at iteration ${i}: ${e}`);
        break;
    }
}