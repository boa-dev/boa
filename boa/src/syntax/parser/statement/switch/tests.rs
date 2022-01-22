use crate::{
    syntax::{
        ast::{
            node::{
                Break, Call, Case, Declaration, DeclarationList, GetConstField, Identifier, Node,
                Switch,
            },
            Const,
        },
        parser::tests::{check_invalid, check_parser},
    },
    Interner,
};

/// Checks parsing malformed switch with no closeblock.
#[test]
fn check_switch_no_closeblock() {
    check_invalid(
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
    check_invalid(
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
    check_invalid(
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
    check_invalid(
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
    check_invalid(
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
    check_invalid(
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

/// Example of JS code https://jsfiddle.net/zq6jx47h/4/.
#[test]
fn check_seperated_switch() {
    let s = r#"
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
        "#;

    let mut interner = Interner::new();
    check_parser(
        s,
        vec![
            DeclarationList::Let(
                vec![Declaration::new_with_identifier(
                    "a",
                    Node::from(Const::from(10)),
                )]
                .into(),
            )
            .into(),
            Switch::new(
                Identifier::from("a"),
                vec![
                    Case::new(
                        Const::from(5),
                        vec![
                            Call::new(
                                GetConstField::new(Identifier::from("console"), "log"),
                                vec![Node::from(Const::from(5))],
                            )
                            .into(),
                            Break::new::<_, Box<str>>(None).into(),
                        ],
                    ),
                    Case::new(
                        Const::from(10),
                        vec![
                            Call::new(
                                GetConstField::new(Identifier::from("console"), "log"),
                                vec![Node::from(Const::from(10))],
                            )
                            .into(),
                            Break::new::<_, Box<str>>(None).into(),
                        ],
                    ),
                ],
                Some(vec![Call::new(
                    GetConstField::new(Identifier::from("console"), "log"),
                    vec![Node::from(Const::from("Default"))],
                )
                .into()]),
            )
            .into(),
        ],
        &mut interner,
    );
}
