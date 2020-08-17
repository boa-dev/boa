pub static OBJECT_CREATION: &str = r#"
(function () {
    let test = {
        my_prop: "hello",
        another: 65,
    };

    return test;
})();
"#;
