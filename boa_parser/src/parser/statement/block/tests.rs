//! Block statement parsing tests.

use crate::parser::tests::check_script_parser;
use boa_ast::{
    declaration::{VarDeclaration, Variable},
    expression::{
        literal::Literal,
        operator::{
            assign::AssignOp,
            update::{UpdateOp, UpdateTarget},
            Assign, Update,
        },
        Call, Identifier,
    },
    function::{FormalParameterList, Function, FunctionBody},
    statement::{Block, Return},
    Declaration, Expression, Statement, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Helper function to check a block.
#[track_caller]
fn check_block<B>(js: &str, block: B, interner: &mut Interner)
where
    B: Into<Box<[StatementListItem]>>,
{
    check_script_parser(
        js,
        vec![Statement::Block(Block::from(block.into())).into()],
        interner,
    );
}

#[test]
fn empty() {
    check_block("{}", vec![], &mut Interner::default());
}

#[test]
fn non_empty() {
    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_block(
        r"{
            var a = 10;
            a++;
        }",
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    a.into(),
                    Some(Literal::from(10).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::Expression(Expression::from(Update::new(
                UpdateOp::IncrementPost,
                UpdateTarget::Identifier(Identifier::new(a)),
            )))
            .into(),
        ],
        interner,
    );

    let interner = &mut Interner::default();
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
            Declaration::Function(Function::new(
                Some(hello.into()),
                FormalParameterList::default(),
                FunctionBody::new(
                    vec![StatementListItem::Statement(Statement::Return(
                        Return::new(Some(Literal::from(10).into())),
                    ))]
                    .into(),
                ),
            ))
            .into(),
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    a.into(),
                    Some(Call::new(Identifier::new(hello).into(), Box::default()).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::Expression(Expression::from(Update::new(
                UpdateOp::IncrementPost,
                UpdateTarget::Identifier(Identifier::new(a)),
            )))
            .into(),
        ],
        interner,
    );
}

#[test]
fn hoisting() {
    let interner = &mut Interner::default();
    let hello = interner.get_or_intern_static("hello", utf16!("hello"));
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_block(
        r"{
            var a = hello();
            a++;

            function hello() { return 10 }
        }",
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    a.into(),
                    Some(Call::new(Identifier::new(hello).into(), Box::default()).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::Expression(Expression::from(Update::new(
                UpdateOp::IncrementPost,
                UpdateTarget::Identifier(Identifier::new(a)),
            )))
            .into(),
            Declaration::Function(Function::new(
                Some(hello.into()),
                FormalParameterList::default(),
                FunctionBody::new(
                    vec![StatementListItem::Statement(Statement::Return(
                        Return::new(Some(Literal::from(10).into())),
                    ))]
                    .into(),
                ),
            ))
            .into(),
        ],
        interner,
    );

    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_block(
        r"{
            a = 10;
            a++;

            var a;
        }",
        vec![
            Statement::Expression(Expression::from(Assign::new(
                AssignOp::Assign,
                Identifier::new(a).into(),
                Literal::from(10).into(),
            )))
            .into(),
            Statement::Expression(Expression::from(Update::new(
                UpdateOp::IncrementPost,
                UpdateTarget::Identifier(Identifier::new(a)),
            )))
            .into(),
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(a.into(), None)]
                    .try_into()
                    .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}
