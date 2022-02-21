#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        function say_hello(msg) {
            if (msg === "") {
                return 0;
            }
            console.log("hello " + msg);
            return;
        };
        say_hello("");
        say_hello("world");
        "#,
    );
}
