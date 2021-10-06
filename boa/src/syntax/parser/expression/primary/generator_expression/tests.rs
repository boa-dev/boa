use crate::syntax::{
    ast::{
        node::{Declaration, DeclarationList, GeneratorExpr, StatementList, Yield},
        Const,
    },
    parser::tests::check_parser,
};

#[test]
fn check_generator_function_expression() {
    check_parser(
        "const gen = function*() {
            yield 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "gen",
                Some(
                    GeneratorExpr::new::<Option<Box<str>>, _, StatementList>(
                        None,
                        [],
                        vec![Yield::new(Const::from(1), false).into()].into(),
                    )
                    .into(),
                ),
            )]
            .into(),
        )
        .into()],
    );
}

#[test]
fn check_generator_function_delegate_yield_expression() {
    check_parser(
        "const gen = function*() {
            yield* 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "gen",
                Some(
                    GeneratorExpr::new::<Option<Box<str>>, _, StatementList>(
                        None,
                        [],
                        vec![Yield::new(Const::from(1), true).into()].into(),
                    )
                    .into(),
                ),
            )]
            .into(),
        )
        .into()],
    );
}
