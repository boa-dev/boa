#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        let other = {
            c: 10,
        };
        let inst = {
            a: 5,
            b: "hello world",
            nested: {
                a: 5,
                b: 6,
            },
            ...other,
            say_hi: function() {
                console.log("hello!");
            }
        };
        "#,
    );
}
