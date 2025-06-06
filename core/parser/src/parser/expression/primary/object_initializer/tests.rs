use crate::parser::tests::{check_invalid_script, check_script_parser};
use boa_ast::{
    declaration::{LexicalDeclaration, Variable},
    expression::{
        literal::{Literal, ObjectLiteral, ObjectMethodDefinition, PropertyDefinition},
        Identifier,
    },
    function::{FormalParameter, FormalParameterList, FormalParameterListFlags, FunctionBody},
    property::{MethodDefinitionKind, PropertyName},
    Declaration, Span,
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;
use indoc::indoc;

const PSEUDO_LINEAR_POS: boa_ast::LinearPosition = boa_ast::LinearPosition::new(0);

/// Checks object literal parsing.
#[test]
fn check_object_literal() {
    let interner = &mut Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::new(true, Span::new((2, 8), (2, 12))).into(),
        ),
        PropertyDefinition::Property(
            interner.get_or_intern_static("b", utf16!("b")).into(),
            Literal::new(false, Span::new((3, 8), (3, 13))).into(),
        ),
    ];

    check_script_parser(
        indoc! {"
            const x = {
                a: true,
                b: false,
            };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (4, 2))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Tests short function syntax.
#[test]
fn check_object_short_function() {
    let interner = &mut Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::new(true, Span::new((2, 8), (2, 12))).into(),
        ),
        PropertyDefinition::MethodDefinition(ObjectMethodDefinition::new(
            interner.get_or_intern_static("b", utf16!("b")).into(),
            FormalParameterList::default(),
            FunctionBody::default(),
            MethodDefinitionKind::Ordinary,
            PSEUDO_LINEAR_POS,
        )),
    ];

    check_script_parser(
        indoc! {"
            const x = {
                a: true,
                b() {},
            };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (4, 2))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Testing short function syntax with arguments.
#[test]
fn check_object_short_function_arguments() {
    let interner = &mut Interner::default();

    let parameters = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(
            interner.get_or_intern_static("test", utf16!("test")).into(),
            None,
        ),
        false,
    ));

    assert_eq!(parameters.flags(), FormalParameterListFlags::default());
    assert_eq!(parameters.length(), 1);

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::new(true, Span::new((2, 8), (2, 12))).into(),
        ),
        PropertyDefinition::MethodDefinition(ObjectMethodDefinition::new(
            interner.get_or_intern_static("b", utf16!("b")).into(),
            parameters,
            FunctionBody::default(),
            MethodDefinitionKind::Ordinary,
            PSEUDO_LINEAR_POS,
        )),
    ];

    check_script_parser(
        indoc! {"
            const x = {
                a: true,
                b(test) {}
            };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (4, 2))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_object_getter() {
    let interner = &mut Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::new(true, Span::new((2, 8), (2, 12))).into(),
        ),
        PropertyDefinition::MethodDefinition(ObjectMethodDefinition::new(
            interner.get_or_intern_static("b", utf16!("b")).into(),
            FormalParameterList::default(),
            FunctionBody::default(),
            MethodDefinitionKind::Get,
            PSEUDO_LINEAR_POS,
        )),
    ];

    check_script_parser(
        indoc! {"
            const x = {
                a: true,
                get b() {}
            };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (4, 2))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_object_setter() {
    let interner = &mut Interner::default();

    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(
            interner.get_or_intern_static("test", utf16!("test")).into(),
            None,
        ),
        false,
    ));

    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 1);

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::new(true, Span::new((2, 8), (2, 12))).into(),
        ),
        PropertyDefinition::MethodDefinition(ObjectMethodDefinition::new(
            interner.get_or_intern_static("b", utf16!("b")).into(),
            params,
            FunctionBody::default(),
            MethodDefinitionKind::Set,
            PSEUDO_LINEAR_POS,
        )),
    ];

    check_script_parser(
        indoc! {"
            const x = {
                a: true,
                set b(test) {}
            };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (4, 2))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_object_short_function_get() {
    let interner = &mut Interner::default();

    let object_properties = vec![PropertyDefinition::MethodDefinition(
        ObjectMethodDefinition::new(
            Sym::GET.into(),
            FormalParameterList::default(),
            FunctionBody::default(),
            MethodDefinitionKind::Ordinary,
            PSEUDO_LINEAR_POS,
        ),
    )];

    check_script_parser(
        indoc! {"
            const x = {
                get() {}
            };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (3, 2))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_object_short_function_set() {
    let interner = &mut Interner::default();

    let object_properties = vec![PropertyDefinition::MethodDefinition(
        ObjectMethodDefinition::new(
            Sym::SET.into(),
            FormalParameterList::default(),
            FunctionBody::default(),
            MethodDefinitionKind::Ordinary,
            PSEUDO_LINEAR_POS,
        ),
    )];

    check_script_parser(
        indoc! {"
            const x = {
                set() {}
            };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (3, 2))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_object_shorthand_property_names() {
    let interner = &mut Interner::default();

    let object_properties = vec![PropertyDefinition::IdentifierReference(
        interner.get_or_intern_static("a", utf16!("a")).into(),
    )];

    check_script_parser(
        indoc! {"
            const a = true;
            const x = { a };
        "},
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::new(true, Span::new((1, 11), (1, 15))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("x", utf16!("x")).into(),
                    Some(ObjectLiteral::new(object_properties, Span::new((2, 11), (2, 16))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn check_object_shorthand_multiple_properties() {
    let interner = &mut Interner::default();

    let object_properties = vec![
        PropertyDefinition::IdentifierReference(
            interner.get_or_intern_static("a", utf16!("a")).into(),
        ),
        PropertyDefinition::IdentifierReference(
            interner.get_or_intern_static("b", utf16!("b")).into(),
        ),
    ];

    check_script_parser(
        indoc! {"
            const a = true;
            const b = false;
            const x = { a, b, };
        "},
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::new(true, Span::new((1, 11), (1, 15))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("b", utf16!("b")).into(),
                    Some(Literal::new(false, Span::new((2, 11), (2, 16))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("x", utf16!("x")).into(),
                    Some(ObjectLiteral::new(object_properties, Span::new((3, 11), (3, 20))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn check_object_spread() {
    let interner = &mut Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::new(1, Span::new((1, 16), (1, 17))).into(),
        ),
        PropertyDefinition::SpreadObject(
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        ),
    ];

    check_script_parser(
        "const x = { a: 1, ...b };",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (1, 25))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_async_method() {
    let interner = &mut Interner::default();

    let object_properties = vec![PropertyDefinition::MethodDefinition(
        ObjectMethodDefinition::new(
            PropertyName::Literal(interner.get_or_intern_static("dive", utf16!("dive"))),
            FormalParameterList::default(),
            FunctionBody::default(),
            MethodDefinitionKind::Async,
            PSEUDO_LINEAR_POS,
        ),
    )];

    check_script_parser(
        indoc! {"
            const x = {
                async dive() {}
            };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (3, 2))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_async_generator_method() {
    let interner = &mut Interner::default();

    let object_properties = vec![PropertyDefinition::MethodDefinition(
        ObjectMethodDefinition::new(
            PropertyName::Literal(interner.get_or_intern_static("vroom", utf16!("vroom"))),
            FormalParameterList::default(),
            FunctionBody::default(),
            MethodDefinitionKind::AsyncGenerator,
            PSEUDO_LINEAR_POS,
        ),
    )];

    check_script_parser(
        indoc! {"
            const x = {
                async* vroom() {}
            };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (3, 2))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_async_method_lineterminator() {
    check_invalid_script(
        "const x = {
            async
            dive(){}
        };
        ",
    );
}

#[test]
fn check_async_gen_method_lineterminator() {
    check_invalid_script(
        "const x = {
            async
            * vroom() {}
        };
        ",
    );
}

#[test]
fn check_async_ordinary_method() {
    let interner = &mut Interner::default();

    let object_properties = vec![PropertyDefinition::MethodDefinition(
        ObjectMethodDefinition::new(
            PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
            FormalParameterList::default(),
            FunctionBody::default(),
            MethodDefinitionKind::Ordinary,
            PSEUDO_LINEAR_POS,
        ),
    )];

    check_script_parser(
        indoc! {r#"
            const x = {
                async() {}
            };
        "#},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (3, 2))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_async_property() {
    let interner = &mut Interner::default();

    let object_properties = vec![PropertyDefinition::Property(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        Literal::new(true, Span::new((2, 12), (2, 16))).into(),
    )];

    check_script_parser(
        indoc! {"
            const x = {
                async: true
            };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::new(object_properties, Span::new((1, 11), (3, 2))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}
