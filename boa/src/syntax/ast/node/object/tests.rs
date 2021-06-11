#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        let inst = {
            a: 5,
            b: "hello world",
            nested: {
                a: 5,
                b: 6,
            },
            say_hi: function() {
                console.log("hello!");
            }
        };
        "#,
    );
}
