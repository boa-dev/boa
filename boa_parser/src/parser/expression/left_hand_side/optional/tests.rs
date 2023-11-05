use crate::parser::tests::{check_invalid_script, check_script_parser};

use boa_ast::{
    expression::{
        access::PropertyAccessField, literal::Literal, Identifier, Optional, OptionalOperation,
        OptionalOperationKind,
    },
    Expression, Statement,
};
use boa_interner::Interner;

#[test]
fn simple() {
    let interner = &mut Interner::default();

    check_script_parser(
        r#"5?.name"#,
        vec![Statement::Expression(
            Optional::new(
                Literal::Int(5).into(),
                vec![OptionalOperation::new(
                    OptionalOperationKind::SimplePropertyAccess {
                        field: PropertyAccessField::Const(interner.get_or_intern("name")),
                    },
                    true,
                )]
                .into(),
            )
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn complex_chain() {
    let interner = &mut Interner::default();

    check_script_parser(
        r#"a?.b(true)?.["c"]"#,
        vec![Statement::Expression(
            Optional::new(
                Identifier::new(interner.get_or_intern("a")).into(),
                vec![
                    OptionalOperation::new(
                        OptionalOperationKind::SimplePropertyAccess {
                            field: PropertyAccessField::Const(interner.get_or_intern("b")),
                        },
                        true,
                    ),
                    OptionalOperation::new(
                        OptionalOperationKind::Call {
                            args: vec![Expression::Literal(Literal::Bool(true))].into(),
                        },
                        false,
                    ),
                    OptionalOperation::new(
                        OptionalOperationKind::SimplePropertyAccess {
                            field: PropertyAccessField::Expr(Box::new(
                                Literal::String(interner.get_or_intern("c")).into(),
                            )),
                        },
                        true,
                    ),
                ]
                .into(),
            )
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn reject_templates() {
    check_invalid_script("console.log?.`Hello`");
    check_invalid_script("console?.log`Hello`");
    check_invalid_script(
        r#"
const a = console?.log
`Hello`"#,
    );
}

#[test]
fn private_identifier_early_error() {
    check_invalid_script("this?.#a");
    check_invalid_script("this.#a");
    check_invalid_script("this?.a?.#a");
    check_invalid_script("this.a.#a");
}
