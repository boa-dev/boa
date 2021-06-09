#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        let a = [1, 2, 3, "words", "more words"];
        let b = [];
        "#,
    );
}
