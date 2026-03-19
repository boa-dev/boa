// Creates a circular structure at depth to measure efficient cycle detection.
// Catches the error to allow the benchmark to run repeatedly.

function createCircularObject(depth) {
    let root = {};
    let cur = root;
    for (let i = 0; i < depth; i++) {
        cur.next = {};
        cur = cur.next;
    }
    cur.next = root; // Create the cycle back to the root
    return root;
}

const circularObj = createCircularObject(100);

function main() {
    try {
        JSON.stringify(circularObj);
    } catch (_) {
        // Expected TypeError: cyclic object value
        return;
    }
}
