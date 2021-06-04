use crate::exec;

#[test]
fn typeof_string() {
    let typeof_object = r#"
        const a = "hello";
        typeof a;
    "#;
    assert_eq!(&exec(typeof_object), "\"string\"");
}

#[test]
fn typeof_number() {
    let typeof_number = r#"
        let a = 1234;
        typeof a;
    "#;
    assert_eq!(&exec(typeof_number), "\"number\"");
}

#[test]
fn basic_op() {
    let basic_op = r#"
        const a = 1;
        const b = 2;
        a + b
    "#;
    assert_eq!(&exec(basic_op), "3");
}
