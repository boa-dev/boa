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
        fn new() -> Self {
            Self
        }

        fn foo(this: JsThis<JsObject>, context: &mut Context) -> u32 {
            eprintln!("this: {this:?}");

            let baseFoo = this
                .0
                .get(js_string!("baseFoo"), context)
                .expect("getting baseFoo property");

            eprintln!("baseFoo: {baseFoo:?}");

            baseFoo
                .as_callable()
                .expect("as callable")
                .call(&this.0.clone().into(), &[JsValue::from(1)], context)
                .expect("baseFoo() call")
                .to_u32(context)
                .expect("to_u32")
                + 1
        }
    }

    let context = &mut Context::default();
    context
        .eval(Source::from_bytes(
            r"
                class Base {
                    baseFoo(a) { return a + 1 }
                }
            ",
        ))
        .expect("eval failed");

    context
        .register_global_class::<X>()
        .expect("global_class registration");

    let v = context
        .eval(Source::from_bytes(
            r#"
                (new X).foo()
            "#,
        ))
        .expect("eval 2 failed");

    assert_eq!(v.to_u32(context).expect("get value"), 3);
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
        "{:?}",
        obj.get(js_string!("baseFoo"), context)
            .expect("get baseFoo")
    );
}
