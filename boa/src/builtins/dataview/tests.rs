use crate::{forward, forward_val, Context};
use num_traits::Zero;
use std::f32;
use std::f64;

#[test]
fn constructor() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);

        const view1 = new DataView(buffer);
        const view2 = new DataView(buffer, 12, 4);
        view1.setInt8(12, 42);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "view2.getInt8(0)"), "42");
    assert_eq!(forward(&mut context, "view1.getInt8(12)"), "42");
}

#[test]
fn get_buffer() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(123);
        const view = new DataView(buffer);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "view.buffer.byteLength"), "123");
}

#[test]
fn get_byte_length() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const view1 = new DataView(buffer);
        const view2 = new DataView(buffer, 12, 4);
        const dataview = new DataView(buffer);
        const dataview2 = new DataView(buffer, 1, 5);
        const dataview3 = new DataView(buffer, 2);
        "#;
    forward(&mut context, init);
    assert_eq!(
        forward(&mut context, "view1.byteLength + view2.byteLength"),
        "20"
    );
    assert_eq!(forward(&mut context, "dataview.byteLength"), "16");
    assert_eq!(forward(&mut context, "dataview2.byteLength"), "5");
    assert_eq!(forward(&mut context, "dataview3.byteLength"), "14");
}

#[test]
fn get_byte_offset() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);

        const view = new DataView(buffer, 12, 4);
        const dataview = new DataView(buffer);
        const dataview2 = new DataView(buffer, 3);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "view.byteOffset"), "12");
    assert_eq!(forward(&mut context, "dataview.byteOffset"), "0");
    assert_eq!(forward(&mut context, "dataview2.byteOffset"), "3");
}

#[test]
fn get_big_int64() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const max = 2n ** (64n - 1n) - 1n;
        const view = new DataView(buffer);
        view.setBigInt64(1, max);
        "#;
    forward(&mut context, init);
    assert_eq!(
        forward(&mut context, "view.getBigInt64(1)"),
        "9223372036854775807n"
    );
}

#[test]
fn get_big_uint64() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const max = 2n ** 64n - 1n;
        const view = new DataView(buffer);
        view.setBigUint64(1, max);
        const buffer2 = new ArrayBuffer(8);
        const view2 = new DataView(buffer2);
        "#;
    forward(&mut context, init);
    assert_eq!(
        forward(&mut context, "view.getBigUint64(1)"),
        "18446744073709551615n"
    );
    assert_eq!(forward(&mut context, "view2.getBigUint64(0)"), "0n");
}

#[test]
fn get_float32() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const view = new DataView(buffer);
        view.setFloat32(1, Math.PI);
        "#;
    forward(&mut context, init);
    let zero = forward_val(&mut context, "view.getFloat32(0)").unwrap();
    let pi = forward_val(&mut context, "view.getFloat32(1)").unwrap();
    assert!(float_cmp::approx_eq!(
        f64,
        pi.to_number(&mut context).unwrap(),
        f32::consts::PI as f64
    ));
    assert!(float_cmp::approx_eq!(
        f64,
        zero.to_number(&mut context).unwrap(),
        0_f64
    ));
}

#[test]
fn get_float64() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const view = new DataView(buffer);
        view.setFloat64(1, Math.PI);
        "#;
    forward(&mut context, init);
    let zero = forward_val(&mut context, "view.getFloat64(0)").unwrap();
    let pi = forward_val(&mut context, "view.getFloat64(1)").unwrap();
    assert!(float_cmp::approx_eq!(
        f64,
        pi.to_number(&mut context).unwrap(),
        f64::consts::PI
    ));
    assert!(float_cmp::approx_eq!(
        f64,
        zero.to_number(&mut context).unwrap(),
        0_f64
    ));
}

#[test]
fn get_int8() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const view = new DataView(buffer);
        view.setInt8(1, 127);
        "#;
    forward(&mut context, init);
    let zero = forward_val(&mut context, "view.getInt8(0)")
        .unwrap()
        .to_int8(&mut context)
        .unwrap();
    assert!(zero.is_zero());
    let max = forward_val(&mut context, "view.getInt8(1)")
        .unwrap()
        .to_int8(&mut context)
        .unwrap();
    assert_eq!(max, 127);
}

#[test]
fn get_int16() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const view = new DataView(buffer);
        view.setInt16(1, 32767);
        "#;
    forward(&mut context, init);
    let small = forward_val(&mut context, "view.getInt16(0)")
        .unwrap()
        .to_int16(&mut context)
        .unwrap();
    assert_eq!(small, 127);
    let max = forward_val(&mut context, "view.getInt16(1)")
        .unwrap()
        .to_int16(&mut context)
        .unwrap();
    assert_eq!(max, 32767);
}

#[test]
fn get_int32() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const view = new DataView(buffer);
        view.setInt32(1, 2147483647);
        "#;
    forward(&mut context, init);
    let max = forward_val(&mut context, "view.getInt32(1)")
        .unwrap()
        .to_i32(&mut context)
        .unwrap();
    assert_eq!(max, 2147483647);
}

#[test]
fn get_uint8() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const view = new DataView(buffer);
        view.setUint8(1, 255);
        "#;
    forward(&mut context, init);
    let zero = forward_val(&mut context, "view.getUint8(0)")
        .unwrap()
        .to_uint8(&mut context)
        .unwrap();
    assert!(zero.is_zero());
    let max = forward_val(&mut context, "view.getUint8(1)")
        .unwrap()
        .to_uint8(&mut context)
        .unwrap();
    assert_eq!(max, 255);
}

#[test]
fn get_uint16() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const view = new DataView(buffer);
        view.setUint16(1, 65535);
        "#;
    forward(&mut context, init);
    let small = forward_val(&mut context, "view.getUint16(0)")
        .unwrap()
        .to_uint16(&mut context)
        .unwrap();
    assert_eq!(small, 255);
    let max = forward_val(&mut context, "view.getUint16(1)")
        .unwrap()
        .to_uint16(&mut context)
        .unwrap();
    assert_eq!(max, 65535);
}

#[test]
fn get_uint32() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const view = new DataView(buffer);
        view.setUint32(1, 4294967295);
        "#;
    forward(&mut context, init);
    let max = forward_val(&mut context, "view.getUint32(1)")
        .unwrap()
        .to_u32(&mut context)
        .unwrap();
    assert_eq!(max, 4294967295);
}
