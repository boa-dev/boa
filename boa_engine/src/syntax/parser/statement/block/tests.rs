//! Block statement parsing tests.

use crate::syntax::{
    ast::{
        node::{
            Assign, Block, Call, Declaration, DeclarationList, FormalParameterList, FunctionDecl,
            Identifier, Node, Return, UnaryOp,
        },
        op, Const,
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;

/// Helper function to check a block.
#[track_caller]
fn check_block<B>(js: &str, block: B, interner: Interner)
where
    B: Into<Box<[Node]>>,
{
    check_parser(js, vec![Block::from(block.into()).into()], interner);
}

#[test]
fn empty() {
    check_block("{}", vec![], Interner::default());
}

#[test]
fn non_empty() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern_static("a");
    check_block(
        r"{
            var a = 10;
            a++;
        }",
        vec![
            DeclarationList::Var(
                vec![Declaration::new_with_identifier(
                    a,
                    Some(Const::from(10).into()),
                )]
                .into(),
            )
            .into(),
            UnaryOp::new(op::UnaryOp::IncrementPost, Identifier::new(a)).into(),
        ],
        interner,
    );

    let mut interner = Interner::default();
    let hello = interner.get_or_intern_static("hello");
    let a = interner.get_or_intern_static("a");
    check_block(
        r"{
            function hello() {
                return 10
            }

            var a = hello();
            a++;
        }",
        vec![
            FunctionDecl::new(
                hello,
                FormalParameterList::default(),
                vec![Return::new(Const::from(10), None).into()],
            )
            .into(),
            DeclarationList::Var(
                vec![Declaration::new_with_identifier(
                    a,
                    Node::from(Call::new(Identifier::new(hello), vec![])),
                )]
                .into(),
            )
            .into(),
            UnaryOp::new(op::UnaryOp::IncrementPost, Identifier::new(a)).into(),
        ],
        interner,
    );
}

#[test]
fn hoisting() {
    let mut interner = Interner::default();
    let hello = interner.get_or_intern_static("hello");
    let a = interner.get_or_intern_static("a");
    check_block(
        r"{
            var a = hello();
            a++;

            function hello() { return 10 }
        }",
        vec![
            FunctionDecl::new(
                hello,
                FormalParameterList::default(),
                vec![Return::new(Const::from(10), None).into()],
            )
            .into(),
            DeclarationList::Var(
                vec![Declaration::new_with_identifier(
                    a,
                    Node::from(Call::new(Identifier::new(hello), vec![])),
                )]
                .into(),
            )
            .into(),
            UnaryOp::new(op::UnaryOp::IncrementPost, Identifier::new(a)).into(),
        ],
        interner,
    );

    let mut interner = Interner::default();
    let a = interner.get_or_intern_static("a");
    check_block(
        r"{
            a = 10;
            a++;

            var a;
        }",
        vec![
            Assign::new(Identifier::new(a), Const::from(10)).into(),
            UnaryOp::new(op::UnaryOp::IncrementPost, Identifier::new(a)).into(),
            DeclarationList::Var(vec![Declaration::new_with_identifier(a, None)].into()).into(),
        ],
        interner,
    );
}
