use super::{Context, InterpreterState};
use crate::{exec::Executable, syntax::ast::node::Break};

#[test]
fn check_post_state() {
    let mut engine = Context::new();

    let brk: Break = Break::new("label");

    brk.run(&mut engine).unwrap();

    assert_eq!(
        engine.executor().get_current_state(),
        &InterpreterState::Break(Some("label".to_string()))
    );
}
