use super::{Interpreter, InterpreterState};
use crate::{exec::Executable, syntax::ast::node::Break, Realm};

#[test]
fn check_post_state() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let brk: Break = Break::new("label");

    brk.run(&mut engine).unwrap();

    assert_eq!(
        engine.get_current_state(),
        &InterpreterState::Break(Some("label".to_string()))
    );
}
