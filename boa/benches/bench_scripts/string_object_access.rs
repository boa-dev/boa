pub static STRING_OBJECT_ACCESS: &str = r#"
new String(
    new String(
        new String(
            new String('Hello').valueOf() + new String(", world").valueOf()
        ).valueOf() + '!'
    ).valueOf()
).valueOf()
"#;
