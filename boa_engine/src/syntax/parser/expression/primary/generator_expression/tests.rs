use crate::syntax::{
    ast::{
        node::{
            Declaration, DeclarationList, FormalParameterList, GeneratorExpr, StatementList, Yield,
        },
        Const,
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;

#[test]
fn check_generator_function_expression() {
    let mut interner = Interner::default();
    check_parser(
        "const gen = function*() {
            yield 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("gen"),
                Some(
                    GeneratorExpr::new::<_, _, StatementList>(
                        None,
                        FormalParameterList::default(),
                        vec![Yield::new(Const::from(1), false).into()].into(),
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
fn check_generator_function_delegate_yield_expression() {
    let mut interner = Interner::default();
    check_parser(
        "const gen = function*() {
            yield* 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("gen"),
                Some(
                    GeneratorExpr::new::<_, _, StatementList>(
                        None,
                        FormalParameterList::default(),
                        vec![Yield::new(Const::from(1), true).into()].into(),
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
