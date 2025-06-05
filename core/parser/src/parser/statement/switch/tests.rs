use crate::parser::tests::{check_invalid_script, check_script_parser};
use boa_ast::{
    declaration::{LexicalDeclaration, Variable},
    expression::{access::SimplePropertyAccess, literal::Literal, Call, Identifier},
    statement::{Break, Case, Switch},
    Declaration, Expression, Span, Statement,
};
use boa_interner::Interner;
use boa_macros::utf16;
use indoc::indoc;

const PSEUDO_LINEAR_POS: boa_ast::LinearPosition = boa_ast::LinearPosition::new(0);

/// Checks parsing malformed switch with no closeblock.
#[test]
fn check_switch_no_closeblock() {
    check_invalid_script(
        r#"
        let a = 10;
        switch (a) {
            case 10:
                a = 20;
                break;

        "#,
    );
}

/// Checks parsing malformed switch in which a case is started but not finished.
#[test]
fn check_switch_case_unclosed() {
    check_invalid_script(
        r#"
        let a = 10;
        switch (a) {
            case 10:
                a = 20;

        "#,
    );
}

/// Checks parsing malformed switch with 2 defaults.
#[test]
fn check_switch_two_default() {
    check_invalid_script(
        r#"
        let a = 10;
        switch (a) {
            default:
                a = 20;
                break;
            default:
                a = 30;
                break;
        }
        "#,
    );
}

/// Checks parsing malformed switch with no expression.
#[test]
fn check_switch_no_expr() {
    check_invalid_script(
        r#"
        let a = 10;
        switch {
            default:
                a = 20;
                break;
        }
        "#,
    );
}

/// Checks parsing malformed switch with an unknown label.
#[test]
fn check_switch_unknown_label() {
    check_invalid_script(
        r#"
        let a = 10;
        switch (a) {
            fake:
                a = 20;
                break;
        }
        "#,
    );
}

/// Checks parsing malformed switch with two defaults that are seperated by cases.
#[test]
fn check_switch_seperated_defaults() {
    check_invalid_script(
        r#"
        let a = 10;
        switch (a) {
            default:
                a = 20;
                break;
            case 10:
                a = 60;
                break;
            default:
                a = 30;
                break;
        }
        "#,
    );
}

/// Example of JS code <https://jsfiddle.net/zq6jx47h/4/>.
#[test]
fn check_separated_switch() {
    let s = indoc! {r#"
        let a = 10;

        switch

        (a)

        {

        case

        5

        :

        console.log(5);

        break;

        case

        10

        :

        console.log(10);

        break;

        default

        :

        console.log("Default")

        }
    "#};

    let interner = &mut Interner::default();
    let log = interner.get_or_intern_static("log", utf16!("log"));
    let console = interner.get_or_intern_static("console", utf16!("console"));
    let a = interner.get_or_intern_static("a", utf16!("a"));

    check_script_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    a.into(),
                    Some(Literal::new(10, Span::new((1, 9), (1, 11))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::Switch(Switch::new(
                Identifier::new(a).into(),
                vec![
                    Case::new(
                        Literal::new(5, Span::new((11, 1), (11, 2))).into(),
                        (
                            vec![
                                Statement::Expression(Expression::from(Call::new(
                                    Expression::PropertyAccess(
                                        SimplePropertyAccess::new(
                                            Identifier::new(console).into(),
                                            log,
                                        )
                                        .into(),
                                    ),
                                    vec![Literal::new(5, Span::new((15, 13), (15, 14))).into()]
                                        .into(),
                                )))
                                .into(),
                                Statement::Break(Break::new(None)).into(),
                            ],
                            PSEUDO_LINEAR_POS,
                        )
                            .into(),
                    ),
                    Case::new(
                        Literal::new(10, Span::new((21, 1), (21, 3))).into(),
                        (
                            vec![
                                Statement::Expression(Expression::from(Call::new(
                                    Expression::PropertyAccess(
                                        SimplePropertyAccess::new(
                                            Identifier::new(console).into(),
                                            log,
                                        )
                                        .into(),
                                    ),
                                    vec![Literal::new(10, Span::new((25, 13), (25, 15))).into()]
                                        .into(),
                                )))
                                .into(),
                                Statement::Break(Break::new(None)).into(),
                            ],
                            PSEUDO_LINEAR_POS,
                        )
                            .into(),
                    ),
                    Case::default(
                        (
                            vec![Statement::Expression(Expression::from(Call::new(
                                Expression::PropertyAccess(
                                    SimplePropertyAccess::new(Identifier::new(console).into(), log)
                                        .into(),
                                ),
                                vec![Literal::new(
                                    interner.get_or_intern_static("Default", utf16!("Default")),
                                    Span::new((33, 13), (33, 22)),
                                )
                                .into()]
                                .into(),
                            )))
                            .into()],
                            PSEUDO_LINEAR_POS,
                        )
                            .into(),
                    ),
                ]
                .into(),
            ))
            .into(),
        ],
        interner,
    );
}
