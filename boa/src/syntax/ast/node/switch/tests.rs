use crate::exec;

#[test]
fn single_case_switch() {
    let scenario = r#"
        let a = 10;
        switch (a) {
            case 10:
                a = 20;
                break;
        }
        
        a;
    "#;
    assert_eq!(&exec(scenario), "20");
}

#[test]
fn no_cases_switch() {
    let scenario = r#"
        let a = 10;
        switch (a) {
        }
        
        a;
    "#;
    assert_eq!(&exec(scenario), "10");
}

#[test]
fn no_true_case_switch() {
    let scenario = r#"
        let a = 10;
        switch (a) {
            case 5:
                a = 15;
                break;
        }
        
        a;
    "#;
    assert_eq!(&exec(scenario), "10");
}

#[test]
fn two_case_switch() {
    let scenario = r#"
        let a = 10;
        switch (a) {
            case 5:
                a = 15;
                break;
            case 10:
                a = 20;
                break;
        }
        
        a;
    "#;
    assert_eq!(&exec(scenario), "20");
}

#[test]
fn two_case_no_break_switch() {
    let scenario = r#"
        let a = 10;
        let b = 10;

        switch (a) {
            case 10:
                a = 150;
            case 20:
                b = 150;
                break;
        }
        
        a + b;
    "#;
    assert_eq!(&exec(scenario), "300");
}

#[test]
fn three_case_partial_fallthrough() {
    let scenario = r#"
        let a = 10;
        let b = 10;

        switch (a) {
            case 10:
                a = 150;
            case 20:
                b = 150;
                break;
            case 15:
                b = 1000;
                break;
        }
        
        a + b;
    "#;
    assert_eq!(&exec(scenario), "300");
}

#[test]
fn default_taken_switch() {
    let scenario = r#"
        let a = 10;

        switch (a) {
            case 5:
                a = 150;
                break;
            default:
                a = 70;
        }
        
        a;
    "#;
    assert_eq!(&exec(scenario), "70");
}

#[test]
fn default_not_taken_switch() {
    let scenario = r#"
        let a = 5;

        switch (a) {
            case 5:
                a = 150;
                break;
            default:
                a = 70;
        }
        
        a;
    "#;
    assert_eq!(&exec(scenario), "150");
}

#[test]
fn string_switch() {
    let scenario = r#"
        let a = "hello";

        switch (a) {
            case "hello":
                a = "world";
                break;
            default:
                a = "hi";
        }
        
        a;
    "#;
    assert_eq!(&exec(scenario), "\"world\"");
}

#[test]
fn bigger_switch_example() {
    let expected = [
        "\"Mon\"",
        "\"Tue\"",
        "\"Wed\"",
        "\"Thurs\"",
        "\"Fri\"",
        "\"Sat\"",
        "\"Sun\"",
    ];

    for (i, val) in expected.iter().enumerate() {
        let scenario = format!(
            r#"
            let a = {i};
            let b = "unknown";

            switch (a) {{
                case 0:
                    b = "Mon";
                    break;
                case 1:
                    b = "Tue";
                    break;
                case 2:
                    b = "Wed";
                    break;
                case 3:
                    b = "Thurs";
                    break;
                case 4:
                    b = "Fri";
                    break;
                case 5:
                    b = "Sat";
                    break;
                case 6:
                    b = "Sun";
                    break; 
            }}

            b;

            "#,
        );

        assert_eq!(&exec(&scenario), val);
    }
}

#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        let a = 3;
        let b = "unknown";
        switch (a) {
            case 0:
                b = "Mon";
                break;
            case 1:
                b = "Tue";
                break;
            case 2:
                b = "Wed";
                break;
            case 3:
                b = "Thurs";
                break;
            case 4:
                b = "Fri";
                break;
            case 5:
                b = "Sat";
                break;
            case 6:
                b = "Sun";
                break;
            default:
                b = "Unknown";
        }
        b;
        "#,
    );
}
