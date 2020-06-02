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