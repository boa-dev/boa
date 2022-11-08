use crate::parser::tests::{check_invalid, check_parser};
use boa_ast::{
    declaration::{LexicalDeclaration, Variable},
    expression::{
        literal::{Literal, ObjectLiteral},
        Identifier,
    },
    function::{
        AsyncFunction, AsyncGenerator, FormalParameter, FormalParameterList,
        FormalParameterListFlags, Function,
    },
    property::{MethodDefinition, PropertyDefinition, PropertyName},
    Declaration, StatementList,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Checks object literal parsing.
#[test]
fn check_object_literal() {
    let interner = &mut Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::from(true).into(),
        ),
        PropertyDefinition::Property(
            interner.get_or_intern_static("b", utf16!("b")).into(),
            Literal::from(false).into(),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b: false,
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
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
            Literal::from(true).into(),
        ),
        PropertyDefinition::MethodDefinition(
            interner.get_or_intern_static("b", utf16!("b")).into(),
            MethodDefinition::Ordinary(Function::new(
                None,
                FormalParameterList::default(),
                StatementList::default(),
            )),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b() {},
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
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
            Literal::from(true).into(),
        ),
        PropertyDefinition::MethodDefinition(
            interner.get_or_intern_static("b", utf16!("b")).into(),
            MethodDefinition::Ordinary(Function::new(None, parameters, StatementList::default())),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b(test) {}
         };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
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
            Literal::from(true).into(),
        ),
        PropertyDefinition::MethodDefinition(
            interner.get_or_intern_static("b", utf16!("b")).into(),
            MethodDefinition::Get(Function::new(
                None,
                FormalParameterList::default(),
                StatementList::default(),
            )),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            get b() {}
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
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
            Literal::from(true).into(),
        ),
        PropertyDefinition::MethodDefinition(
            interner.get_or_intern_static("b", utf16!("b")).into(),
            MethodDefinition::Set(Function::new(None, params, StatementList::default())),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            set b(test) {}
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
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
        interner.get_or_intern_static("get", utf16!("get")).into(),
        MethodDefinition::Ordinary(Function::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
        )),
    )];

    check_parser(
        "const x = {
            get() {}
         };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
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
        interner.get_or_intern_static("set", utf16!("set")).into(),
        MethodDefinition::Ordinary(Function::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
        )),
    )];

    check_parser(
        "const x = {
            set() {}
         };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
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

    check_parser(
        "const a = true;
            const x = { a };
        ",
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::from(true).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("x", utf16!("x")).into(),
                    Some(ObjectLiteral::from(object_properties).into()),
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

    check_parser(
        "const a = true;
            const b = false;
            const x = { a, b, };
        ",
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::from(true).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("b", utf16!("b")).into(),
                    Some(Literal::from(false).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("x", utf16!("x")).into(),
                    Some(ObjectLiteral::from(object_properties).into()),
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
            Literal::from(1).into(),
        ),
        PropertyDefinition::SpreadObject(
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        ),
    ];

    check_parser(
        "const x = { a: 1, ...b };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
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
        interner.get_or_intern_static("dive", utf16!("dive")).into(),
        MethodDefinition::Async(AsyncFunction::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
            false,
        )),
    )];

    check_parser(
        "const x = {
            async dive() {}
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
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
        interner
            .get_or_intern_static("vroom", utf16!("vroom"))
            .into(),
        MethodDefinition::AsyncGenerator(AsyncGenerator::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
            false,
        )),
    )];

    check_parser(
        "const x = {
            async* vroom() {}
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
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

#[test]
fn check_async_ordinary_method() {
    let interner = &mut Interner::default();

    let object_properties = vec![PropertyDefinition::MethodDefinition(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        MethodDefinition::Ordinary(Function::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
        )),
    )];

    check_parser(
        "const x = {
            async() {}
         };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
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
        Literal::from(true).into(),
    )];

    check_parser(
        "const x = {
            async: true
         };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}
