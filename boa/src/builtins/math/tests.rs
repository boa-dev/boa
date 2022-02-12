#![allow(clippy::float_cmp)]

use crate::{forward, forward_val, Context};
use std::f64;

#[test]
fn abs() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.abs(3 - 5);
        var b = Math.abs(1.23456 - 7.89012);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 2.0);
    assert_eq!(b.to_number(&mut context).unwrap(), 6.655_559_999_999_999_5);
}

#[test]
fn acos() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.acos(8 / 10);
        var b = Math.acos(5 / 3);
        var c = Math.acos(1);
        var d = Math.acos(2);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward(&mut context, "b");
    let c = forward_val(&mut context, "c").unwrap();
    let d = forward(&mut context, "d");

    assert_eq!(a.to_number(&mut context).unwrap(), 0.643_501_108_793_284_3);
    assert_eq!(b, "NaN");
    assert_eq!(c.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(d, "NaN");
}

#[test]
fn acosh() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.acosh(2);
        var b = Math.acosh(-1);
        var c = Math.acosh(0.5);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward(&mut context, "b");
    let c = forward(&mut context, "c");

    assert_eq!(a.to_number(&mut context).unwrap(), 1.316_957_896_924_816_6);
    assert_eq!(b, "NaN");
    assert_eq!(c, "NaN");
}

#[test]
fn asin() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.asin(6 / 10);
        var b = Math.asin(5 / 3);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward(&mut context, "b");

    assert_eq!(a.to_number(&mut context).unwrap(), 0.643_501_108_793_284_4);
    assert_eq!(b, String::from("NaN"));
}

#[test]
fn asinh() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.asinh(1);
        var b = Math.asinh(0);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 0.881_373_587_019_542_9);
    assert_eq!(b.to_number(&mut context).unwrap(), 0_f64);
}

#[test]
fn atan() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.atan(1);
        var b = Math.atan(0);
        var c = Math.atan(-0);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), f64::consts::FRAC_PI_4);
    assert_eq!(b.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(c.to_number(&mut context).unwrap(), f64::from(-0));
}

#[test]
fn atan2() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.atan2(90, 15);
        var b = Math.atan2(15, 90);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 1.405_647_649_380_269_9);
    assert_eq!(b.to_number(&mut context).unwrap(), 0.165_148_677_414_626_83);
}

#[test]
fn cbrt() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.cbrt(64);
        var b = Math.cbrt(-1);
        var c = Math.cbrt(1);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 4_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), -1_f64);
    assert_eq!(c.to_number(&mut context).unwrap(), 1_f64);
}

#[test]
fn ceil() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.ceil(1.95);
        var b = Math.ceil(4);
        var c = Math.ceil(-7.004);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 2_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), 4_f64);
    assert_eq!(c.to_number(&mut context).unwrap(), -7_f64);
}

#[test]
#[allow(clippy::many_single_char_names)]
fn clz32() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.clz32();
        var b = Math.clz32({});
        var c = Math.clz32(-173);
        var d = Math.clz32("1");
        var e = Math.clz32(2147483647);
        var f = Math.clz32(Infinity);
        var g = Math.clz32(true);
        var h = Math.clz32(0);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();
    let d = forward_val(&mut context, "d").unwrap();
    let e = forward_val(&mut context, "e").unwrap();
    let f = forward_val(&mut context, "f").unwrap();
    let g = forward_val(&mut context, "g").unwrap();
    let h = forward_val(&mut context, "h").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 32_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), 32_f64);
    assert_eq!(c.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(d.to_number(&mut context).unwrap(), 31_f64);
    assert_eq!(e.to_number(&mut context).unwrap(), 1_f64);
    assert_eq!(f.to_number(&mut context).unwrap(), 32_f64);
    assert_eq!(g.to_number(&mut context).unwrap(), 31_f64);
    assert_eq!(h.to_number(&mut context).unwrap(), 32_f64);
}

#[test]
fn cos() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.cos(0);
        var b = Math.cos(1);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 1_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), 0.540_302_305_868_139_8);
}

#[test]
fn cosh() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.cosh(0);
        var b = Math.cosh(1);
        var c = Math.cosh(-1);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 1_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), 1.543_080_634_815_243_7);
    assert_eq!(c.to_number(&mut context).unwrap(), 1.543_080_634_815_243_7);
}

#[test]
fn exp() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.exp(0);
        var b = Math.exp(-1);
        var c = Math.exp(2);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 1_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), 0.367_879_441_171_442_33);
    assert_eq!(c.to_number(&mut context).unwrap(), 7.389_056_098_930_65);
}

