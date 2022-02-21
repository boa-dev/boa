use boa::Context;

#[test]
#[should_panic]
fn invalid_break_target() {
    let source = r#"
while (false) {
  break nonexistent;
}
"#;

    let mut context = Context::default();
    assert!(context.eval(source).is_ok());
}