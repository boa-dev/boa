use crate::parse;

#[test]
fn fmt() {
    let scenario = r#"
        {
            let a = function_call();
            console.log("hello");
        }
        another_statement();
        "#[1..] // Remove the preceding newline
        .lines()
        .map(|l| &l[8..]) // Remove preceding whitespace from each line
        .collect::<Vec<&'static str>>()
        .join("\n");
    assert_eq!(format!("{}", parse(&scenario, false).unwrap()), scenario);
}
