use crate::exec;

#[test]
fn construct_class() {
    let scenario = r#"
    class MyClass {
        constructor() {
            this.val = 5;
        }
    }
    let c = new MyClass();
    c.val;
    "#;

    assert_eq!(&exec(scenario), "5");
}

#[test]
fn call_on_class() {
    let scenario = r#"
    class MyClass {
        get_value() {
            return 15;
        }
    }
    let c = new MyClass();
    c.get_value();
    "#;

    assert_eq!(&exec(scenario), "15");
}

#[test]
fn call_static_on_class() {
    let scenario = r#"
    class MyClass {
        static get_value() {
            return 15;
        }
    }
    MyClass.get_value();
    "#;

    assert_eq!(&exec(scenario), "15");
}

#[test]
fn get_class_field() {
    let scenario = r#"
    class MyClass {
        constructor() {
            this.val = 20;
        }
        get_value() {
            return this.val;
        }
    }
    let c = new MyClass();
    c.get_value();
    "#;

    assert_eq!(&exec(scenario), "20");
}

#[test]
fn get_class_field_literal() {
    let scenario = r#"
    class MyClass {
        val = 30;
        get_value() {
            return this.val;
        }
    }
    let c = new MyClass();
    c.get_value();
    "#;

    assert_eq!(&exec(scenario), "30");
}
