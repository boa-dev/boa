use crate::syntax::parser::tests::check_invalid;

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
