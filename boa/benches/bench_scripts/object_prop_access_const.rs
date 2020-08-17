pub static OBJECT_PROP_ACCESS_CONST: &str = r#"
(function () {
    let test = {
        my_prop: "hello",
        another: 65,
    };

    return test.my_prop;
})();
"#;
