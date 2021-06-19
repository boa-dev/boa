#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        let a = true ? 5 : 6;
        if (false) {
            a = 10;
        } else {
            a = 20;
        }
        "#,
    );
}
