use crate::{forward, Context};

#[test]
fn equality() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "0n == 0n"), "true");
    assert_eq!(forward(&mut context, "1n == 0n"), "false");
    assert_eq!(
        forward(
            &mut context,
            "1000000000000000000000000000000000n == 1000000000000000000000000000000000n"
        ),
        "true"
    );

    assert_eq!(forward(&mut context, "0n == ''"), "true");
    assert_eq!(forward(&mut context, "100n == '100'"), "true");
    assert_eq!(forward(&mut context, "100n == '100.5'"), "false");
    assert_eq!(
        forward(&mut context, "10000000000000000n == '10000000000000000'"),
        "true"
    );

    assert_eq!(forward(&mut context, "'' == 0n"), "true");
    assert_eq!(forward(&mut context, "'100' == 100n"), "true");
    assert_eq!(forward(&mut context, "'100.5' == 100n"), "false");
    assert_eq!(
        forward(&mut context, "'10000000000000000' == 10000000000000000n"),
        "true"
    );

    assert_eq!(forward(&mut context, "0n == 0"), "true");
    assert_eq!(forward(&mut context, "0n == 0.0"), "true");
    assert_eq!(forward(&mut context, "100n == 100"), "true");
    assert_eq!(forward(&mut context, "100n == 100.0"), "true");
    assert_eq!(forward(&mut context, "100n == '100.5'"), "false");
    assert_eq!(forward(&mut context, "100n == '1005'"), "false");
    assert_eq!(
        forward(&mut context, "10000000000000000n == 10000000000000000"),
        "true"
    );

    assert_eq!(forward(&mut context, "0 == 0n"), "true");
    assert_eq!(forward(&mut context, "0.0 == 0n"), "true");
    assert_eq!(forward(&mut context, "100 == 100n"), "true");
    assert_eq!(forward(&mut context, "100.0 == 100n"), "true");
    assert_eq!(forward(&mut context, "100.5 == 100n"), "false");
    assert_eq!(forward(&mut context, "1005 == 100n"), "false");
    assert_eq!(
        forward(&mut context, "10000000000000000 == 10000000000000000n"),
        "true"
    );
}

#[test]
fn bigint_function_conversion_from_integer() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "BigInt(1000)"), "1000n");
    assert_eq!(
        forward(&mut context, "BigInt(20000000000000000)"),
        "20000000000000000n"
    );
    assert_eq!(
        forward(&mut context, "BigInt(1000000000000000000000000000000000)"),
        "999999999999999945575230987042816n"
    );
}

#[test]
fn bigint_function_conversion_from_rational() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "BigInt(0.0)"), "0n");
    assert_eq!(forward(&mut context, "BigInt(1.0)"), "1n");
    assert_eq!(forward(&mut context, "BigInt(10000.0)"), "10000n");
}

#[test]
fn bigint_function_conversion_from_rational_with_fractional_part() {
    let mut context = Context::default();

    let scenario = r#"
        try {
            BigInt(0.1);
        } catch (e) {
            e.toString();
        }
    "#;
    assert_eq!(
        forward(&mut context, scenario),
        "\"RangeError: Cannot convert 0.1 to BigInt\""
    );
}

#[test]
fn bigint_function_conversion_from_null() {
    let mut context = Context::default();

    let scenario = r#"
        try {
            BigInt(null);
        } catch (e) {
            e.toString();
        }
    "#;
    assert_eq!(
        forward(&mut context, scenario),
        "\"TypeError: cannot convert null to a BigInt\""
    );
}

#[test]
fn bigint_function_conversion_from_undefined() {
    let mut context = Context::default();

    let scenario = r#"
        try {
            BigInt(undefined);
        } catch (e) {
            e.toString();
        }
    "#;
    assert_eq!(
        forward(&mut context, scenario),
        "\"TypeError: cannot convert undefined to a BigInt\""
    );
}

