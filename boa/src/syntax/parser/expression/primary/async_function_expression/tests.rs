use crate::{
    syntax::{
        ast::{
            node::{AsyncFunctionExpr, Declaration, DeclarationList, Return, StatementList},
            Const,
        },
        parser::tests::check_parser,
    },
    Interner,
};

/// Checks async expression parsing.
#[test]
fn check_async_expression() {
    let mut interner = Interner::default();
    check_parser(
        "const add = async function() {
            return 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("add"),
                Some(
                    AsyncFunctionExpr::new::<_, _, StatementList>(
                        None,
                        [],
                        vec![Return::new(Const::from(1), None).into()].into(),
                    )
                    .into(),
                ),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn check_nested_async_expression() {
    let mut interner = Interner::default();
    check_parser(
        "const a = async function() {
            const b = async function() {
                return 1;
            };
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("a"),
                Some(
                    AsyncFunctionExpr::new::<_, _, StatementList>(
                        None,
                        [],
                        vec![DeclarationList::Const(
                            vec![Declaration::new_with_identifier(
                                interner.get_or_intern_static("b"),
                                Some(
                                    AsyncFunctionExpr::new::<_, _, StatementList>(
                                        None,
                                        [],
                                        vec![Return::new(Const::from(1), None).into()].into(),
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
        &mut interner,
    );
}
