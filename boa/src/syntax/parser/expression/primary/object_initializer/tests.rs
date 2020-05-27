use crate::syntax::{
    ast::{
        node::{
            ConstDecl, ConstDeclList, FormalParameter, FunctionExpr, MethodDefinitionKind, Object,
            PropertyDefinition,
        },
        Const,
    },
    parser::tests::check_parser,
};

/// Checks object literal parsing.
#[test]
fn check_object_literal() {
    let object_properties = vec![
        PropertyDefinition::property("a", Const::from(true)),
        PropertyDefinition::property("b", Const::from(false)),
    ];

    check_parser(
        "const x = {
            a: true,
            b: false,
        };
        ",
        vec![
            ConstDeclList::from(vec![ConstDecl::new("x", Object::from(object_properties))]).into(),
        ],
    );
}

/// Tests short function syntax.
#[test]
fn check_object_short_function() {
    let object_properties = vec![
        PropertyDefinition::property("a", Const::from(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Ordinary,
            "b",
            FunctionExpr::new(None, vec![], vec![]),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b() {},
        };
        ",
        vec![
            ConstDeclList::from(vec![ConstDecl::new("x", Object::from(object_properties))]).into(),
        ],
    );
}

/// Testing short function syntax with arguments.
#[test]
fn check_object_short_function_arguments() {
    let object_properties = vec![
        PropertyDefinition::property("a", Const::from(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Ordinary,
            "b",
            FunctionExpr::new(
                None,
                vec![FormalParameter::new("test", None, false)],
                vec![],
            ),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b(test) {}
         };
        ",
        vec![
            ConstDeclList::from(vec![ConstDecl::new("x", Object::from(object_properties))]).into(),
        ],
    );
}

#[test]
fn check_object_getter() {
    let object_properties = vec![
        PropertyDefinition::property("a", Const::from(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Get,
            "b",
            FunctionExpr::new(None, vec![], vec![]),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            get b() {}
        };
        ",
        vec![
            ConstDeclList::from(vec![ConstDecl::new("x", Object::from(object_properties))]).into(),
        ],
    );
}

#[test]
fn check_object_setter() {
    let object_properties = vec![
        PropertyDefinition::property("a", Const::from(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Set,
            "b",
            FunctionExpr::new(
                None,
                vec![FormalParameter::new("test", None, false)],
                vec![],
            ),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            set b(test) {}
        };
        ",
        vec![
            ConstDeclList::from(vec![ConstDecl::new("x", Object::from(object_properties))]).into(),
        ],
    );
}
