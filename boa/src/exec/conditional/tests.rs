use crate::exec;

#[test]
fn if_true() {
    let scenario = r#"
        var a = 0;
        if (true) {
            a = 100;
        } else {
            a = 50;
        }
        a;
    "#;

    assert_eq!(&exec(scenario), "100");
}

#[test]
fn if_false() {
    let scenario = r#"
        var a = 0;
        if (false) {
            a = 100;
        } else {
            a = 50;
        }
        a;
    "#;

    assert_eq!(&exec(scenario), "50");
}

#[test]
fn if_truthy() {
    let scenario = r#"
        var a = 0;
        if ("this value doesn't matter") {
            a = 100;
        } else {
            a = 50;
        }
        a;
    "#;

    assert_eq!(&exec(scenario), "100");
}

#[test]
fn if_falsy() {
    let scenario = r#"
        var a = 0;
        if (null) {
            a = 100;
        } else {
            a = 50;
        }
        a;
    "#;

    assert_eq!(&exec(scenario), "50");
}

#[test]
fn if_throws() {
    let scenario = r#"
        var a = 0;
        if ((function() { throw new Error("Oh no!") })()) {
            a = 100;
        } else {
            a = 50;
        }
        a;
    "#;

    assert_eq!(&exec(scenario), r#"Error: "Error": "Oh no!""#);
}

#[test]
fn conditional_op_true() {
    let scenario = r#"true ? 100 : 50"#;

    assert_eq!(&exec(scenario), "100");
}

#[test]
fn conditional_op_false() {
    let scenario = r#"false ? 100 : 50"#;

    assert_eq!(&exec(scenario), "50");
}

#[test]
fn conditional_op_truthy() {
    let scenario = r#""this text should be irrelevant" ? 100 : 50"#;

    assert_eq!(&exec(scenario), "100");
}

#[test]
fn conditional_op_falsy() {
    let scenario = r#"null ? 100 : 50"#;

    assert_eq!(&exec(scenario), "50");
}

#[test]
fn conditional_op_throws() {
    let scenario = r#"(function() { throw new Error("Oh no!") })() ? 100 : 50"#;

    assert_eq!(&exec(scenario), r#"Error: "Error": "Oh no!""#);
}
