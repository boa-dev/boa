use crate::{exec::Interpreter, forward, forward_val, realm::Realm};

#[test]
fn call_symbol_and_check_return_type() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var sym = Symbol();
        "#;
    eprintln!("{}", forward(&mut engine, init));
    let sym = forward_val(&mut engine, "sym").unwrap();
    assert_eq!(sym.is_symbol(), true);
}

#[test]
fn print_symbol_expect_description() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var sym = Symbol("Hello");
        "#;
    eprintln!("{}", forward(&mut engine, init));
    let sym = forward_val(&mut engine, "sym.toString()").unwrap();
    assert_eq!(sym.display().to_string(), "\"Symbol(Hello)\"");
}
