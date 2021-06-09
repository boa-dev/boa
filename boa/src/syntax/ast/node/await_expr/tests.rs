use crate::parse;

#[test]
fn fmt() {
    // TODO: `let a = await fn()` is invalid syntax as of writing. It should be tested here once implemented.
    let scenario = r#"
        await function_call();
        "#[1..] // Remove the preceding newline
        .lines()
        .map(|l| l.trim()) // Remove trailing whitespace from each line
        .collect::<Vec<&'static str>>()
        .join("\n");
    assert_eq!(format!("{}", parse(&scenario, false).unwrap()), scenario);
}