#[test]
#[allow(clippy::many_single_char_names)]
fn expm1() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.expm1();
        var b = Math.expm1({});
        var c = Math.expm1(1);
        var d = Math.expm1(-1);
        var e = Math.expm1(0);
        var f = Math.expm1(2);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward(&mut context, "a");
    let b = forward(&mut context, "b");
    let c = forward_val(&mut context, "c").unwrap();
    let d = forward_val(&mut context, "d").unwrap();
    let e = forward_val(&mut context, "e").unwrap();
    let f = forward_val(&mut context, "f").unwrap();

    assert_eq!(a, String::from("NaN"));
    assert_eq!(b, String::from("NaN"));
    assert!(float_cmp::approx_eq!(
        f64,
        c.to_number(&mut context).unwrap(),
        1.718_281_828_459_045
    ));
    assert!(float_cmp::approx_eq!(
        f64,
        d.to_number(&mut context).unwrap(),
        -0.632_120_558_828_557_7
    ));
    assert!(float_cmp::approx_eq!(
        f64,
        e.to_number(&mut context).unwrap(),
        0_f64
    ));
    assert!(float_cmp::approx_eq!(
        f64,
        f.to_number(&mut context).unwrap(),
        6.389_056_098_930_65
    ));
}

#[test]
fn floor() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.floor(1.95);
        var b = Math.floor(-3.01);
        var c = Math.floor(3.01);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 1_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), -4_f64);
    assert_eq!(c.to_number(&mut context).unwrap(), 3_f64);
}

#[test]
#[allow(clippy::many_single_char_names)]
fn fround() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.fround(NaN);
        var b = Math.fround(Infinity);
        var c = Math.fround(5);
        var d = Math.fround(5.5);
        var e = Math.fround(5.05);
        var f = Math.fround(-5.05);
        var g = Math.fround();
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward(&mut context, "a");
    let b = forward(&mut context, "b");
    let c = forward_val(&mut context, "c").unwrap();
    let d = forward_val(&mut context, "d").unwrap();
    let e = forward_val(&mut context, "e").unwrap();
    let f = forward_val(&mut context, "f").unwrap();
    let g = forward(&mut context, "g");

    assert_eq!(a, String::from("NaN"));
    assert_eq!(b, String::from("Infinity"));
    assert_eq!(c.to_number(&mut context).unwrap(), 5f64);
    assert_eq!(d.to_number(&mut context).unwrap(), 5.5f64);
    assert_eq!(e.to_number(&mut context).unwrap(), 5.050_000_190_734_863);
    assert_eq!(f.to_number(&mut context).unwrap(), -5.050_000_190_734_863);
    assert_eq!(g, String::from("NaN"));
}

#[test]
#[allow(clippy::many_single_char_names)]
fn hypot() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.hypot();
        var b = Math.hypot(3, 4);
        var c = Math.hypot(5, 12);
        var d = Math.hypot(3, 4, -5);
        var e = Math.hypot(4, [5], 6);
        var f = Math.hypot(3, -Infinity);
        var g = Math.hypot(12);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();
    let d = forward_val(&mut context, "d").unwrap();
    let e = forward_val(&mut context, "e").unwrap();
    let f = forward_val(&mut context, "f").unwrap();
    let g = forward_val(&mut context, "g").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 0f64);
    assert_eq!(b.to_number(&mut context).unwrap(), 5f64);
    assert_eq!(c.to_number(&mut context).unwrap(), 13f64);
    assert_eq!(d.to_number(&mut context).unwrap(), 7.071_067_811_865_475_5);
    assert_eq!(e.to_number(&mut context).unwrap(), 8.774964387392123);
    assert!(f.to_number(&mut context).unwrap().is_infinite());
    assert_eq!(g.to_number(&mut context).unwrap(), 12f64);
}

#[test]
#[allow(clippy::many_single_char_names)]
fn imul() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.imul(3, 4);
        var b = Math.imul(-5, 12);
        var c = Math.imul(0xffffffff, 5);
        var d = Math.imul(0xfffffffe, 5);
        var e = Math.imul(12);
        var f = Math.imul();
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();
    let d = forward_val(&mut context, "d").unwrap();
    let e = forward_val(&mut context, "e").unwrap();
    let f = forward_val(&mut context, "f").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 12f64);
    assert_eq!(b.to_number(&mut context).unwrap(), -60f64);
    assert_eq!(c.to_number(&mut context).unwrap(), -5f64);
    assert_eq!(d.to_number(&mut context).unwrap(), -10f64);
    assert_eq!(e.to_number(&mut context).unwrap(), 0f64);
    assert_eq!(f.to_number(&mut context).unwrap(), 0f64);
}

