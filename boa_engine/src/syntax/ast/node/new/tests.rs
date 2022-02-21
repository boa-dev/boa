#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        function MyClass() {};
        let inst = new MyClass();
        "#,
    );
}
