use crate::syntax::{
    ast::{
        node::{
            Declaration, DeclarationList, FormalParameter, FunctionExpr, Identifier,
            MethodDefinitionKind, Object, PropertyDefinition,
        },
        Const,
    },
    parser::tests::{check_invalid, check_parser},
};
use boa_interner::Interner;

/// Checks object literal parsing.
#[test]
fn check_object_literal() {
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::property(interner.get_or_intern_static("a"), Const::from(true)),
        PropertyDefinition::property(interner.get_or_intern_static("b"), Const::from(false)),
    ];

    check_parser(
        "const x = {
            a: true,
            b: false,
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x"),
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
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::property(interner.get_or_intern_static("a"), Const::from(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Ordinary,
            interner.get_or_intern_static("b"),
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
                interner.get_or_intern_static("x"),
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
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::property(interner.get_or_intern_static("a"), Const::from(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Ordinary,
            interner.get_or_intern_static("b"),
            FunctionExpr::new(
                None,
                vec![FormalParameter::new(
                    Declaration::new_with_identifier(interner.get_or_intern_static("test"), None),
                    false,
                )],
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
                interner.get_or_intern_static("x"),
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
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::property(interner.get_or_intern_static("a"), Const::from(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Get,
            interner.get_or_intern_static("b"),
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
                interner.get_or_intern_static("x"),
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
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::property(interner.get_or_intern_static("a"), Const::from(true)),
        PropertyDefinition::method_definition(
            MethodDefinitionKind::Set,
            interner.get_or_intern_static("b"),
            FunctionExpr::new(
                None,
                vec![FormalParameter::new(
                    Declaration::new_with_identifier(interner.get_or_intern_static("test"), None),
                    false,
                )],
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
                interner.get_or_intern_static("x"),
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
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinitionKind::Ordinary,
        interner.get_or_intern_static("get"),
        FunctionExpr::new(None, vec![], vec![]),
    )];

    check_parser(
        "const x = {
            get() {}
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x"),
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
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinitionKind::Ordinary,
        interner.get_or_intern_static("set"),
        FunctionExpr::new(None, vec![], vec![]),
    )];

    check_parser(
        "const x = {
            set() {}
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x"),
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
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::property(
        interner.get_or_intern_static("a"),
        Identifier::new(interner.get_or_intern_static("a")),
    )];

    check_parser(
        "const a = true;
            const x = { a };
        ",
        vec![
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("a"),
                    Some(Const::from(true).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("x"),
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
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::property(
            interner.get_or_intern_static("a"),
            Identifier::new(interner.get_or_intern_static("a")),
        ),
        PropertyDefinition::property(
            interner.get_or_intern_static("b"),
            Identifier::new(interner.get_or_intern_static("b")),
        ),
    ];

    check_parser(
        "const a = true;
            const b = false;
            const x = { a, b, };
        ",
        vec![
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("a"),
                    Some(Const::from(true).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("b"),
                    Some(Const::from(false).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("x"),
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
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::property(interner.get_or_intern_static("a"), Const::from(1)),
        PropertyDefinition::spread_object(Identifier::new(interner.get_or_intern_static("b"))),
    ];

    check_parser(
        "const x = { a: 1, ...b };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x"),
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
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinitionKind::Async,
        interner.get_or_intern_static("dive"),
        FunctionExpr::new(None, vec![], vec![]),
    )];

    check_parser(
        "const x = {
            async dive() {}
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x"),
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
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinitionKind::AsyncGenerator,
        interner.get_or_intern_static("vroom"),
        FunctionExpr::new(None, vec![], vec![]),
    )];

    check_parser(
        "const x = {
            async* vroom() {}
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x"),
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
    );
}

#[test]
fn check_async_gen_method_lineterminator() {
    check_invalid(
        "const x = {
            async
            * vroom() {}
        };
        ",
    );
}
