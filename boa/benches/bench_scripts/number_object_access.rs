pub static NUMBER_OBJECT_ACCESS: &str = r#"
new Number(
    new Number(
        new Number(
            new Number(100).valueOf() - 10.5
        ).valueOf() + 100
    ).valueOf() * 1.6
)
"#;
