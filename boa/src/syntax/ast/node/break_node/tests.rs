use crate::{
    exec::{Executable, InterpreterState},
    syntax::ast::node::Break,
    Context,
};

#[test]
fn check_post_state() {
    let mut engine = Context::new();

    let brk: Break = Break::new("label");

    brk.run(&mut engine).unwrap();

    assert_eq!(
        engine.executor().get_current_state(),
        &InterpreterState::Break(Some("label".into()))
    );
}
