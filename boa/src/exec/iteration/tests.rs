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
fn break_out_of_inner_loop() {
    let scenario = r#"
        var a = 0, b = 0;
        for (let i = 0; i < 2; i++) {
            a++;
            for (let j = 0; j < 10; j++) {
                b++;
                if (j == 3)
                    break;
            }
            a++;
        }
        [a, b]
    "#;
    assert_eq!(&exec(scenario), "[ 4, 8 ]");
}

#[test]
fn continue_inner_loop() {
    let scenario = r#"
        var a = 0, b = 0;
        for (let i = 0; i < 2; i++) {
            a++;
            for (let j = 0; j < 10; j++) {
                if (j < 3)
                    continue;
                b++;
            }
            a++;
        }
        [a, b]
    "#;
    assert_eq!(&exec(scenario), "[ 4, 14 ]");
}

#[test]
fn for_loop_continue_out_of_switch() {
    let scenario = r#"
        var a = 0, b = 0, c = 0;
        for (let i = 0; i < 3; i++) {
            a++;
            switch (i) {
            case 0:
               continue;
               c++;
            case 1:
               continue;
            case 5:
               c++;
            }
            b++;
        }
        [a, b, c]
    "#;
    assert_eq!(&exec(scenario), "[ 3, 1, 0 ]");
}

#[test]
fn while_loop_continue() {
    let scenario = r#"
        var i = 0, a = 0, b = 0;
        while (i < 3) {
            i++;
            if (i < 2) {
               a++;
               continue;
            }
            b++;
        }
        [a, b]
    "#;
    assert_eq!(&exec(scenario), "[ 1, 2 ]");
}

#[test]
fn do_while_loop_continue() {
    let scenario = r#"
        var i = 0, a = 0, b = 0;
        do {
            i++;
            if (i < 2) {
               a++;
               continue;
            }
            b++;
        } while (i < 3)
        [a, b]
    "#;
    assert_eq!(&exec(scenario), "[ 1, 2 ]");
}

#[test]
fn for_loop_break_label() {
    let scenario = r#"
        var str = "";

        loop1: for (let i = 0; i < 5; i++) {
            loop2: for (let b = 0; b < 5; b++) {
                if (b === 2) {
                break loop1;
                }
                str = str + b;
            }
            str = str + i;
        }
        str
    "#;
    assert_eq!(&exec(scenario), "\"01\"")
}
