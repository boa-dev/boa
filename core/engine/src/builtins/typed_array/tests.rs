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
