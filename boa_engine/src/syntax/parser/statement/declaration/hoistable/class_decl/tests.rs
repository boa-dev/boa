use crate::syntax::{
    ast::{
        expression::literal::Literal,
        function::{Class, ClassElement, FormalParameterList, Function},
        property::{MethodDefinition, PropertyName},
        statement::StatementList,
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn check_async_ordinary_method() {
    let mut interner = Interner::default();

    let elements = vec![ClassElement::MethodDefinition(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        MethodDefinition::Ordinary(Function::new(
            None,
            FormalParameterList::default(),
            StatementList::default(),
        )),
    )];

    check_parser(
        "class A {
            async() { }
         }
        ",
        [Class::new(
            Some(interner.get_or_intern_static("A", utf16!("A"))),
            None,
            None,
            elements.into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_async_field_initialization() {
    let mut interner = Interner::default();

    let elements = vec![ClassElement::FieldDefinition(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        Some(Literal::from(1).into()),
    )];

    check_parser(
        "class A {
            async
              = 1
         }
        ",
        [Class::new(
            Some(interner.get_or_intern_static("A", utf16!("A"))),
            None,
            None,
            elements.into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_async_field() {
    let mut interner = Interner::default();

    let elements = vec![ClassElement::FieldDefinition(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        None,
    )];

    check_parser(
        "class A {
            async
         }
        ",
        [Class::new(
            interner.get_or_intern_static("A", utf16!("A")).into(),
            None,
            None,
            elements.into(),
        )
        .into()],
        interner,
    );
}
