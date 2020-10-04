use crate::exec;

#[test]
fn simple_try() {
    let scenario = r#"
        let a = 10;
        try {
            a = 20;
        } catch {
            a = 30;
        }

        a;
    "#;
    assert_eq!(&exec(scenario), "20");
}

#[test]
fn finally() {
    let scenario = r#"
        let a = 10;
        try {
            a = 20;
        } finally {
            a = 30;
        }

        a;
    "#;
    assert_eq!(&exec(scenario), "30");
}

#[test]
fn catch_finally() {
    let scenario = r#"
        let a = 10;
        try {
            a = 20;
        } catch {
            a = 40;
        } finally {
            a = 30;
        }

        a;
    "#;
    assert_eq!(&exec(scenario), "30");
}

#[test]
fn catch() {
    let scenario = r#"
        let a = 10;
        try {
            throw "error";
        } catch {
            a = 20;
        }

        a;
    "#;
    assert_eq!(&exec(scenario), "20");
}

#[test]
fn catch_binding() {
    let scenario = r#"
        let a = 10;
        try {
            throw 20;
        } catch(err) {
            a = err;
        }

        a;
    "#;
    assert_eq!(&exec(scenario), "20");
}

#[test]
fn catch_binding_finally() {
    let scenario = r#"
        let a = 10;
        try {
            throw 20;
        } catch(err) {
            a = err;
        } finally {
            a = 30;
        }

        a;
    "#;
    assert_eq!(&exec(scenario), "30");
}
