use boa_interner::Interner;
use boa_macros::utf16;

use crate::syntax::{
    ast::{
        expression::{
            access::PropertyAccessField, literal::Literal, Identifier, Optional, OptionalItem,
            OptionalItemKind,
        },
        Expression, Statement,
    },
    parser::tests::{check_invalid, check_parser},
};

#[test]
fn simple() {
    let mut interner = Interner::default();

    check_parser(
        r#"5?.name"#,
        vec![Statement::Expression(
            Optional::new(
                Literal::Int(5).into(),
                vec![OptionalItem::new(
                    OptionalItemKind::SimplePropertyAccess {
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
    let mut interner = Interner::default();

    check_parser(
        r#"a?.b(true)?.["c"]"#,
        vec![Statement::Expression(
            Optional::new(
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                vec![
                    OptionalItem::new(
                        OptionalItemKind::SimplePropertyAccess {
                            field: PropertyAccessField::Const(
                                interner.get_or_intern_static("b", utf16!("b")),
                            ),
                        },
                        true,
                    ),
                    OptionalItem::new(
                        OptionalItemKind::Call {
                            args: vec![Expression::Literal(Literal::Bool(true))].into(),
                        },
                        false,
                    ),
                    OptionalItem::new(
                        OptionalItemKind::SimplePropertyAccess {
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
