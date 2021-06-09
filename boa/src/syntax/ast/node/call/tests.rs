use crate::parse;

#[test]
fn fmt() {
    let scenario = r#"
        call_1(1, 2, 3);
        call_2("argument here");
        call_3();
        "#[1..] // Remove the preceding newline
        .lines()
        .map(|l| &l[8..]) // Remove preceding whitespace from each line
        .collect::<Vec<&'static str>>()
        .join("\n");
    assert_eq!(format!("{}", parse(&scenario, false).unwrap()), scenario);
}
