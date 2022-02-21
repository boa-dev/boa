#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        try {
            throw "hello";
        } catch(e) {
            console.log(e);
        };
        "#,
    );
}
