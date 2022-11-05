use boa_interner::Interner;
use boa_macros::utf16;

use crate::parser::tests::{check_invalid, check_parser};
use boa_ast::{
    expression::{
        access::PropertyAccessField, literal::Literal, Identifier, Optional, OptionalOperation,
        OptionalOperationKind,
    },
    Expression, Statement,
};

#[test]
fn simple() {
    let interner = &mut Interner::default();

    check_parser(
        r#"5?.name"#,
        vec![Statement::Expression(
            Optional::new(
                Literal::Int(5).into(),
                vec![OptionalOperation::new(
                    OptionalOperationKind::SimplePropertyAccess {
                        field: PropertyAccessField::Const(
                            interner.get_or_intern_static("name", utf16!("name")),
                        ),
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

    check_parser(
        r#"a?.b(true)?.["c"]"#,
        vec![Statement::Expression(
            Optional::new(
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                vec![
                    OptionalOperation::new(
                        OptionalOperationKind::SimplePropertyAccess {
                            field: PropertyAccessField::Const(
                                interner.get_or_intern_static("b", utf16!("b")),
                            ),
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
                                Literal::String(interner.get_or_intern_static("c", utf16!("c")))
                                    .into(),
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
    check_invalid("console.log?.`Hello`");
    check_invalid("console?.log`Hello`");
    check_invalid(
        r#"
const a = console?.log
`Hello`"#,
    );
}
