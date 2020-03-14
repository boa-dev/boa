use crate::{exec::Executor, forward, forward_val, realm::Realm};
use std::f64;

#[test]
fn abs() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.abs(3 - 5);
        var b = Math.abs(1.23456 - 7.89012);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();

    assert_eq!(a.to_num(), f64::from(2.0));
    assert_eq!(b.to_num(), f64::from(6.655_559_999_999_999_5));
}

#[test]
fn acos() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.acos(8 / 10);
        var b = Math.acos(5 / 3);
        var c = Math.acos(1);
        var d = Math.acos(2);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward(&mut engine, "b");
    let c = forward_val(&mut engine, "c").unwrap();
    let d = forward(&mut engine, "d");

    assert_eq!(a.to_num(), f64::from(0.643_501_108_793_284_3));
    assert_eq!(b, String::from("NaN"));
    assert_eq!(c.to_num(), f64::from(0));
    assert_eq!(d, String::from("NaN"));
}

#[test]
fn acosh() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.acosh(2);
        var b = Math.acosh(-1);
        var c = Math.acosh(0.5);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward(&mut engine, "b");
    let c = forward(&mut engine, "c");

    assert_eq!(a.to_num(), f64::from(1.316_957_896_924_816_6));
    assert_eq!(b, String::from("NaN"));
    assert_eq!(c, String::from("NaN"));
}

#[test]
fn asin() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.asin(6 / 10);
        var b = Math.asin(5 / 3);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward(&mut engine, "b");

    assert_eq!(a.to_num(), f64::from(0.643_501_108_793_284_4));
    assert_eq!(b, String::from("NaN"));
}

#[test]
fn asinh() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.asinh(1);
        var b = Math.asinh(0);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();

    assert_eq!(a.to_num(), f64::from(0.881_373_587_019_542_9));
    assert_eq!(b.to_num(), f64::from(0));
}

#[test]
fn atan() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.atan(1);
        var b = Math.atan(0);
        var c = Math.atan(-0);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward_val(&mut engine, "c").unwrap();

    assert_eq!(a.to_num(), f64::from(0.785_398_163_397_448_3));
    assert_eq!(b.to_num(), f64::from(0));
    assert_eq!(c.to_num(), f64::from(-0));
}

#[test]
fn atan2() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.atan2(90, 15);
        var b = Math.atan2(15, 90);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();

    assert_eq!(a.to_num(), f64::from(1.405_647_649_380_269_9));
    assert_eq!(b.to_num(), f64::from(0.165_148_677_414_626_83));
}

#[test]
fn cbrt() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.cbrt(64);
        var b = Math.cbrt(-1);
        var c = Math.cbrt(1);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward_val(&mut engine, "c").unwrap();

    assert_eq!(a.to_num(), f64::from(4));
    assert_eq!(b.to_num(), f64::from(-1));
    assert_eq!(c.to_num(), f64::from(1));
}

#[test]
fn ceil() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.ceil(1.95);
        var b = Math.ceil(4);
        var c = Math.ceil(-7.004);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward_val(&mut engine, "c").unwrap();

    assert_eq!(a.to_num(), f64::from(2));
    assert_eq!(b.to_num(), f64::from(4));
    assert_eq!(c.to_num(), f64::from(-7));
}

#[test]
fn cos() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.cos(0);
        var b = Math.cos(1);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();

    assert_eq!(a.to_num(), f64::from(1));
    assert_eq!(b.to_num(), f64::from(0.540_302_305_868_139_8));
}

#[test]
fn cosh() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.cosh(0);
        var b = Math.cosh(1);
        var c = Math.cosh(-1);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward_val(&mut engine, "c").unwrap();

    assert_eq!(a.to_num(), f64::from(1));
    assert_eq!(b.to_num(), f64::from(1.543_080_634_815_243_7));
    assert_eq!(c.to_num(), f64::from(1.543_080_634_815_243_7));
}

#[test]
fn exp() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.exp(0);
        var b = Math.exp(-1);
        var c = Math.exp(2);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward_val(&mut engine, "c").unwrap();

    assert_eq!(a.to_num(), f64::from(1));
    assert_eq!(b.to_num(), f64::from(0.367_879_441_171_442_33));
    assert_eq!(c.to_num(), f64::from(7.389_056_098_930_65));
}

#[test]
fn floor() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.floor(1.95);
        var b = Math.floor(-3.01);
        var c = Math.floor(3.01);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward_val(&mut engine, "c").unwrap();

    assert_eq!(a.to_num(), f64::from(1));
    assert_eq!(b.to_num(), f64::from(-4));
    assert_eq!(c.to_num(), f64::from(3));
}

