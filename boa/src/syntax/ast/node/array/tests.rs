use crate::parse;

#[test]
fn fmt() {
    let scenario = r#"
        let a = [1, 2, 3, "words", "more words"];
        let b = [];
        "#[1..] // Remove the preceding newline
        .lines()
        .map(|l| l.trim()) // Remove trailing whitespace from each line
        .collect::<Vec<&'static str>>()
        .join("\n");
    assert_eq!(format!("{}", parse(&scenario, false).unwrap()), scenario);
}
