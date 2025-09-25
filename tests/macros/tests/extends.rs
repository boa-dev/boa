//! Test for inheritance.
use boa_engine::interop::JsThis;
use boa_engine::{
    Context, Finalize, JsData, JsObject, JsValue, Source, Trace, boa_class, js_string,
};

#[test]
fn extends_js() {
    #[derive(Debug, Trace, Finalize, JsData)]
    struct X;

    #[boa_class(extends = "Base")]
    impl X {
        #[boa(constructor)]
        #[boa(length = 0)]
        fn new(_this: JsThis<JsObject>, _context: &mut Context) -> Self {
            Self
        }

        fn foo(JsThis(_this): JsThis<JsObject>, _context: &mut Context) -> u32 {
            0
        }
    }

    let context = &mut Context::default();
    context
        .eval(Source::from_bytes(
            r"
                class Base {
                    static baseStatic() { return 'hello'; }
                    baseFoo(a) { return a + 1 }
                }
            ",
        ))
        .expect("eval failed");

    context
        .register_global_class::<X>()
        .expect("global_class registration");

    let x = context
        .eval(Source::from_bytes(
            r#"
                new Uint8Array();
                (new X)
            "#,
        ))
        .expect("eval 2 failed");

    eprintln!(
        "x.foo = {}",
        x.as_object()
            .unwrap()
            .get(js_string!("foo"), context)
            .unwrap()
            .display()
    );
    eprintln!(
        "x.baseFoo = {}",
        x.as_object()
            .unwrap()
            .get(js_string!("baseFoo"), context)
            .unwrap()
            .display()
    );
}

#[test]
fn do_the_thing() {
    let context = &mut Context::default();
    let x = context
        .eval(Source::from_bytes(
            r"
        class Base {
            baseFoo(a) { return a + 1 }
        }

        class X extends Base {
            foo() { return this.baseFoo(1) + 1 }
        }

        new X
    ",
        ))
        .expect("eval failed");

    let obj = x.as_object().expect("x as object");
    eprintln!(
        "baseFoo: {:?}",
        obj.get(js_string!("baseFoo"), context)
            .expect("get baseFoo")
    );
}
