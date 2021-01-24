use crate::{forward, Context};

#[test]
fn simple_func() {
    let mut context = Context::new();
    let js = r#"
        async function foo(a) { return a; }
    "#;

    eprintln!("{}", forward(&mut context, js));
    assert_eq!(forward(&mut context, "foo(1);"), "Promise"); // TODO define promise display.
}
