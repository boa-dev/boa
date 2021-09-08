#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        call_1(1, 2, 3);
        call_2("argument here");
        call_3();
        "#,
    );
}
