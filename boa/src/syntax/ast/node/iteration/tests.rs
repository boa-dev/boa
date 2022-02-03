use crate::{check_output, exec, TestAction};

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
    let scenario = r#"
        var result = 0;
        for (i of [1, 2, 3]) {
            result = i;
        }
    "#;
    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("result", "3"),
        TestAction::TestEq("i", "3"),
    ]);
}

#[test]
fn for_of_loop_var() {
    let scenario = r#"
        var result = 0;
        for (var i of [1, 2, 3]) {
            result = i;
        }
    "#;
    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("result", "3"),
        TestAction::TestEq("i", "3"),
    ]);
}

#[test]
fn for_of_loop_let() {
    let scenario = r#"
        var result = 0;
        for (let i of [1, 2, 3]) {
            result = i;
        }
    "#;
    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("result", "3"),
        TestAction::TestEq(
            r#"
        try {
            i
        } catch(e) {
            e.toString()
        }
    "#,
            "\"ReferenceError: i is not defined\"",
        ),
    ]);
}

#[test]
fn for_of_loop_const() {
    let scenario = r#"
        var result = 0;
        for (let i of [1, 2, 3]) {
            result = i;
        }
    "#;
    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("result", "3"),
        TestAction::TestEq(
            r#"
        try {
            i
        } catch(e) {
            e.toString()
        }
    "#,
            "\"ReferenceError: i is not defined\"",
        ),
    ]);
}

#[test]
fn for_of_loop_break() {
    let scenario = r#"
        var result = 0;
        for (var i of [1, 2, 3]) {
            if (i > 1)
                break;
            result = i
        }
    "#;
    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("result", "1"),
        TestAction::TestEq("i", "2"),
    ]);
}

#[test]
fn for_of_loop_continue() {
    let scenario = r#"
        var result = 0;
        for (var i of [1, 2, 3]) {
            if (i == 3)
                continue;
            result = i
        }
    "#;
    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("result", "2"),
        TestAction::TestEq("i", "3"),
    ]);
}

#[test]
fn for_of_loop_return() {
    let scenario = r#"
        function foo() {
            for (i of [1, 2, 3]) {
                if (i > 1)
                    return i;
            }
        }
    "#;
    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("foo()", "2"),
    ]);
}

#[test]
fn for_loop_break_label() {
    let scenario = r#"
        var str = "";

        outer: for (let i = 0; i < 5; i++) {
            inner: for (let b = 0; b < 5; b++) {
                if (b === 2) {
                break outer;
                }
                str = str + b;
            }
            str = str + i;
        }
        str
    "#;
    assert_eq!(&exec(scenario), "\"01\"");
}

#[test]
fn for_loop_continue_label() {
    let scenario = r#"
    var count = 0;
    label: for (let x = 0; x < 10;) {
        while (true) {
            x++;
            count++;
            continue label;
        }
    }
    count
    "#;
    assert_eq!(&exec(scenario), "10");
}

#[test]
fn for_in_declaration() {
    let init = r#"
        let result = [];
        let obj = { a: "a", b: 2};
        var i;
        for (i in obj) {
            result = result.concat([i]);
        }
    "#;
    check_output(&[
        TestAction::Execute(init),
        TestAction::TestEq(
            "result.length === 2 && result.includes('a') && result.includes('b')",
            "true",
        ),
    ]);
}

#[test]
fn for_in_var_object() {
    let init = r#"
        let result = [];
        let obj = { a: "a", b: 2};
        for (var i in obj) {
            result = result.concat([i]);
        }
    "#;
    check_output(&[
        TestAction::Execute(init),
        TestAction::TestEq(
            "result.length === 2 && result.includes('a') && result.includes('b')",
            "true",
        ),
    ]);
}

#[test]
fn for_in_var_array() {
    let init = r#"
        let result = [];
        let arr = ["a", "b"];
        for (var i in arr) {
            result = result.concat([i]);
        }
    "#;
    check_output(&[
        TestAction::Execute(init),
        TestAction::TestEq(
            "result.length === 2 && result.includes('0') && result.includes('1')",
            "true",
        ),
    ]);
}

#[test]
fn for_in_let_object() {
    let init = r#"
        let result = [];
        let obj = { a: "a", b: 2};
        for (let i in obj) {
            result = result.concat([i]);
        }
    "#;
    check_output(&[
        TestAction::Execute(init),
        TestAction::TestEq(
            "result.length === 2 && result.includes('a') && result.includes('b')",
            "true",
        ),
    ]);
}

#[test]
fn for_in_const_array() {
    let init = r#"
        let result = [];
        let arr = ["a", "b"];
        for (const i in arr) {
            result = result.concat([i]);
        }
    "#;
    check_output(&[
        TestAction::Execute(init),
        TestAction::TestEq(
            "result.length === 2 && result.includes('0') && result.includes('1')",
            "true",
        ),
    ]);
}

#[test]
fn for_in_break_label() {
    let scenario = r#"
        var str = "";

        outer: for (let i in [1, 2]) {
            inner: for (let b in [2, 3, 4]) {
                if (b === "1") {
                    break outer;
                }
                str = str + b;
            }
            str = str + i;
        }
        str
    "#;
    assert_eq!(&exec(scenario), "\"0\"");
}

#[test]
fn for_in_continue_label() {
    let scenario = r#"
        var str = "";

        outer: for (let i in [1, 2]) {
            inner: for (let b in [2, 3, 4]) {
                if (b === "1") {
                    continue outer;
                }
                str = str + b;
            }
            str = str + i;
        }
        str
    "#;
    assert_eq!(&exec(scenario), "\"00\"");
}

#[test]
fn fmt() {
    // Labeled and unlabeled for in loops
    super::super::test_formatting(
        r#"
        var str = "";
        outer: for (let i in [1, 2]) {
            for (let b in [2, 3, 4]) {
                if (b === "1") {
                    continue outer;
                }
                str = str + b;
            };
            str = str + i;
        };
        str;
        "#,
    );
    // Labeled and unlabeled for loops
    super::super::test_formatting(
        r#"
        var str = "";
        outer: for (let i = 0; i < 10; ++i) {
            for (let j = 3; j < 6; ++j) {
                if (j === "1") {
                    continue outer;
                }
                str = str + j;
            };
            str = str + i;
        };
        str;
        "#,
    );
    // Labeled and unlabeled for of loops
    super::super::test_formatting(
        r#"
        for (i of [1, 2, 3]) {
            if (false) {
                break;
            }
        };
        label: for (i of [1, 2, 3]) {
            if (false) {
                break label;
            }
        };
        "#,
    );
    // Labeled and unlabeled do while loops
    super::super::test_formatting(
        r#"
        do {
            break;
        } while (true);
        label: do {
            break label;
        } while (true);
        "#,
    );
    // Labeled and unlabeled while loops
    super::super::test_formatting(
        r#"
        while (true) {
            break;
        }
        label: while (true) {
            break label;
        }
        "#,
    );
}
