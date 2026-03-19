// Creates a deep acyclic object to measure linear vs O(1) cycle detection cost.
// Depth of 400 is large enough to show the benefit while avoiding stack overflow.

function createDeepObject(depth) {
    let root = {};
    let cur = root;
    for (let i = 0; i < depth; i++) {
        cur.value = i;
        cur.next = {};
        cur = cur.next;
    }
    return root;
}

const deepObj = createDeepObject(400);

function main() {
    return JSON.stringify(deepObj);
}
