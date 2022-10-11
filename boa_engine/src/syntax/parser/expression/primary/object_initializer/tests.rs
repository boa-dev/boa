use crate::{
    string::utf16,
    syntax::{
        ast::{
            node::{
                object::{MethodDefinition, PropertyDefinition},
                AsyncFunctionExpr, AsyncGeneratorExpr, Declaration, DeclarationList,
                FormalParameter, FormalParameterList, FormalParameterListFlags, FunctionExpr,
                Identifier, Node, Object,
            },
            Const,
        },
        parser::tests::{check_invalid, check_parser},
    },
};
use boa_interner::Interner;

/// Checks object literal parsing.
#[test]
fn check_object_literal() {
    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::property(
            interner.get_or_intern_static("a", utf16!("a")),
            Const::from(true),
        ),
        PropertyDefinition::property(
            interner.get_or_intern_static("b", utf16!("b")),
            Const::from(false),
        ),
    ];

    check_parser(
        "const x = {
            a: true,
            b: false,
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
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
        PropertyDefinition::property(
            interner.get_or_intern_static("a", utf16!("a")),
            Const::from(true),
        ),
        PropertyDefinition::method_definition(
            MethodDefinition::Ordinary(FunctionExpr::new(
                None,
                FormalParameterList::default(),
                vec![],
            )),
            interner.get_or_intern_static("b", utf16!("b")),
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
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
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
        PropertyDefinition::property(
            interner.get_or_intern_static("a", utf16!("a")),
            Const::from(true),
        ),
        PropertyDefinition::method_definition(
            MethodDefinition::Ordinary(FunctionExpr::new(
                None,
                FormalParameterList {
                    parameters: Box::new([FormalParameter::new(
                        Declaration::new_with_identifier(
                            interner.get_or_intern_static("test", utf16!("test")),
                            None,
                        ),
                        false,
                    )]),
                    flags: FormalParameterListFlags::default(),
                    length: 1,
                },
                vec![],
            )),
            interner.get_or_intern_static("b", utf16!("b")),
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
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
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
        PropertyDefinition::property(
            interner.get_or_intern_static("a", utf16!("a")),
            Const::from(true),
        ),
        PropertyDefinition::method_definition(
            MethodDefinition::Get(FunctionExpr::new(
                None,
                FormalParameterList::default(),
                vec![],
            )),
            interner.get_or_intern_static("b", utf16!("b")),
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
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
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
        PropertyDefinition::property(
            interner.get_or_intern_static("a", utf16!("a")),
            Const::from(true),
        ),
        PropertyDefinition::method_definition(
            MethodDefinition::Set(FunctionExpr::new(
                None,
                FormalParameterList {
                    parameters: Box::new([FormalParameter::new(
                        Declaration::new_with_identifier(
                            interner.get_or_intern_static("test", utf16!("test")),
                            None,
                        ),
                        false,
                    )]),
                    flags: FormalParameterListFlags::default(),
                    length: 1,
                },
                vec![],
            )),
            interner.get_or_intern_static("b", utf16!("b")),
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
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
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

    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinition::Ordinary(FunctionExpr::new(
            None,
            FormalParameterList::default(),
            vec![],
        )),
        interner.get_or_intern_static("get", utf16!("get")),
    )];

    check_parser(
        "const x = {
            get() {}
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
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

    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinition::Ordinary(FunctionExpr::new(
            None,
            FormalParameterList::default(),
            vec![],
        )),
        interner.get_or_intern_static("set", utf16!("set")),
    )];

    check_parser(
        "const x = {
            set() {}
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
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

    let object_properties = vec![PropertyDefinition::property(
        interner.get_or_intern_static("a", utf16!("a")),
        Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
    )];

    check_parser(
        "const a = true;
            const x = { a };
        ",
        vec![
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("a", utf16!("a")),
                    Some(Const::from(true).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("x", utf16!("x")),
                    Some(Object::from(object_properties).into()),
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
        PropertyDefinition::property(
            interner.get_or_intern_static("a", utf16!("a")),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
        ),
        PropertyDefinition::property(
            interner.get_or_intern_static("b", utf16!("b")),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
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
                    interner.get_or_intern_static("a", utf16!("a")),
                    Some(Const::from(true).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("b", utf16!("b")),
                    Some(Const::from(false).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Const(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("x", utf16!("x")),
                    Some(Object::from(object_properties).into()),
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
        PropertyDefinition::property(
            interner.get_or_intern_static("a", utf16!("a")),
            Const::from(1),
        ),
        PropertyDefinition::spread_object(Identifier::new(
            interner.get_or_intern_static("b", utf16!("b")),
        )),
    ];

    check_parser(
        "const x = { a: 1, ...b };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
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

    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinition::Async(AsyncFunctionExpr::new(
            None,
            FormalParameterList::default(),
            vec![],
        )),
        interner.get_or_intern_static("dive", utf16!("dive")),
    )];

    check_parser(
        "const x = {
            async dive() {}
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
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

    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinition::AsyncGenerator(AsyncGeneratorExpr::new(
            None,
            FormalParameterList::default(),
            vec![],
        )),
        interner.get_or_intern_static("vroom", utf16!("vroom")),
    )];

    check_parser(
        "const x = {
            async* vroom() {}
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
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

    let object_properties = vec![PropertyDefinition::method_definition(
        MethodDefinition::Ordinary(FunctionExpr::new(
            None,
            FormalParameterList::default(),
            vec![],
        )),
        Node::Const(Const::from(
            interner.get_or_intern_static("async", utf16!("async")),
        )),
    )];

    check_parser(
        "const x = {
            async() {}
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
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

    let object_properties = vec![PropertyDefinition::property(
        Node::Const(Const::from(
            interner.get_or_intern_static("async", utf16!("async")),
        )),
        Const::from(true),
    )];

    check_parser(
        "const x = {
            async: true
         };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x", utf16!("x")),
                Some(Object::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}
