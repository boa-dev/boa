use crate::exec;

#[test]
fn vm_typeof_string() {
    let typeof_object = r#"
        const a = "hello";
        typeof a;
    "#;
    assert_eq!(&exec(typeof_object), "\"string\"");
}

#[test]
fn vm_typeof_number() {
    let typeof_number = r#"
        let a = 1234;
        typeof a;
    "#;
    assert_eq!(&exec(typeof_number), "\"number\"");
}
