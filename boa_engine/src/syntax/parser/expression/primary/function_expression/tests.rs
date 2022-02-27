use crate::syntax::{
    ast::{
        node::{
            Declaration, DeclarationList, FormalParameterList, FunctionExpr, Return, StatementList,
        },
        Const,
    },
    parser::tests::check_parser,
};
use boa_interner::{Interner, Sym};

/// Checks async expression parsing.
#[test]
fn check_function_expression() {
    let mut interner = Interner::default();
    check_parser(
        "const add = function() {
            return 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("add"),
                Some(
                    FunctionExpr::new::<_, _, StatementList>(
                        None,
                        FormalParameterList::default(),
                        vec![Return::new::<_, _, Option<Sym>>(Const::from(1), None).into()].into(),
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
fn check_nested_function_expression() {
    let mut interner = Interner::default();
    check_parser(
        "const a = function() {
            const b = function() {
                return 1;
            };
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("a"),
                Some(
                    FunctionExpr::new::<_, _, StatementList>(
                        None,
                        FormalParameterList::default(),
                        vec![DeclarationList::Const(
                            vec![Declaration::new_with_identifier(
                                interner.get_or_intern_static("b"),
                                Some(
                                    FunctionExpr::new::<_, _, StatementList>(
                                        None,
                                        FormalParameterList::default(),
                                        vec![Return::new::<_, _, Option<Sym>>(
                                            Const::from(1),
                                            None,
                                        )
                                        .into()]
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
        &mut interner,
    );
}
