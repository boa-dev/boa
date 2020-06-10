use crate::{builtins::Value, exec, exec::Interpreter, forward, realm::Realm};

#[test]
fn property_accessor_member_expression_dot_notation_on_string_literal() {
    let scenario = r#"
        NaN;
        "#;

    assert_eq!(&exec(scenario), "NaN");
}
