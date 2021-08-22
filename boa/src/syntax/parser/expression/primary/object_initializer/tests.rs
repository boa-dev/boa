use crate::syntax::{
    ast::{
        node::{
            Declaration, DeclarationList, FormalParameter, FunctionExpr, MethodDefinitionKind,
            Object, PropertyDefinition,
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
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "x",
                Some(Object::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
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
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "x",
                Some(Object::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
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
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "x",
                Some(Object::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
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
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "x",
                Some(Object::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
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
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "x",
                Some(Object::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
    );
}

#[test]
fn check_object_short_function_get() {
    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinitionKind::Ordinary,
        "get",
        FunctionExpr::new(None, vec![], vec![]),
    )];

    check_parser(
        "const x = {
            get() {}
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "x",
                Some(Object::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
    );
}

#[test]
fn check_object_short_function_set() {
    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinitionKind::Ordinary,
        "set",
        FunctionExpr::new(None, vec![], vec![]),
    )];

    check_parser(
        "const x = {
            set() {}
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "x",
                Some(Object::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
    );
}