#[test]
fn bigint_function_conversion_from_string() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "BigInt('')"), "0n");
    assert_eq!(forward(&mut context, "BigInt('   ')"), "0n");
    assert_eq!(
        forward(&mut context, "BigInt('200000000000000000')"),
        "200000000000000000n"
    );
    assert_eq!(
        forward(&mut context, "BigInt('1000000000000000000000000000000000')"),
        "1000000000000000000000000000000000n"
    );
    assert_eq!(forward(&mut context, "BigInt('0b1111')"), "15n");
    assert_eq!(forward(&mut context, "BigInt('0o70')"), "56n");
    assert_eq!(forward(&mut context, "BigInt('0xFF')"), "255n");
}

#[test]
fn add() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "10000n + 1000n"), "11000n");
}

#[test]
fn sub() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "10000n - 1000n"), "9000n");
}

#[test]
fn mul() {
    let mut context = Context::default();

    assert_eq!(
        forward(&mut context, "123456789n * 102030n"),
        "12596296181670n"
    );
}

#[test]
fn div() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "15000n / 50n"), "300n");
}

#[test]
fn div_with_truncation() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "15001n / 50n"), "300n");
}

#[test]
fn r#mod() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "15007n % 10n"), "7n");
}

#[test]
fn pow() {
    let mut context = Context::default();

    assert_eq!(
        forward(&mut context, "100n ** 10n"),
        "100000000000000000000n"
    );
}

#[test]
fn pow_negative_exponent() {
    let mut context = Context::default();

    assert_throws(&mut context, "10n ** (-10n)", "RangeError");
}

#[test]
fn shl() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "8n << 2n"), "32n");
}

#[test]
fn shl_out_of_range() {
    let mut context = Context::default();

    assert_throws(&mut context, "1000n << 1000000000000000n", "RangeError");
}

#[test]
fn shr() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "8n >> 2n"), "2n");
}

#[test]
fn shr_out_of_range() {
    let mut context = Context::default();

    assert_throws(&mut context, "1000n >> 1000000000000000n", "RangeError");
}

#[test]
fn to_string() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "1000n.toString()"), "\"1000\"");
    assert_eq!(forward(&mut context, "1000n.toString(2)"), "\"1111101000\"");
    assert_eq!(forward(&mut context, "255n.toString(16)"), "\"ff\"");
    assert_eq!(forward(&mut context, "1000n.toString(36)"), "\"rs\"");
}

#[test]
fn to_string_invalid_radix() {
    let mut context = Context::default();

    assert_throws(&mut context, "10n.toString(null)", "RangeError");
    assert_throws(&mut context, "10n.toString(-1)", "RangeError");
    assert_throws(&mut context, "10n.toString(37)", "RangeError");
}

#[test]
fn as_int_n() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "BigInt.asIntN(0, 1n)"), "0n");
    assert_eq!(forward(&mut context, "BigInt.asIntN(1, 1n)"), "-1n");
    assert_eq!(forward(&mut context, "BigInt.asIntN(3, 10n)"), "2n");
    assert_eq!(forward(&mut context, "BigInt.asIntN({}, 1n)"), "0n");
    assert_eq!(forward(&mut context, "BigInt.asIntN(2, 0n)"), "0n");
    assert_eq!(forward(&mut context, "BigInt.asIntN(2, -0n)"), "0n");

    assert_eq!(
        forward(&mut context, "BigInt.asIntN(2, -123456789012345678901n)"),
        "-1n"
    );
    assert_eq!(
        forward(&mut context, "BigInt.asIntN(2, -123456789012345678900n)"),
        "0n"
    );

    assert_eq!(
        forward(&mut context, "BigInt.asIntN(2, 123456789012345678900n)"),
        "0n"
    );
    assert_eq!(
        forward(&mut context, "BigInt.asIntN(2, 123456789012345678901n)"),
        "1n"
    );

    assert_eq!(
        forward(
            &mut context,
            "BigInt.asIntN(200, 0xcffffffffffffffffffffffffffffffffffffffffffffffffffn)"
        ),
        "-1n"
    );
    assert_eq!(
        forward(
            &mut context,
            "BigInt.asIntN(201, 0xcffffffffffffffffffffffffffffffffffffffffffffffffffn)"
        ),
        "1606938044258990275541962092341162602522202993782792835301375n"
    );

    assert_eq!(
        forward(
            &mut context,
            "BigInt.asIntN(200, 0xc89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n)"
        ),
        "-741470203160010616172516490008037905920749803227695190508460n"
    );
    assert_eq!(
        forward(
            &mut context,
            "BigInt.asIntN(201, 0xc89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n)"
        ),
        "865467841098979659369445602333124696601453190555097644792916n"
    );
}

