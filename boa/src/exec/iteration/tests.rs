use crate::exec;

#[test]
fn while_loop_late_break() {
    // Ordering with statement before the break.
    let scenario = r#"
        let a = 1;
        while (a < 5) {
            a++;
            if (a == 3) {
                break;
            }
        }
        a;
    "#;

    assert_eq!(&exec(scenario), "3");
}

#[test]
fn while_loop_early_break() {
    // Ordering with statements after the break.
    let scenario = r#"
        let a = 1;
        while (a < 5) {
            if (a == 3) {
                break;
            }
            a++;
        }
        a;
    "#;

    assert_eq!(&exec(scenario), "3");
}

#[test]
fn while_loop_continue() {
    let scenario = r#"
        let a = 1;
        let b = 1;
        while (a < 5) {
            a++;
            if (a >= 3) {
                continue;
            }
            b++;
        }
        [a, b];
    "#;

    assert_eq!(&exec(scenario), "[ 5, 2 ]");
}

#[test]
fn for_loop_break() {
    let scenario = r#"
        let a = 1;
        for (; a < 5; a++) {
            if (a == 3) {
                break;
            }
        }
        a;
    "#;

    assert_eq!(&exec(scenario), "3");
}

#[test]
fn for_loop_return() {
    let scenario = r#"
    function foo() {
        for (let a = 1; a < 5; a++) {
            if (a == 3) {
                return a;
            }
        }
    }

    foo();
    "#;

    assert_eq!(&exec(scenario), "3");
}

#[test]
fn for_loop_continue() {
    let scenario = r#"
        let a = 1;
        let b = 1;
        for (; a < 5; a++) {
            if (a >= 3) {
                continue;
            }
            b++;
        }
        [a, b];
    "#;

    assert_eq!(&exec(scenario), "[ 5, 3 ]");
}

#[test]
fn do_loop_late_break() {
    // Ordering with statement before the break.
    let scenario = r#"
        let a = 1;
        do {
            a++;
            if (a == 3) {
                break;
            }
        } while (a < 5);
        a;
    "#;

    assert_eq!(&exec(scenario), "3");
}

#[test]
fn do_loop_early_break() {
    // Ordering with statements after the break.
    let scenario = r#"
        let a = 1;
        do {
            if (a == 3) {
                break;
            }
            a++;
        } while (a < 5);
        a;
    "#;

    assert_eq!(&exec(scenario), "3");
}

#[test]
fn do_loop_continue() {
    let scenario = r#"
        let a = 1;
        let b = 1;
        do {
            a++;
            if (a >= 3) {
                continue;
            }
            b++;
        } while (a < 5);
        [a, b];
    "#;

    assert_eq!(&exec(scenario), "[ 5, 2 ]");
}
