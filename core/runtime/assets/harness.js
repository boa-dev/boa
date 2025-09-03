function assert(cond, message) {
    if (!cond) {
        throw `AssertionError: ${message ? message + ", " : ""}condition was falsy`;
    }
}

function assertEq(lhs, rhs, message) {
    if (lhs !== rhs) {
        throw `AssertionError: ${message ? message + ", " : ""}expected ${JSON.stringify(rhs)}, actual ${JSON.stringify(lhs)}`;
    }
}

function assertNEq(lhs, rhs, message) {
    if (lhs === rhs) {
        throw `AssertionError: ${message ? message + ", " : ""}expected ${JSON.stringify(rhs)}, actual ${JSON.stringify(lhs)}`;
    }
}

function assertArrayEqual(lhs, rhs, message) {
    if (lhs === rhs) {
        return;
    }

    // Supports iterables.
    let l;
    try {
        l = [...lhs];
    } catch (e) {
        throw `AssertionError: ${message ? message + ", " : ""}expected an iterable, actual isn't.`;
    }
    const r = [...rhs];

    if (l.length === r.length) {
        for (let i = 0; i < l.length; i++) {
            assertEq(l[i], r[i], message);
        }
        return;
    }

    throw `AssertionError: ${message ? message + ", " : ""}expected ${JSON.stringify(rhs)}, actual ${JSON.stringify(lhs)}`;
}

function assertThrows(fn, message) {
    try {
        fn();
    } catch (e) {
        return;
    }
    throw `AssertionError: ${message ? message + ", " : ""}function did not throw.`;
}
