use crate::syntax::{
    ast::{
        expression::literal::Literal,
        function::{AsyncGenerator, FormalParameterList},
        statement::{
            declaration::{Declaration, DeclarationList},
            Return,
        },
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;
use boa_macros::utf16;

///checks async generator expression parsing

#[test]
fn check_async_generator_expr() {
    let mut interner = Interner::default();
    let add = interner.get_or_intern_static("add", utf16!("add"));
    check_parser(
        "const add = async function*(){
            return 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                add.into(),
                Some(
                    AsyncGenerator::new(
                        Some(add.into()),
                        FormalParameterList::default(),
                        vec![Return::new(Some(Literal::from(1).into()), None).into()].into(),
                    )
                    .into(),
                ),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_nested_async_generator_expr() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    let b = interner.get_or_intern_static("b", utf16!("b"));
    check_parser(
        "const a = async function*() {
            const b = async function*() {
                return 1;
            };
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                a.into(),
                Some(
                    AsyncGenerator::new(
                        Some(a.into()),
                        FormalParameterList::default(),
                        vec![DeclarationList::Const(
                            vec![Declaration::from_identifier(
                                b.into(),
                                Some(
                                    AsyncGenerator::new(
                                        Some(b.into()),
                                        FormalParameterList::default(),
                                        vec![
                                            Return::new(Some(Literal::from(1).into()), None).into()
                                        ]
                                        .into(),
                                    )
                                    .into(),
                                ),
                            )]
                            .into(),
                        )
                        .into()]
                        .into(),
                    )
                    .into(),
                ),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}
