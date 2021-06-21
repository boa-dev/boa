use crate::{exec, Context};

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

#[test]
fn getter() {
    let scenario = r#"
    class MyClass {
        val = 30;
        get a() {
            return this.val + 10;
        }
    }
    let c = new MyClass();
    c.a;
    "#;

    assert_eq!(&exec(scenario), "40");

    // Make sure that a() is executed at the correct time (not when the class is built).
    let scenario = r#"
    class MyClass {
        val = 30;
        get a() {
            return this.val + 10;
        }
    }
    let c = new MyClass();
    c.val = 50;
    c.a;
    "#;

    assert_eq!(&exec(scenario), "60");
}

#[test]
fn setter() {
    let scenario = r#"
    class MyClass {
        set a(val) {
            this.val = val - 10;
        }
    }
    let c = new MyClass();
    c.a = 30;
    c.val;
    "#;

    assert_eq!(&exec(scenario), "20");

    // Make sure that a() is executed at the correct time (not when the class is built).
    let scenario = r#"
    class MyClass {
        set a(val) {
            this.val = val - 10;
        }
    }
    let c = new MyClass();
    c.a = 50;
    c.val
    "#;

    assert_eq!(&exec(scenario), "40");
}

#[test]
fn redeclaration() {
    let js = r#"
    let MyClass = 5;
    class MyClass {}
    "#;

    assert!(Context::new().eval(js).is_err());
}

#[test]
fn fmt_test() {
    super::super::super::test_formatting(
        r#"
        class Hello {
            a = 5;
            say_hi(a, ...b) {
                console.log("Hello" + a);
            }
            static c = 5;
            static static_say_hi(a, ...b) {
                console.log("Hello" + a);
            }
        };
        "#,
    );
}
