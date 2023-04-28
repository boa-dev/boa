use crate::parser::tests::check_script_parser;
use boa_ast::{
    expression::{literal::Literal, Call, Identifier},
    function::{Class, ClassElement, FormalParameterList, Function},
    property::{MethodDefinition, PropertyName},
    Declaration, StatementList, Statement, Expression, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn check_async_ordinary_method() {
    let interner = &mut Interner::default();

    let elements = vec![ClassElement::MethodDefinition(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        MethodDefinition::Ordinary(Function::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
        )),
    )];

    check_script_parser(
        "class A {
            async() { }
         }
        ",
        [Declaration::Class(Class::new(
            Some(interner.get_or_intern_static("A", utf16!("A")).into()),
            None,
            None,
            elements.into(),
            true,
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_async_field_initialization() {
    let interner = &mut Interner::default();

    let elements = vec![ClassElement::FieldDefinition(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        Some(Literal::from(1).into()),
    )];

    check_script_parser(
        "class A {
            async
              = 1
         }
        ",
        [Declaration::Class(Class::new(
            Some(interner.get_or_intern_static("A", utf16!("A")).into()),
            None,
            None,
            elements.into(),
            true,
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_async_field() {
    let interner = &mut Interner::default();

    let elements = vec![ClassElement::FieldDefinition(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        None,
    )];

    check_script_parser(
        "class A {
            async
         }
        ",
        [Declaration::Class(Class::new(
            Some(interner.get_or_intern_static("A", utf16!("A")).into()),
            None,
            None,
            elements.into(),
            true,
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_new_target_with_property_access() {
    let interner = &mut Interner::default();

    let constructor_body = StatementList {
        statements: vec![
            Statement::Expression(
                Expression::Call(
                    Call::new(
                        Expression::Identifier(
                            Identifier::new(
                                interner.get_or_intern_static("console", utf16!("console")),
                            ),
                        ),
                        vec![
                            Expression::Member(
                                Box::new(Expression::NewTarget),
                                Box::new(Expression::Identifier(
                                    Identifier::new(
                                        interner.get_or_intern_static("name", utf16!("name")),
                                    ),
                                )),
                            ),
                        ]
                        .into_iter()
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    ),
                ),
            ),
        ]
        .into_iter()
        .map(StatementListItem::from)
        .collect::<Vec<_>>()
        .into_boxed_slice(),
        strict: false,
    };
    
    let elements = vec![ClassElement::MethodDefinition(
        PropertyName::Identifier(interner.get_or_intern_static("constructor", utf16!("constructor"))),
        MethodDefinition::Ordinary(Function::new(
            None,
            Vec::new(),
            constructor_body,
        )),
    )];

    check_script_parser(
        r#"
            class A {
                constructor() {
                    console.log(new.target.name);
                }
            }
            const a = new A();
        "#,
        [Declaration::Class(Class::new(
            Some(interner.get_or_intern_static("A", utf16!("A")).into()),
            None,
            None,
            elements.into(),
            true,
        ))
        .into()],
        interner,
    );
}