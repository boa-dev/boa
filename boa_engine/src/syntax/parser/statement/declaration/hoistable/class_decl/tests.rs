use crate::{
    string::utf16,
    syntax::{
        ast::{
            node::{
                declaration::class_decl::ClassElement as ClassElementNode,
                object::{MethodDefinition, PropertyName},
                Class, FormalParameterList, FunctionExpr, Node,
            },
            Const,
        },
        parser::tests::check_parser,
    },
};
use boa_interner::Interner;

#[test]
fn check_async_ordinary_method() {
    let mut interner = Interner::default();

    let elements = vec![ClassElementNode::MethodDefinition(
        PropertyName::Computed(Node::Const(Const::from(
            interner.get_or_intern_static("async", utf16!("async")),
        ))),
        MethodDefinition::Ordinary(FunctionExpr::new(
            None,
            FormalParameterList::default(),
            vec![],
        )),
    )];

    check_parser(
        "class A {
            async() { }
         }
        ",
        [Node::ClassDecl(Class::new(
            interner.get_or_intern_static("A", utf16!("A")),
            None,
            None,
            elements,
        ))],
        interner,
    );
}

#[test]
fn check_async_field_initialization() {
    let mut interner = Interner::default();

    let elements = vec![ClassElementNode::FieldDefinition(
        PropertyName::Computed(Node::Const(Const::from(
            interner.get_or_intern_static("async", utf16!("async")),
        ))),
        Some(Node::Const(Const::from(1))),
    )];

    check_parser(
        "class A {
            async
              = 1
         }
        ",
        [Node::ClassDecl(Class::new(
            interner.get_or_intern_static("A", utf16!("A")),
            None,
            None,
            elements,
        ))],
        interner,
    );
}

#[test]
fn check_async_field() {
    let mut interner = Interner::default();

    let elements = vec![ClassElementNode::FieldDefinition(
        PropertyName::Computed(Node::Const(Const::from(
            interner.get_or_intern_static("async", utf16!("async")),
        ))),
        None,
    )];

    check_parser(
        "class A {
            async
         }
        ",
        [Node::ClassDecl(Class::new(
            interner.get_or_intern_static("A", utf16!("A")),
            None,
            None,
            elements,
        ))],
        interner,
    );
}
