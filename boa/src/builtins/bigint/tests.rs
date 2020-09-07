use crate::{forward, Context};

#[test]
fn equality() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "0n == 0n"), "true");
    assert_eq!(forward(&mut engine, "1n == 0n"), "false");
    assert_eq!(
        forward(
            &mut engine,
            "1000000000000000000000000000000000n == 1000000000000000000000000000000000n"
        ),
        "true"
    );

    assert_eq!(forward(&mut engine, "0n == ''"), "true");
    assert_eq!(forward(&mut engine, "100n == '100'"), "true");
    assert_eq!(forward(&mut engine, "100n == '100.5'"), "false");
    assert_eq!(
        forward(&mut engine, "10000000000000000n == '10000000000000000'"),
        "true"
    );

    assert_eq!(forward(&mut engine, "'' == 0n"), "true");
    assert_eq!(forward(&mut engine, "'100' == 100n"), "true");
    assert_eq!(forward(&mut engine, "'100.5' == 100n"), "false");
    assert_eq!(
        forward(&mut engine, "'10000000000000000' == 10000000000000000n"),
        "true"
    );

    assert_eq!(forward(&mut engine, "0n == 0"), "true");
    assert_eq!(forward(&mut engine, "0n == 0.0"), "true");
    assert_eq!(forward(&mut engine, "100n == 100"), "true");
    assert_eq!(forward(&mut engine, "100n == 100.0"), "true");
    assert_eq!(forward(&mut engine, "100n == '100.5'"), "false");
    assert_eq!(forward(&mut engine, "100n == '1005'"), "false");
    assert_eq!(
        forward(&mut engine, "10000000000000000n == 10000000000000000"),
        "true"
    );

    assert_eq!(forward(&mut engine, "0 == 0n"), "true");
    assert_eq!(forward(&mut engine, "0.0 == 0n"), "true");
    assert_eq!(forward(&mut engine, "100 == 100n"), "true");
    assert_eq!(forward(&mut engine, "100.0 == 100n"), "true");
    assert_eq!(forward(&mut engine, "100.5 == 100n"), "false");
    assert_eq!(forward(&mut engine, "1005 == 100n"), "false");
    assert_eq!(
        forward(&mut engine, "10000000000000000 == 10000000000000000n"),
        "true"
    );
}

#[test]
fn bigint_function_conversion_from_integer() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "BigInt(1000)"), "1000n");
    assert_eq!(
        forward(&mut engine, "BigInt(20000000000000000)"),
        "20000000000000000n"
    );
    assert_eq!(
        forward(&mut engine, "BigInt(1000000000000000000000000000000000)"),
        "999999999999999945575230987042816n"
    );
}

#[test]
fn bigint_function_conversion_from_rational() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "BigInt(0.0)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt(1.0)"), "1n");
    assert_eq!(forward(&mut engine, "BigInt(10000.0)"), "10000n");
}

#[test]
fn bigint_function_conversion_from_rational_with_fractional_part() {
    let mut engine = Context::new();

    let scenario = r#"
        try {
            BigInt(0.1);
        } catch (e) {
            e.toString();
        }
    "#;
    assert_eq!(
        forward(&mut engine, scenario),
        "\"TypeError: The number 0.1 cannot be converted to a BigInt because it is not an integer\""
    );
}

#[test]
fn bigint_function_conversion_from_null() {
    let mut engine = Context::new();

    let scenario = r#"
        try {
            BigInt(null);
        } catch (e) {
            e.toString();
        }
    "#;
    assert_eq!(
        forward(&mut engine, scenario),
        "\"TypeError: cannot convert null to a BigInt\""
    );
}

#[test]
fn bigint_function_conversion_from_undefined() {
    let mut engine = Context::new();

    let scenario = r#"
        try {
            BigInt(undefined);
        } catch (e) {
            e.toString();
        }
    "#;
    assert_eq!(
        forward(&mut engine, scenario),
        "\"TypeError: cannot convert undefined to a BigInt\""
    );
}

#[test]
fn bigint_function_conversion_from_string() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "BigInt('')"), "0n");
    assert_eq!(
        forward(&mut engine, "BigInt('200000000000000000')"),
        "200000000000000000n"
    );
    assert_eq!(
        forward(&mut engine, "BigInt('1000000000000000000000000000000000')"),
        "1000000000000000000000000000000000n"
    );
}

#[test]
fn add() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "10000n + 1000n"), "11000n");
}

#[test]
fn sub() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "10000n - 1000n"), "9000n");
}

#[test]
fn mul() {
    let mut engine = Context::new();

    assert_eq!(
        forward(&mut engine, "123456789n * 102030n"),
        "12596296181670n"
    );
}

#[test]
fn div() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "15000n / 50n"), "300n");
}

#[test]
fn div_with_truncation() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "15001n / 50n"), "300n");
}

#[test]
fn r#mod() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "15007n % 10n"), "7n");
}

#[test]
fn pow() {
    let mut engine = Context::new();

    assert_eq!(
        forward(&mut engine, "100n ** 10n"),
        "100000000000000000000n"
    );
}