#[test]
fn log() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.log(1);
        var b = Math.log(10);
        var c = Math.log(-1);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward(&mut context, "c");

    assert_eq!(a.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), f64::consts::LN_10);
    assert_eq!(c, String::from("NaN"));
}

#[test]
#[allow(clippy::many_single_char_names)]
fn log1p() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.log1p(1);
        var b = Math.log1p(0);
        var c = Math.log1p(-0.9999999999999999);
        var d = Math.log1p(-1);
        var e = Math.log1p(-1.000000000000001);
        var f = Math.log1p(-2);
        var g = Math.log1p();
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();
    let d = forward(&mut context, "d");
    let e = forward(&mut context, "e");
    let f = forward(&mut context, "f");
    let g = forward(&mut context, "g");

    assert_eq!(a.to_number(&mut context).unwrap(), f64::consts::LN_2);
    assert_eq!(b.to_number(&mut context).unwrap(), 0f64);
    assert_eq!(c.to_number(&mut context).unwrap(), -36.736_800_569_677_1);
    assert_eq!(d, "-Infinity");
    assert_eq!(e, String::from("NaN"));
    assert_eq!(f, String::from("NaN"));
    assert_eq!(g, String::from("NaN"));
}

#[test]
fn log10() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.log10(2);
        var b = Math.log10(1);
        var c = Math.log10(-2);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward(&mut context, "c");

    assert_eq!(a.to_number(&mut context).unwrap(), f64::consts::LOG10_2);
    assert_eq!(b.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(c, String::from("NaN"));
}

#[test]
fn log2() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.log2(3);
        var b = Math.log2(1);
        var c = Math.log2(-2);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward(&mut context, "c");

    assert_eq!(a.to_number(&mut context).unwrap(), 1.584_962_500_721_156);
    assert_eq!(b.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(c, String::from("NaN"));
}

#[test]
fn max() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.max(10, 20);
        var b = Math.max(-10, -20);
        var c = Math.max(-10, 20);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 20_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), -10_f64);
    assert_eq!(c.to_number(&mut context).unwrap(), 20_f64);
}

#[test]
fn min() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.min(10, 20);
        var b = Math.min(-10, -20);
        var c = Math.min(-10, 20);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 10_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), -20_f64);
    assert_eq!(c.to_number(&mut context).unwrap(), -10_f64);
}

#[test]
fn pow() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.pow(2, 10);
        var b = Math.pow(-7, 2);
        var c = Math.pow(4, 0.5);
        var d = Math.pow(7, -2);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();
    let d = forward_val(&mut context, "d").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 1_024_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), 49_f64);
    assert_eq!(c.to_number(&mut context).unwrap(), 2.0);
    assert_eq!(d.to_number(&mut context).unwrap(), 0.020_408_163_265_306_12);
}

#[test]
fn round() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.round(20.5);
        var b = Math.round(-20.3);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 21.0);
    assert_eq!(b.to_number(&mut context).unwrap(), -20.0);
}

#[test]
fn sign() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.sign(3);
        var b = Math.sign(-3);
        var c = Math.sign(0);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 1_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), -1_f64);
    assert_eq!(c.to_number(&mut context).unwrap(), 0_f64);
}

#[test]
fn sin() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.sin(0);
        var b = Math.sin(1);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), 0.841_470_984_807_896_5);
}

#[test]
fn sinh() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.sinh(0);
        var b = Math.sinh(1);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), 1.175_201_193_643_801_4);
}

#[test]
fn sqrt() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.sqrt(0);
        var b = Math.sqrt(2);
        var c = Math.sqrt(9);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();
    let c = forward_val(&mut context, "c").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), f64::consts::SQRT_2);
    assert_eq!(c.to_number(&mut context).unwrap(), 3_f64);
}

#[test]
fn tan() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.tan(1.1);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();

    assert!(float_cmp::approx_eq!(
        f64,
        a.to_number(&mut context).unwrap(),
        1.964_759_657_248_652_5
    ));
}

#[test]
fn tanh() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.tanh(1);
        var b = Math.tanh(0);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 0.761_594_155_955_764_9);
    assert_eq!(b.to_number(&mut context).unwrap(), 0_f64);
}

#[test]
fn trunc() {
    let mut context = Context::default();
    let init = r#"
        var a = Math.trunc(13.37);
        var b = Math.trunc(0.123);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let a = forward_val(&mut context, "a").unwrap();
    let b = forward_val(&mut context, "b").unwrap();

    assert_eq!(a.to_number(&mut context).unwrap(), 13_f64);
    assert_eq!(b.to_number(&mut context).unwrap(), 0_f64);
}
