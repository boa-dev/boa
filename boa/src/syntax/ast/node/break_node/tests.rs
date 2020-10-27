use crate::{
    exec::{Executable, InterpreterState},
    syntax::ast::node::Break,
    Context,
};

#[test]
fn check_post_state() {
    let mut context = Context::new();

    let brk: Break = Break::new("label");

    brk.run(&mut context).unwrap();

    assert_eq!(
        context.executor().get_current_state(),
        &InterpreterState::Break(Some("label".into()))
    );
}