#[test]
fn to_string() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "1000n.toString()"), "\"1000\"");
    assert_eq!(forward(&mut engine, "1000n.toString(2)"), "\"1111101000\"");
    assert_eq!(forward(&mut engine, "255n.toString(16)"), "\"ff\"");
    assert_eq!(forward(&mut engine, "1000n.toString(36)"), "\"rs\"");
}

#[test]
fn as_int_n() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "BigInt.asIntN(0, 1n)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt.asIntN(1, 1n)"), "-1n");
    assert_eq!(forward(&mut engine, "BigInt.asIntN(3, 10n)"), "2n");
    assert_eq!(forward(&mut engine, "BigInt.asIntN({}, 1n)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt.asIntN(2, 0n)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt.asIntN(2, -0n)"), "0n");

    assert_eq!(
        forward(&mut engine, "BigInt.asIntN(2, -123456789012345678901n)"),
        "-1n"
    );
    assert_eq!(
        forward(&mut engine, "BigInt.asIntN(2, -123456789012345678900n)"),
        "0n"
    );

    assert_eq!(
        forward(&mut engine, "BigInt.asIntN(2, 123456789012345678900n)"),
        "0n"
    );
    assert_eq!(
        forward(&mut engine, "BigInt.asIntN(2, 123456789012345678901n)"),
        "1n"
    );

    assert_eq!(
        forward(
            &mut engine,
            "BigInt.asIntN(200, 0xcffffffffffffffffffffffffffffffffffffffffffffffffffn)"
        ),
        "-1n"
    );
    assert_eq!(
        forward(
            &mut engine,
            "BigInt.asIntN(201, 0xcffffffffffffffffffffffffffffffffffffffffffffffffffn)"
        ),
        "1606938044258990275541962092341162602522202993782792835301375n"
    );

    assert_eq!(
        forward(
            &mut engine,
            "BigInt.asIntN(200, 0xc89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n)"
        ),
        "-741470203160010616172516490008037905920749803227695190508460n"
    );
    assert_eq!(
        forward(
            &mut engine,
            "BigInt.asIntN(201, 0xc89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n)"
        ),
        "865467841098979659369445602333124696601453190555097644792916n"
    );
}

#[test]
fn as_int_n_errors() {
    let mut engine = Context::new();

    assert_throws(&mut engine, "BigInt.asIntN(-1, 0n)", "RangeError");
    assert_throws(&mut engine, "BigInt.asIntN(-2.5, 0n)", "RangeError");
    assert_throws(
        &mut engine,
        "BigInt.asIntN(9007199254740992, 0n)",
        "RangeError",
    );
    assert_throws(&mut engine, "BigInt.asIntN(0n, 0n)", "TypeError");
}

#[test]
fn as_uint_n() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "BigInt.asUintN(0, -2n)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt.asUintN(0, -1n)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt.asUintN(0, 0n)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt.asUintN(0, 1n)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt.asUintN(0, 2n)"), "0n");

    assert_eq!(forward(&mut engine, "BigInt.asUintN(1, -3n)"), "1n");
    assert_eq!(forward(&mut engine, "BigInt.asUintN(1, -2n)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt.asUintN(1, -1n)"), "1n");
    assert_eq!(forward(&mut engine, "BigInt.asUintN(1, 0n)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt.asUintN(1, 1n)"), "1n");
    assert_eq!(forward(&mut engine, "BigInt.asUintN(1, 2n)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt.asUintN(1, 3n)"), "1n");

    assert_eq!(
        forward(&mut engine, "BigInt.asUintN(1, -123456789012345678901n)"),
        "1n"
    );
    assert_eq!(
        forward(&mut engine, "BigInt.asUintN(1, -123456789012345678900n)"),
        "0n"
    );
    assert_eq!(
        forward(&mut engine, "BigInt.asUintN(1, 123456789012345678900n)"),
        "0n"
    );
    assert_eq!(
        forward(&mut engine, "BigInt.asUintN(1, 123456789012345678901n)"),
        "1n"
    );

    assert_eq!(
        forward(
            &mut engine,
            "BigInt.asUintN(200, 0xbffffffffffffffffffffffffffffffffffffffffffffffffffn)"
        ),
        "1606938044258990275541962092341162602522202993782792835301375n"
    );
    assert_eq!(
        forward(
            &mut engine,
            "BigInt.asUintN(201, 0xbffffffffffffffffffffffffffffffffffffffffffffffffffn)"
        ),
        "3213876088517980551083924184682325205044405987565585670602751n"
    );
}

#[test]
fn as_uint_n_errors() {
    let mut engine = Context::new();

    assert_throws(&mut engine, "BigInt.asUintN(-1, 0n)", "RangeError");
    assert_throws(&mut engine, "BigInt.asUintN(-2.5, 0n)", "RangeError");
    assert_throws(
        &mut engine,
        "BigInt.asUintN(9007199254740992, 0n)",
        "RangeError",
    );
    assert_throws(&mut engine, "BigInt.asUintN(0n, 0n)", "TypeError");
}

fn assert_throws(engine: &mut Context, src: &str, error_type: &str) {
    let result = forward(engine, src);
    assert!(result.contains(error_type));
}
