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
        fn new(this: JsThis<JsObject>, super: JsSuper, context: &mut Context) -> Self {
            eprintln!(
                "this.proto: {}",
                boa_engine::JsValue::from(this.0.prototype().unwrap()).display()
            );
            eprintln!(
                "this.proto.is_callable: {}",
                this.0.prototype().unwrap().is_callable()
            );
            this.0
                .prototype()
                .unwrap()
                .get(js_string!("constructor"), context)
                .unwrap()
                .as_callable()
                .unwrap()
                .construct(&[], Some(&this.0), context)
                .unwrap();

            Self
        }

        fn foo(JsThis(this): JsThis<JsObject>, context: &mut Context) -> u32 {
            eprintln!("this: {}", JsValue::new(this.clone()).display());

            eprintln!(
                "zthis.foo: {}",
                boa_engine::JsValue::from(this.get(js_string!("foo"), context).unwrap())
                    .display()
                    .to_string()
            );
            eprintln!(
                "zthis.baseFoo: {}",
                boa_engine::JsValue::from(this.get(js_string!("baseFoo"), context).unwrap())
                    .display()
                    .to_string()
            );
            eprintln!(
                "zthis.proto: {}",
                boa_engine::JsValue::from(this.prototype().unwrap()).display_obj(true)
            );

            this.get(js_string!("baseFoo"), context)
                .unwrap()
                .as_callable()
                .expect("as callable")
                .call(&this.clone().into(), &[JsValue::from(1)], context)
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
                (new X)
            "#,
        ))
        .expect("eval 2 failed");

    eprintln!("x: {}", x.display());
    eprintln!(
        "x.foo: {}",
        x.clone()
            .as_object()
            .unwrap()
            .get(js_string!("foo"), context)
            .unwrap()
            .display()
    );

    eprintln!(
        "x.proto: {}",
        JsValue::from(x.clone().as_object().unwrap().prototype().unwrap()).display()
    );
    eprintln!(
        "x.baseFoo: {}",
        x.clone()
            .as_object()
            .unwrap()
            .get(js_string!("baseFoo"), context)
            .unwrap()
            .display()
    );

    // assert_eq!(v.to_u32(context).expect("get value"), 3);
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
