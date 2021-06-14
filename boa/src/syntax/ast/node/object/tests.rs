#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        let other = {
            c: 10,
        };
        let inst = {
            val: 5,
            b: "hello world",
            nested: {
                a: 5,
                b: 6,
            },
            ...other,
            say_hi: function() {
                console.log("hello!");
            },
            get a() {
                return this.val + 1;
            },
            set a(new_value) {
                this.val = new_value;
            },
            say_hello(msg) {
                console.log("hello " + msg);
            },
        };
        inst.a = 20;
        inst.a;
        inst.say_hello("humans");
        "#,
    );
}
