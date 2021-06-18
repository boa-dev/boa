#[test]
fn fmt() {
    // TODO: `let a = await fn()` is invalid syntax as of writing. It should be tested here once implemented.
    super::super::test_formatting(
        r#"
        await function_call();
        "#,
    );
}
