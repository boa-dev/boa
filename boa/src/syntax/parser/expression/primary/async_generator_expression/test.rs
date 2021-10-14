use crate::syntax::{
    ast::{
        node::{AsyncGeneratorExpr, Declaration, DeclarationList, Return, StatementList},
        Const,
    },
    parser::tests::check_parser,
}

///checks async generator expression parsing

#[test]
fn check_async_generator_expr() {
    check_parser(
        "const add = async *function(){
            return 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "add",
                Some(
                    AsyncGeneratorExpr::new::<Option<Box<str>>, _, StatementList>(
                        None,
                        [],
                        vec![Return::new::<_, _, Option<Box<str>>>(Const::from(1), None).into()].into(),
                    ).into()
                ),
            )].into(),
        ).into()]
    );
}

#[test]
fn check_nested_async_generator_expr() {
    check_parser(
        "const a = async *function() {
            const b = async *function() {
                return 1;
            };
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "a",
                Some(
                    AsyncGeneratorExpr::new::<Option<Box<str>>, _, StatementList>(
                        None,
                        [],
                        vec![DeclarationList::Const(
                            vec![Declaration::new_with_identifier(
                                "b",
                                Some(
                                    AsyncGeneratorExpr::new::<Option<Box<str>>, _, StatementList>(
                                        None,
                                        [],
                                        vec![Return::new::<_, _, Option<Box<str>>>(
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
    );
}
