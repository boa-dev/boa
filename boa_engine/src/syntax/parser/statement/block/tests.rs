//! Block statement parsing tests.

use crate::syntax::{
    ast::{
        expression::{
            literal::Literal,
            operator::{assign::op::AssignOp, unary::op::UnaryOp, Assign, Unary},
            Call, Identifier,
        },
        function::{FormalParameterList, Function},
        statement::{
            declaration::{Declaration, DeclarationList},
            Block, Return,
        },
        Expression, Statement,
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Helper function to check a block.
#[track_caller]
fn check_block<B>(js: &str, block: B, interner: Interner)
where
    B: Into<Box<[Statement]>>,
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
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_block(
        r"{
            var a = 10;
            a++;
        }",
        vec![
            DeclarationList::Var(
                vec![Declaration::from_identifier(
                    a.into(),
                    Some(Literal::from(10).into()),
                )]
                .into(),
            )
            .into(),
            Expression::from(Unary::new(
                UnaryOp::IncrementPost,
                Identifier::new(a).into(),
            ))
            .into(),
        ],
        interner,
    );

    let mut interner = Interner::default();
    let hello = interner.get_or_intern_static("hello", utf16!("hello"));
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_block(
        r"{
            function hello() {
                return 10
            }

            var a = hello();
            a++;
        }",
        vec![
            Function::new(
                hello.into(),
                FormalParameterList::default(),
                vec![Return::new(Some(Literal::from(10).into()), None).into()].into(),
            )
            .into(),
            DeclarationList::Var(
                vec![Declaration::from_identifier(
                    a.into(),
                    Some(Call::new(Identifier::new(hello).into(), Box::default()).into()),
                )]
                .into(),
            )
            .into(),
            Expression::from(Unary::new(
                UnaryOp::IncrementPost,
                Identifier::new(a).into(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn hoisting() {
    let mut interner = Interner::default();
    let hello = interner.get_or_intern_static("hello", utf16!("hello"));
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_block(
        r"{
            var a = hello();
            a++;

            function hello() { return 10 }
        }",
        vec![
            Function::new(
                Some(hello),
                FormalParameterList::default(),
                vec![Return::new(Some(Literal::from(10).into()), None).into()].into(),
            )
            .into(),
            DeclarationList::Var(
                vec![Declaration::from_identifier(
                    a.into(),
                    Some(Call::new(Identifier::new(hello).into(), Box::default()).into()),
                )]
                .into(),
            )
            .into(),
            Expression::from(Unary::new(
                UnaryOp::IncrementPost,
                Identifier::new(a).into(),
            ))
            .into(),
        ],
        interner,
    );

    let mut interner = Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_block(
        r"{
            a = 10;
            a++;

            var a;
        }",
        vec![
            Expression::from(Assign::new(
                AssignOp::Assign,
                Identifier::new(a).into(),
                Literal::from(10).into(),
            ))
            .into(),
            Expression::from(Unary::new(
                UnaryOp::IncrementPost,
                Identifier::new(a).into(),
            ))
            .into(),
            DeclarationList::Var(vec![Declaration::from_identifier(a.into(), None)].into()).into(),
        ],
        interner,
    );
}
