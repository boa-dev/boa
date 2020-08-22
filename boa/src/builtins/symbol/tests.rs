use crate::{forward, forward_val, Context};

#[test]
fn call_symbol_and_check_return_type() {
    let mut engine = Context::new();
    let init = r#"
        var sym = Symbol();
        "#;
    eprintln!("{}", forward(&mut engine, init));
    let sym = forward_val(&mut engine, "sym").unwrap();
    assert_eq!(sym.is_symbol(), true);
}

#[test]
fn print_symbol_expect_description() {
    let mut engine = Context::new();
    let init = r#"
        var sym = Symbol("Hello");
        "#;
    eprintln!("{}", forward(&mut engine, init));
    let sym = forward_val(&mut engine, "sym.toString()").unwrap();
    assert_eq!(sym.display().to_string(), "\"Symbol(Hello)\"");
}

#[test]
fn symbol_access() {
    let mut engine = Context::new();
    let init = r#"
        var x = {};
        var sym1 = Symbol("Hello");
        var sym2 = Symbol("Hello");
        x[sym1] = 10;
        x[sym2] = 20;
        "#;
    forward_val(&mut engine, init).unwrap();
    assert_eq!(forward(&mut engine, "x[sym1]"), "10");
    assert_eq!(forward(&mut engine, "x[sym2]"), "20");
    assert_eq!(forward(&mut engine, "x['Symbol(Hello)']"), "undefined");
}