#[test]
fn log() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.log(1);
        var b = Math.log(10);
        var c = Math.log(-1);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward(&mut engine, "c");

    assert_eq!(a.to_num(), f64::from(0));
    assert_eq!(b.to_num(), f64::from(2.302_585_092_994_046));
    assert_eq!(c, String::from("NaN"));
}

#[test]
fn log10() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.log10(2);
        var b = Math.log10(1);
        var c = Math.log10(-2);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward(&mut engine, "c");

    assert_eq!(a.to_num(), f64::from(0.301_029_995_663_981_2));
    assert_eq!(b.to_num(), f64::from(0));
    assert_eq!(c, String::from("NaN"));
}

#[test]
fn log2() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.log2(3);
        var b = Math.log2(1);
        var c = Math.log2(-2);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward(&mut engine, "c");

    assert_eq!(a.to_num(), f64::from(1.584_962_500_721_156));
    assert_eq!(b.to_num(), f64::from(0));
    assert_eq!(c, String::from("NaN"));
}

#[test]
fn max() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.max(10, 20);
        var b = Math.max(-10, -20);
        var c = Math.max(-10, 20); 
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward_val(&mut engine, "c").unwrap();

    assert_eq!(a.to_num(), f64::from(20));
    assert_eq!(b.to_num(), f64::from(-10));
    assert_eq!(c.to_num(), f64::from(20));
}

#[test]
fn min() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.min(10, 20);
        var b = Math.min(-10, -20);
        var c = Math.min(-10, 20); 
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward_val(&mut engine, "c").unwrap();

    assert_eq!(a.to_num(), f64::from(10));
    assert_eq!(b.to_num(), f64::from(-20));
    assert_eq!(c.to_num(), f64::from(-10));
}

#[test]
fn pow() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.pow(2, 10);
        var b = Math.pow(-7, 2);
        var c = Math.pow(4, 0.5);
        var d = Math.pow(7, -2);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward_val(&mut engine, "c").unwrap();
    let d = forward_val(&mut engine, "d").unwrap();

    assert_eq!(a.to_num(), f64::from(1024));
    assert_eq!(b.to_num(), f64::from(49));
    assert_eq!(c.to_num(), f64::from(2.0));
    assert_eq!(d.to_num(), f64::from(0.020_408_163_265_306_12));
}

#[test]
fn round() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.round(20.5);
        var b = Math.round(-20.3);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();

    assert_eq!(a.to_num(), f64::from(21.0));
    assert_eq!(b.to_num(), f64::from(-20.0));
}

#[test]
fn sign() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.sign(3);
        var b = Math.sign(-3);
        var c = Math.sign(0); 
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward_val(&mut engine, "c").unwrap();

    assert_eq!(a.to_num(), f64::from(1));
    assert_eq!(b.to_num(), f64::from(-1));
    assert_eq!(c.to_num(), f64::from(0));
}

#[test]
fn sin() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.sin(0);
        var b = Math.sin(1);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();

    assert_eq!(a.to_num(), f64::from(0));
    assert_eq!(b.to_num(), f64::from(0.841_470_984_807_896_5));
}

#[test]
fn sinh() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.sinh(0);
        var b = Math.sinh(1);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();

    assert_eq!(a.to_num(), f64::from(0));
    assert_eq!(b.to_num(), f64::from(1.175_201_193_643_801_4));
}

#[test]
fn sqrt() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.sqrt(0);
        var b = Math.sqrt(2);
        var c = Math.sqrt(9);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();
    let c = forward_val(&mut engine, "c").unwrap();

    assert_eq!(a.to_num(), f64::from(0));
    assert_eq!(b.to_num(), f64::from(1.414_213_562_373_095_1));
    assert_eq!(c.to_num(), f64::from(3));
}

// TODO: Precision is always off between ci and local. We proably need a better way to compare floats anyways

// #[test]
// fn tan() {
//     let realm = Realm::create();
//     let mut engine = Executor::new(realm);
//     let init = r#"
//         var a = Math.tan(1.1);
//         "#;

//     forward(&mut engine, init);

//     let a = forward_val(&mut engine, "a").unwrap();

//     assert_eq!(a.to_num(), f64::from(1.964_759_657_248_652_5));
// }

#[test]
fn tanh() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.tanh(1);
        var b = Math.tanh(0);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();

    assert_eq!(a.to_num(), f64::from(0.761_594_155_955_764_9));
    assert_eq!(b.to_num(), f64::from(0));
}

#[test]
fn trunc() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = Math.trunc(13.37);
        var b = Math.trunc(0.123);
        "#;

    forward(&mut engine, init);

    let a = forward_val(&mut engine, "a").unwrap();
    let b = forward_val(&mut engine, "b").unwrap();

    assert_eq!(a.to_num(), f64::from(13));
    assert_eq!(b.to_num(), f64::from(0));
}