#[test]
fn as_int_n_errors() {
    let mut context = Context::default();

    assert_throws(&mut context, "BigInt.asIntN(-1, 0n)", "RangeError");
    assert_throws(&mut context, "BigInt.asIntN(-2.5, 0n)", "RangeError");
    assert_throws(
        &mut context,
        "BigInt.asIntN(9007199254740992, 0n)",
        "RangeError",
    );
    assert_throws(&mut context, "BigInt.asIntN(0n, 0n)", "TypeError");
}

#[test]
fn as_uint_n() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "BigInt.asUintN(0, -2n)"), "0n");
    assert_eq!(forward(&mut context, "BigInt.asUintN(0, -1n)"), "0n");
    assert_eq!(forward(&mut context, "BigInt.asUintN(0, 0n)"), "0n");
    assert_eq!(forward(&mut context, "BigInt.asUintN(0, 1n)"), "0n");
    assert_eq!(forward(&mut context, "BigInt.asUintN(0, 2n)"), "0n");

    assert_eq!(forward(&mut context, "BigInt.asUintN(1, -3n)"), "1n");
    assert_eq!(forward(&mut context, "BigInt.asUintN(1, -2n)"), "0n");
    assert_eq!(forward(&mut context, "BigInt.asUintN(1, -1n)"), "1n");
    assert_eq!(forward(&mut context, "BigInt.asUintN(1, 0n)"), "0n");
    assert_eq!(forward(&mut context, "BigInt.asUintN(1, 1n)"), "1n");
    assert_eq!(forward(&mut context, "BigInt.asUintN(1, 2n)"), "0n");
    assert_eq!(forward(&mut context, "BigInt.asUintN(1, 3n)"), "1n");

    assert_eq!(
        forward(&mut context, "BigInt.asUintN(1, -123456789012345678901n)"),
        "1n"
    );
    assert_eq!(
        forward(&mut context, "BigInt.asUintN(1, -123456789012345678900n)"),
        "0n"
    );
    assert_eq!(
        forward(&mut context, "BigInt.asUintN(1, 123456789012345678900n)"),
        "0n"
    );
    assert_eq!(
        forward(&mut context, "BigInt.asUintN(1, 123456789012345678901n)"),
        "1n"
    );

    assert_eq!(
        forward(
            &mut context,
            "BigInt.asUintN(200, 0xbffffffffffffffffffffffffffffffffffffffffffffffffffn)"
        ),
        "1606938044258990275541962092341162602522202993782792835301375n"
    );
    assert_eq!(
        forward(
            &mut context,
            "BigInt.asUintN(201, 0xbffffffffffffffffffffffffffffffffffffffffffffffffffn)"
        ),
        "3213876088517980551083924184682325205044405987565585670602751n"
    );
}

#[test]
fn as_uint_n_errors() {
    let mut context = Context::default();

    assert_throws(&mut context, "BigInt.asUintN(-1, 0n)", "RangeError");
    assert_throws(&mut context, "BigInt.asUintN(-2.5, 0n)", "RangeError");
    assert_throws(
        &mut context,
        "BigInt.asUintN(9007199254740992, 0n)",
        "RangeError",
    );
    assert_throws(&mut context, "BigInt.asUintN(0n, 0n)", "TypeError");
}

fn assert_throws(context: &mut Context, src: &str, error_type: &str) {
    let result = forward(context, src);
    assert!(result.contains(error_type));
}

#[test]
fn division_by_zero() {
    let mut context = Context::default();
    assert_throws(&mut context, "1n/0n", "RangeError");
}

#[test]
fn remainder_by_zero() {
    let mut context = Context::default();
    assert_throws(&mut context, "1n % 0n", "RangeError");
}
