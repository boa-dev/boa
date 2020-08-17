pub static BOOLEAN_OBJECT_ACCESS: &str = r#"
new Boolean(
    !new Boolean(
        new Boolean(
            !(new Boolean(false).valueOf()) && (new Boolean(true).valueOf())
        ).valueOf()
    ).valueOf()
).valueOf()
"#;
