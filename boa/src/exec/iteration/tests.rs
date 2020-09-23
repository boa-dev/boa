use crate::{exec, forward, Context};

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
fn for_of_loop_declaration() {
    let mut engine = Context::new();
    let scenario = r#"
        var result = 0;
        for (i of [1, 2, 3]) {
            result = i;
        }
    "#;
    engine.eval(scenario).unwrap();
    assert_eq!(&forward(&mut engine, "result"), "3");
    assert_eq!(&forward(&mut engine, "i"), "3");
}

#[test]
fn for_of_loop_var() {
    let mut engine = Context::new();
    let scenario = r#"
        var result = 0;
        for (var i of [1, 2, 3]) {
            result = i;
        }
    "#;
    engine.eval(scenario).unwrap();
    assert_eq!(&forward(&mut engine, "result"), "3");
    assert_eq!(&forward(&mut engine, "i"), "3");
}

#[test]
fn for_of_loop_let() {
    let mut engine = Context::new();
    let scenario = r#"
        var result = 0;
        for (let i of [1, 2, 3]) {
            result = i;
        }
    "#;
    engine.eval(scenario).unwrap();
    assert_eq!(&forward(&mut engine, "result"), "3");
    assert_eq!(
        &forward(
            &mut engine,
            r#"
        try {
            i
        } catch(e) {
            e.toString()
        }
    "#
        ),
        "\"ReferenceError: i is not defined\""
    );
}

#[test]
fn for_of_loop_const() {
    let mut engine = Context::new();
    let scenario = r#"
        var result = 0;
        for (let i of [1, 2, 3]) {
            result = i;
        }
    "#;
    engine.eval(scenario).unwrap();
    assert_eq!(&forward(&mut engine, "result"), "3");
    assert_eq!(
        &forward(
            &mut engine,
            r#"
        try {
            i
        } catch(e) {
            e.toString()
        }
    "#
        ),
        "\"ReferenceError: i is not defined\""
    );
}

#[test]
fn for_of_loop_break() {
    let mut engine = Context::new();
    let scenario = r#"
        var result = 0;
        for (var i of [1, 2, 3]) {
            if (i > 1)
                break;
            result = i
        }
    "#;
    engine.eval(scenario).unwrap();
    assert_eq!(&forward(&mut engine, "result"), "1");
    assert_eq!(&forward(&mut engine, "i"), "2");
}

#[test]
fn for_of_loop_continue() {
    let mut engine = Context::new();
    let scenario = r#"
        var result = 0;
        for (var i of [1, 2, 3]) {
            if (i == 3)
                continue;
            result = i
        }
    "#;
    engine.eval(scenario).unwrap();
    assert_eq!(&forward(&mut engine, "result"), "2");
    assert_eq!(&forward(&mut engine, "i"), "3");
}

#[test]
fn for_of_loop_return() {
    let mut engine = Context::new();
    let scenario = r#"
        function foo() {
            for (i of [1, 2, 3]) {
                if (i > 1)
                    return i;
            }
        }
    "#;
    engine.eval(scenario).unwrap();
    assert_eq!(&forward(&mut engine, "foo()"), "2");
}
