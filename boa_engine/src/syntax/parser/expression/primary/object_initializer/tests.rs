use crate::syntax::{
    ast::{
        expression::{
            literal::{Literal, ObjectLiteral},
            Identifier,
        },
        function::{
            AsyncFunction, AsyncGenerator, FormalParameter, FormalParameterList,
            FormalParameterListFlags, Function,
        },
        property::{MethodDefinition, PropertyDefinition, PropertyName},
        statement::{
            declaration::{Declaration, DeclarationList},
            StatementList,
        },
    },
    parser::tests::{check_invalid, check_parser},
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Checks object literal parsing.
#[test]
fn check_object_literal() {
    let mut interner = Interner::default();

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
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Tests short function syntax.
#[test]
fn check_object_short_function() {
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::from(true).into(),
        ),
        PropertyDefinition::MethodDefinition(
            MethodDefinition::Ordinary(Function::new(
                None,
                FormalParameterList::default(),
                StatementList::default(),
            )),
            interner.get_or_intern_static("b", utf16!("b")).into(),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b() {},
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Testing short function syntax with arguments.
#[test]
fn check_object_short_function_arguments() {
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::from(true).into(),
        ),
        PropertyDefinition::MethodDefinition(
            MethodDefinition::Ordinary(Function::new(
                None,
                FormalParameterList {
                    parameters: Box::new([FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("test", utf16!("test")).into(),
                            None,
                        ),
                        false,
                    )]),
                    flags: FormalParameterListFlags::default(),
                    length: 1,
                },
                StatementList::default(),
            )),
            interner.get_or_intern_static("b", utf16!("b")).into(),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b(test) {}
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_object_getter() {
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::from(true).into(),
        ),
        PropertyDefinition::MethodDefinition(
            MethodDefinition::Get(Function::new(
                None,
                FormalParameterList::default(),
                StatementList::default(),
            )),
            interner.get_or_intern_static("b", utf16!("b")).into(),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            get b() {}
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_object_setter() {
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::from(true).into(),
        ),
        PropertyDefinition::MethodDefinition(
            MethodDefinition::Set(Function::new(
                None,
                FormalParameterList {
                    parameters: Box::new([FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("test", utf16!("test")).into(),
                            None,
                        ),
                        false,
                    )]),
                    flags: FormalParameterListFlags::default(),
                    length: 1,
                },
                StatementList::default(),
            )),
            interner.get_or_intern_static("b", utf16!("b")).into(),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            set b(test) {}
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_object_short_function_get() {
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::MethodDefinition(
        MethodDefinition::Ordinary(Function::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
        )),
        interner.get_or_intern_static("get", utf16!("get")).into(),
    )];

    check_parser(
        "const x = {
            get() {}
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_object_short_function_set() {
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::MethodDefinition(
        MethodDefinition::Ordinary(Function::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
        )),
        interner.get_or_intern_static("set", utf16!("set")).into(),
    )];

    check_parser(
        "const x = {
            set() {}
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_object_shorthand_property_names() {
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::Property(
        interner.get_or_intern_static("a", utf16!("a")).into(),
        Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
    )];

    check_parser(
        "const a = true;
            const x = { a };
        ",
        vec![
            DeclarationList::Const(
                vec![Declaration::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::from(true).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::from_identifier(
                    interner.get_or_intern_static("x", utf16!("x")).into(),
                    Some(ObjectLiteral::from(object_properties).into()),
                )]
                .into(),
            )
            .into(),
        ],
        interner,
    );
}

#[test]
fn check_object_shorthand_multiple_properties() {
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
        ),
        PropertyDefinition::Property(
            interner.get_or_intern_static("b", utf16!("b")).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        ),
    ];

    check_parser(
        "const a = true;
            const b = false;
            const x = { a, b, };
        ",
        vec![
            DeclarationList::Const(
                vec![Declaration::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::from(true).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::from_identifier(
                    interner.get_or_intern_static("b", utf16!("b")).into(),
                    Some(Literal::from(false).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::from_identifier(
                    interner.get_or_intern_static("x", utf16!("x")).into(),
                    Some(ObjectLiteral::from(object_properties).into()),
                )]
                .into(),
            )
            .into(),
        ],
        interner,
    );
}

#[test]
fn check_object_spread() {
    let mut interner = Interner::default();

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
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_async_method() {
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::MethodDefinition(
        MethodDefinition::Async(AsyncFunction::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
        )),
        interner.get_or_intern_static("dive", utf16!("dive")).into(),
    )];

    check_parser(
        "const x = {
            async dive() {}
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_async_generator_method() {
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::MethodDefinition(
        MethodDefinition::AsyncGenerator(AsyncGenerator::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
        )),
        interner
            .get_or_intern_static("vroom", utf16!("vroom"))
            .into(),
    )];

    check_parser(
        "const x = {
            async* vroom() {}
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
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
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::MethodDefinition(
        MethodDefinition::Ordinary(Function::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
        )),
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
    )];

    check_parser(
        "const x = {
            async() {}
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_async_property() {
    let mut interner = Interner::default();

    let object_properties = vec![PropertyDefinition::Property(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        Literal::from(true).into(),
    )];

    check_parser(
        "const x = {
            async: true
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}
