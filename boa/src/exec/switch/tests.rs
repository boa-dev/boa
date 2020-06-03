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
