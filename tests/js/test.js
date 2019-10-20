const o1 = {};

const o2 = { ref: o1 };

o1.ref = o2;

o1;