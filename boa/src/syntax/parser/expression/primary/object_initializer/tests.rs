use crate::syntax::{
    ast::node::{FormalParameter, MethodDefinitionKind, Node, PropertyDefinition},
    parser::tests::check_parser,
};

/// Checks object literal parsing.
#[test]
fn check_object_literal() {
    let object_properties = vec![
        PropertyDefinition::property("a", Node::const_node(true)),
        PropertyDefinition::property("b", Node::const_node(false)),
    ];

    check_parser(
        "const x = {
            a: true,
            b: false,
        };
        ",
        &[Node::const_decl(vec![(
            String::from("x"),
            Node::Object(object_properties),
        )])],
    );
}

/// Tests short function syntax.
#[test]
fn check_object_short_function() {
    let object_properties = vec![
        PropertyDefinition::property("a", Node::const_node(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Ordinary,
            "b",
            Node::function_decl::<_, String, _, _>(None, Vec::new(), Node::StatementList(vec![])),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b() {},
        };
        ",
        &[Node::ConstDecl(vec![(
            String::from("x"),
            Node::Object(object_properties),
        )])],
    );
}

/// Testing short function syntax with arguments.
#[test]
fn check_object_short_function_arguments() {
    let object_properties = vec![
        PropertyDefinition::property("a", Node::const_node(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Ordinary,
            "b",
            Node::function_decl::<_, String, _, _>(
                None,
                vec![FormalParameter::new("test", None, false)],
                Node::StatementList(Vec::new()),
            ),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b(test) {}
         };
        ",
        &[Node::ConstDecl(vec![(
            String::from("x"),
            Node::Object(object_properties),
        )])],
    );
}

#[test]
fn check_object_getter() {
    let object_properties = vec![
        PropertyDefinition::property("a", Node::const_node(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Get,
            "b",
            Node::FunctionDecl(None, Vec::new(), Box::new(Node::StatementList(Vec::new()))),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            get b() {}
        };
        ",
        &[Node::ConstDecl(vec![(
            String::from("x"),
            Node::Object(object_properties),
        )])],
    );
}

#[test]
fn check_object_setter() {
    let object_properties = vec![
        PropertyDefinition::property("a", Node::const_node(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Set,
            "b",
            Node::FunctionDecl(
                None,
                vec![FormalParameter::new("test", None, false)],
                Box::new(Node::StatementList(Vec::new())),
            ),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            set b(test) {}
        };
        ",
        &[Node::ConstDecl(vec![(
            String::from("x"),
            Node::Object(object_properties),
        )])],
    );
}
