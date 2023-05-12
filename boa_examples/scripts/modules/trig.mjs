import { sum, mult, sqrt } from "./operations.mjs";

function pyth(a, b) {
    let a2 = mult(a, a);
    let b2 = mult(b, b);
    let a2b2 = sum(a2, b2);

    return sqrt(a2b2);
}

export { pyth };