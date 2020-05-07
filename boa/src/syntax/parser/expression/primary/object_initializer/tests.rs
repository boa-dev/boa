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
        vec![Node::const_decl(vec![(
            String::from("x"),
            Node::object(object_properties),
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
            Node::function_expr::<_, String, _, _>(None, Vec::new(), Node::statement_list(vec![])),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b() {},
        };
        ",
        vec![Node::const_decl(vec![(
            String::from("x"),
            Node::object(object_properties),
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
            Node::FunctionExpr(
                None,
                Box::new([FormalParameter::new("test", None, false)]),
                Box::new(Node::StatementList(Box::new([]))),
            ),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b(test) {}
         };
        ",
        vec![Node::const_decl(vec![(
            String::from("x"),
            Node::object(object_properties),
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
            Node::FunctionExpr(
                None,
                Box::new([]),
                Box::new(Node::statement_list(Vec::new())),
            ),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            get b() {}
        };
        ",
        vec![Node::const_decl(vec![(
            String::from("x"),
            Node::object(object_properties),
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
            Node::function_expr::<_, String, _, _>(
                None,
                vec![FormalParameter::new("test", None, false)],
                Node::statement_list(Vec::new()),
            ),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            set b(test) {}
        };
        ",
        vec![Node::const_decl(vec![(
            String::from("x"),
            Node::object(object_properties),
        )])],
    );
}
