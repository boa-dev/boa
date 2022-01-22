use crate::{
    syntax::{
        ast::{
            node::{
                Declaration, DeclarationList, FormalParameter, FunctionExpr, Identifier,
                MethodDefinitionKind, Object, PropertyDefinition,
            },
            Const,
        },
        parser::tests::{check_invalid, check_parser},
    },
    Interner,
};

/// Checks object literal parsing.
#[test]
fn check_object_literal() {
    let object_properties = vec![
        PropertyDefinition::property("a", Const::from(true)),
        PropertyDefinition::property("b", Const::from(false)),
    ];

    let mut interner = Interner::new();
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
        &mut interner,
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

    let mut interner = Interner::new();
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
        &mut interner,
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
                vec![FormalParameter::new(
                    Declaration::new_with_identifier("test", None),
                    false,
                )],
                vec![],
            ),
        ),
    ];

    let mut interner = Interner::new();
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
        &mut interner,
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

    let mut interner = Interner::new();
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
        &mut interner,
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
                vec![FormalParameter::new(
                    Declaration::new_with_identifier("test", None),
                    false,
                )],
                vec![],
            ),
        ),
    ];

    let mut interner = Interner::new();
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
        &mut interner,
    );
}

#[test]
fn check_object_short_function_get() {
    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinitionKind::Ordinary,
        "get",
        FunctionExpr::new(None, vec![], vec![]),
    )];

    let mut interner = Interner::new();
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
        &mut interner,
    );
}

#[test]
fn check_object_short_function_set() {
    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinitionKind::Ordinary,
        "set",
        FunctionExpr::new(None, vec![], vec![]),
    )];

    let mut interner = Interner::new();
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
        &mut interner,
    );
}

#[test]
fn check_object_shorthand_property_names() {
    let object_properties = vec![PropertyDefinition::property("a", Identifier::from("a"))];

    let mut interner = Interner::new();
    check_parser(
        "const a = true;
            const x = { a };
        ",
        vec![
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    "a",
                    Some(Const::from(true).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    "x",
                    Some(Object::from(object_properties).into()),
                )]
                .into(),
            )
            .into(),
        ],
        &mut interner,
    );
}

#[test]
fn check_object_shorthand_multiple_properties() {
    let object_properties = vec![
        PropertyDefinition::property("a", Identifier::from("a")),
        PropertyDefinition::property("b", Identifier::from("b")),
    ];

    let mut interner = Interner::new();
    check_parser(
        "const a = true;
            const b = false;
            const x = { a, b, };
        ",
        vec![
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    "a",
                    Some(Const::from(true).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    "b",
                    Some(Const::from(false).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    "x",
                    Some(Object::from(object_properties).into()),
                )]
                .into(),
            )
            .into(),
        ],
        &mut interner,
    );
}

#[test]
fn check_object_spread() {
    let object_properties = vec![
        PropertyDefinition::property("a", Const::from(1)),
        PropertyDefinition::spread_object(Identifier::from("b")),
    ];

    let mut interner = Interner::new();
    check_parser(
        "const x = { a: 1, ...b };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "x",
                Some(Object::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn check_async_method() {
    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinitionKind::Async,
        "dive",
        FunctionExpr::new(None, vec![], vec![]),
    )];

    let mut interner = Interner::new();
    check_parser(
        "const x = {
            async dive() {}
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
        &mut interner,
    );
}

#[test]
fn check_async_generator_method() {
    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinitionKind::AsyncGenerator,
        "vroom",
        FunctionExpr::new(None, vec![], vec![]),
    )];

    let mut interner = Interner::new();
    check_parser(
        "const x = {
            async* vroom() {}
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
        &mut interner,
    );
}

#[test]
fn check_async_method_lineterminator() {
    check_invalid(
        "const x = {
            async
            dive(){}
        };
        ",
    )
}

#[test]
fn check_async_gen_method_lineterminator() {
    check_invalid(
        "const x = {
            async
            * vroom() {}
        };
        ",
    )
}
