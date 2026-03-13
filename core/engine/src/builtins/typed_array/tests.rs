use crate::{TestAction, run_test_actions};

#[test]
fn uint8array_constructor_length() {
    run_test_actions([
        TestAction::run("let a = new Uint8Array(4);"),
        TestAction::assert_eq("a.length", 4),
        TestAction::assert_eq("a.byteLength", 4),
    ]);
}

#[test]
fn uint8array_constructor_from_array() {
    run_test_actions([
        TestAction::run("let a = new Uint8Array([1, 2, 3]);"),
        TestAction::assert_eq("a.length", 3),
        TestAction::assert_eq("a[1]", 2),
    ]);
}

#[test]
fn uint8array_constructor_from_array_buffer() {
    run_test_actions([
        TestAction::run("let buffer = new ArrayBuffer(4); let a = new Uint8Array(buffer);"),
        TestAction::assert_eq("a.length", 4),
    ]);
}

#[test]
fn uint8array_read_write_semantics() {
    run_test_actions([
        TestAction::run("let a = new Uint8Array(2); a[0] = 42;"),
        TestAction::assert_eq("a[0]", 42),
    ]);
}

#[test]
fn uint8array_out_of_bounds_behavior() {
    run_test_actions([
        TestAction::run("let a = new Uint8Array(2);"),
        TestAction::assert("a[10] === undefined"),
        TestAction::run("a[10] = 5;"),
        TestAction::assert_eq("a.length", 2),
    ]);
}

#[test]
fn uint8array_numeric_coercion() {
    run_test_actions([
        TestAction::run("let a = new Uint8Array(1); a[0] = 256;"),
        TestAction::assert_eq("a[0]", 0),
        TestAction::run("a[0] = -1;"),
        TestAction::assert_eq("a[0]", 255),
    ]);
}

#[test]
fn float32array_storage() {
    run_test_actions([
        TestAction::run("let f = new Float32Array(1); f[0] = 1.5;"),
        TestAction::assert_eq("f[0]", 1.5),
    ]);
}

#[test]
fn float32array_nan_behavior() {
    run_test_actions([
        TestAction::run("let f = new Float32Array(1); f[0] = 'hello';"),
        TestAction::assert("Number.isNaN(f[0])"),
    ]);
}

#[test]
fn float32_precision_behavior() {
    run_test_actions([
        TestAction::run("let f = new Float32Array(1); f[0] = 1.337;"),
        TestAction::assert("Math.fround(1.337) === f[0]"),
    ]);
}

#[test]
fn typedarray_prototype_set() {
    run_test_actions([
        TestAction::run("let a = new Uint8Array(4); a.set([1, 2], 1);"),
        TestAction::assert_eq("a[1]", 1),
        TestAction::assert_eq("a[2]", 2),
    ]);
}

#[test]
fn typedarray_prototype_fill() {
    run_test_actions([
        TestAction::run("let a = new Uint8Array(3); a.fill(7);"),
        TestAction::assert_eq("a[2]", 7),
    ]);
}

#[test]
fn typedarray_prototype_subarray_shared_memory() {
    run_test_actions([
        TestAction::run(
            "
        let a = new Uint8Array([1, 2, 3, 4]);
        let b = a.subarray(1, 3);
        b[0] = 99;
        ",
        ),
        TestAction::assert_eq("a[1]", 99),
        TestAction::assert_eq("b[0]", 99),
    ]);
}

#[test]
fn typedarray_constructor_from_typedarray_number_casts() {
    run_test_actions([
        TestAction::run(
            "
            let a = new Int32Array(new Uint8Array([1, 255]));
            let b = new Uint8Array(new Int16Array([-1, 256]));
            let c = new Uint8ClampedArray(new Float32Array([-10, 12.5, 13.5, 300, NaN]));
            ",
        ),
        TestAction::assert_eq("a[0]", 1),
        TestAction::assert_eq("a[1]", 255),
        TestAction::assert_eq("b[0]", 255),
        TestAction::assert_eq("b[1]", 0),
        TestAction::assert_eq("c[0]", 0),
        TestAction::assert_eq("c[1]", 12),
        TestAction::assert_eq("c[2]", 14),
        TestAction::assert_eq("c[3]", 255),
        TestAction::assert_eq("c[4]", 0),
    ]);
}

#[test]
fn typedarray_constructor_from_typedarray_bigint_casts() {
    run_test_actions([
        TestAction::run(
            "
            let a = new BigUint64Array(new BigInt64Array([-1n, 2n]));
            let b = new BigInt64Array(new BigUint64Array([18446744073709551615n, 2n]));
            ",
        ),
        TestAction::assert("a[0] === 18446744073709551615n"),
        TestAction::assert("a[1] === 2n"),
        TestAction::assert("b[0] === -1n"),
        TestAction::assert("b[1] === 2n"),
    ]);
}
