//! Tests for explicit resource management (using declarations).
//!
//! This module tests the core disposal mechanism for `using` declarations,
//! verifying that resources are properly disposed when scopes exit.

#![allow(unused_crate_dependencies)]

use boa_engine::{Context, JsValue, Source};

#[test]
fn basic_disposal() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        r"
        let disposed = false;
        {
            using x = {
                [Symbol.dispose]() {
                    disposed = true;
                }
            };
        }
        disposed;
        ",
    ));

    if let Err(ref e) = result {
        eprintln!("Error: {e:?}");
    }
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(true));
}

#[test]
fn disposal_order() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        r"
        let order = [];
        {
            using a = {
                [Symbol.dispose]() {
                    order.push('a');
                }
            };
            using b = {
                [Symbol.dispose]() {
                    order.push('b');
                }
            };
        }
        order.join(',');
        ",
    ));

    assert!(result.is_ok());
    let value = result.unwrap();
    // Should dispose in reverse order: b, then a
    assert_eq!(value.to_string(&mut context).unwrap(), "b,a");
}

#[test]
fn null_undefined_disposal() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        r"
        {
            using x = null;
            using y = undefined;
        }
        'ok';
        ",
    ));

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.to_string(&mut context).unwrap(), "ok");
}

#[test]
fn disposal_with_no_method() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        r"
        {
            using x = {
                // No Symbol.dispose method
            };
        }
        'ok';
        ",
    ));

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.to_string(&mut context).unwrap(), "ok");
}

#[test]
fn disposal_on_exception() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        r"
        let disposed = false;
        try {
            using x = {
                [Symbol.dispose]() {
                    disposed = true;
                }
            };
            throw new Error('test error');
        } catch (e) {
            // Disposal should happen before catch
        }
        disposed;
        ",
    ));

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(true));
}

#[test]
fn nested_scopes() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        r"
        let order = [];
        {
            using a = {
                [Symbol.dispose]() {
                    order.push('a');
                }
            };
            {
                using b = {
                    [Symbol.dispose]() {
                        order.push('b');
                    }
                };
            }
            // b should be disposed here
            order.push('middle');
        }
        // a should be disposed here
        order.join(',');
        ",
    ));

    assert!(result.is_ok());
    let value = result.unwrap();
    // Should dispose b first, then a
    assert_eq!(value.to_string(&mut context).unwrap(), "b,middle,a");
}

#[test]
fn multiple_resources_in_one_declaration() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        r"
        let order = [];
        {
            using a = {
                [Symbol.dispose]() {
                    order.push('a');
                }
            }, b = {
                [Symbol.dispose]() {
                    order.push('b');
                }
            };
        }
        order.join(',');
        ",
    ));

    assert!(result.is_ok());
    let value = result.unwrap();
    // Should dispose in reverse order: b, then a
    assert_eq!(value.to_string(&mut context).unwrap(), "b,a");
}

#[test]
fn disposal_on_return() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        r"
        let disposed = false;
        function test() {
            using x = {
                [Symbol.dispose]() {
                    disposed = true;
                }
            };
            return 'early';
        }
        test();
        // Return the disposed flag
        disposed;
        ",
    ));

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(true));
}

#[test]
fn disposal_on_break() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        r"
        let disposed = false;
        while (true) {
            using x = {
                [Symbol.dispose]() {
                    disposed = true;
                }
            };
            break;
        }
        disposed;
        ",
    ));

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(true));
}

#[test]
fn disposal_on_continue() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        r"
        let disposed = false;
        let count = 0;
        while (count < 2) {
            count++;
            using x = {
                [Symbol.dispose]() {
                    disposed = true;
                }
            };
            continue;
        }
        disposed;
        ",
    ));

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(true));
}
