use crate::{check_output, forward, forward_val, Context, TestAction};

#[test]
fn call_symbol_and_check_return_type() {
    let mut context = Context::default();
    let init = r#"
        var sym = Symbol();
        "#;
    eprintln!("{}", forward(&mut context, init));
    let sym = forward_val(&mut context, "sym").unwrap();
    assert!(sym.is_symbol());
}

#[test]
fn print_symbol_expect_description() {
    let mut context = Context::default();
    let init = r#"
        var sym = Symbol("Hello");
        "#;
    eprintln!("{}", forward(&mut context, init));
    let sym = forward_val(&mut context, "sym.toString()").unwrap();
    assert_eq!(sym.display().to_string(), "\"Symbol(Hello)\"");
}

#[test]
fn symbol_access() {
    let init = r#"
        var x = {};
        var sym1 = Symbol("Hello");
        var sym2 = Symbol("Hello");
        x[sym1] = 10;
        x[sym2] = 20;
        "#;
    check_output(&[
        TestAction::Execute(init),
        TestAction::TestEq("x[sym1]", "10"),
        TestAction::TestEq("x[sym2]", "20"),
        TestAction::TestEq("x['Symbol(Hello)']", "undefined"),
    ]);
}
