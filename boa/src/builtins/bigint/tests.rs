use crate::{forward, forward_val, Interpreter, Realm};

#[test]
fn equality() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(forward(&mut engine, "BigInt(0.0)"), "0n");
    assert_eq!(forward(&mut engine, "BigInt(1.0)"), "1n");
    assert_eq!(forward(&mut engine, "BigInt(10000.0)"), "10000n");
}

#[test]
fn bigint_function_conversion_from_rational_with_fractional_part() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let scenario = r#"
        var x = false;
        try {
            BigInt(0.1);
        } catch (e) {
            x = true;
        }
    "#;
    forward_val(&mut engine, scenario).unwrap();
    assert_eq!(forward(&mut engine, "x"), "true");
}

#[test]
fn bigint_function_conversion_from_null() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let scenario = r#"
        var x = false;
        try {
            BigInt(null);
        } catch (e) {
            x = true;
        }
    "#;
    forward_val(&mut engine, scenario).unwrap();
    assert_eq!(forward(&mut engine, "x"), "true");
}

#[test]
fn bigint_function_conversion_from_undefined() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let scenario = r#"
        var x = false;
        try {
            BigInt(undefined);
        } catch (e) {
            x = true;
        }
    "#;
    forward_val(&mut engine, scenario).unwrap();
    assert_eq!(forward(&mut engine, "x"), "true");
}

#[test]
fn bigint_function_conversion_from_string() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(forward(&mut engine, "10000n + 1000n"), "11000n");
}

#[test]
fn sub() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(forward(&mut engine, "10000n - 1000n"), "9000n");
}

#[test]
fn mul() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(
        forward(&mut engine, "123456789n * 102030n"),
        "12596296181670n"
    );
}

#[test]
fn div() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(forward(&mut engine, "15000n / 50n"), "300n");
}

#[test]
fn div_with_truncation() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(forward(&mut engine, "15001n / 50n"), "300n");
}

#[test]
fn r#mod() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(forward(&mut engine, "15007n % 10n"), "7n");
}

#[test]
fn pow() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(
        forward(&mut engine, "100n ** 10n"),
        "100000000000000000000n"
    );
}

#[test]
fn to_string() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(forward(&mut engine, "1000n.toString()"), "1000");
    assert_eq!(forward(&mut engine, "1000n.toString(2)"), "1111101000");
    assert_eq!(forward(&mut engine, "255n.toString(16)"), "ff");
    assert_eq!(forward(&mut engine, "1000n.toString(36)"), "rs");
}
