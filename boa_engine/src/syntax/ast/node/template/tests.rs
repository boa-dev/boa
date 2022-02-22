use crate::exec;

#[test]
fn template_literal() {
    let scenario = r#"
        let a = 10;
        `result: ${a} and ${a+10}`;
        "#;

    assert_eq!(&exec(scenario), "\"result: 10 and 20\"");
}

#[test]
fn tagged_template() {
    let scenario = r#"
        function tag(t, ...args) {
           let a = []
           a = a.concat([t[0], t[1], t[2]]);
           a = a.concat([t.raw[0], t.raw[1], t.raw[2]]);
           a = a.concat([args[0], args[1]]);
           return a
        }
        let a = 10;
        tag`result: ${a} \x26 ${a+10}`;
        "#;

    assert_eq!(
        &exec(scenario),
        r#"[ "result: ", " & ", "", "result: ", " \x26 ", "", 10, 20 ]"#
    );
}

#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        function tag(t, ...args) {
            let a = [];
            a = a.concat([t[0], t[1], t[2]]);
            a = a.concat([t.raw[0], t.raw[1], t.raw[2]]);
            a = a.concat([args[0], args[1]]);
            return a;
        };
        let a = 10;
        tag`result: ${a} \x26 ${a + 10}`;
        "#,
    );
}
