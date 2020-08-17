pub static REGEXP_CREATION: &str = r#"
(function () {
    let regExp = new RegExp('hello', 'i');

    return regExp;
})();
"#;
