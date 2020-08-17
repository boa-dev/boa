pub static REGEXP_LITERAL_CREATION: &str = r#"
(function () {
    let regExp = /hello/i;

    return regExp;
})();
"#;
