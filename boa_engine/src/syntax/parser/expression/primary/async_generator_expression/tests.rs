use crate::syntax::{
    ast::{
        node::{
            AsyncGeneratorExpr, Declaration, DeclarationList, FormalParameterList, Return,
            StatementList,
        },
        Const,
    },
    parser::tests::check_parser,
};
use boa_interner::{Interner, Sym};

///checks async generator expression parsing

#[test]
fn check_async_generator_expr() {
    let mut interner = Interner::default();
    let add = interner.get_or_intern_static("add");
    check_parser(
        "const add = async function*(){
            return 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                add,
                Some(
                    AsyncGeneratorExpr::new::<_, _, StatementList>(
<<<<<<< HEAD:boa_engine/src/syntax/parser/expression/primary/async_generator_expression/tests.rs
                        None,
=======
                        Some(add),
>>>>>>> 09bfabb0b0204b0534d4616927d869d0221a3edd:boa/src/syntax/parser/expression/primary/async_generator_expression/tests.rs
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
fn check_nested_async_generator_expr() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern_static("a");
    let b = interner.get_or_intern_static("b");
    check_parser(
        "const a = async function*() {
            const b = async function*() {
                return 1;
            };
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                a,
                Some(
                    AsyncGeneratorExpr::new::<_, _, StatementList>(
<<<<<<< HEAD:boa_engine/src/syntax/parser/expression/primary/async_generator_expression/tests.rs
                        None,
=======
                        Some(a),
>>>>>>> 09bfabb0b0204b0534d4616927d869d0221a3edd:boa/src/syntax/parser/expression/primary/async_generator_expression/tests.rs
                        FormalParameterList::default(),
                        vec![DeclarationList::Const(
                            vec![Declaration::new_with_identifier(
                                b,
                                Some(
                                    AsyncGeneratorExpr::new::<_, _, StatementList>(
<<<<<<< HEAD:boa_engine/src/syntax/parser/expression/primary/async_generator_expression/tests.rs
                                        None,
=======
                                        Some(b),
>>>>>>> 09bfabb0b0204b0534d4616927d869d0221a3edd:boa/src/syntax/parser/expression/primary/async_generator_expression/tests.rs
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
