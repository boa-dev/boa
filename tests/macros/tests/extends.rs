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
        fn new(this: JsThis<JsObject>) -> Self {
            eprintln!(
                "this.proto: {}",
                boa_engine::JsValue::from(this.0.prototype().unwrap()).display_obj(true)
            );
            eprintln!(
                "this.proto.is_callable: {}",
                this.0.prototype().unwrap().is_callable()
            );
            this.0
                .prototype()
                .unwrap()
                .call(&this.0.clone().into(), &[], &mut Context::default());

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
fn extends_js_explicit() {
    #[derive(Debug, Trace, Finalize, JsData)]
    struct X;

    #[allow(clippy::needless_pass_by_value)]
    impl X {
        fn new(this: JsThis<JsObject>) -> Self {
            eprintln!(
                "this.proto: {}",
                boa_engine::JsValue::from(this.0.prototype().unwrap()).display_obj(true)
            );
            eprintln!(
                "this.proto.is_callable: {}",
                this.0.prototype().unwrap().is_callable()
            );
            this.0
                .prototype()
                .unwrap()
                .call(&this.0.clone().into(), &[], &mut Context::default());

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
    impl boa_engine::class::Class for X {
        const NAME: &'static str = "X";
        const LENGTH: usize = 1usize;
        fn data_constructor(
            this: &boa_engine::JsValue,
            args: &[boa_engine::JsValue],
            context: &mut boa_engine::Context,
        ) -> boa_engine::JsResult<Self> {
            let rest = args;
            let (boa_arg_0, rest): (JsThis<JsObject>, &[boa_engine::JsValue]) =
                boa_engine::interop::TryFromJsArgument::try_from_js_argument(this, rest, context)?;
            let result = Self::new(boa_arg_0);
            Ok(result)
        }
        fn init(builder: &mut boa_engine::class::ClassBuilder) -> boa_engine::JsResult<()> {
            {
                let proto = builder
                    .context()
                    .eval(Source::from_bytes("Base"))
                    .map_err(|_| boa_engine::js_error!(TypeError : "invalid extends prototype" ))?;
                builder.inherit(proto.as_object().ok_or_else(
                    || boa_engine::js_error!(TypeError : "invalid extends prototype" ),
                )?);
            }
            builder.method(
                boa_engine::js_string!("foo"),
                1usize,
                boa_engine::NativeFunction::from_fn_ptr(
                    |this: &boa_engine::JsValue,
                     args: &[boa_engine::JsValue],
                     context: &mut boa_engine::Context|
                     -> boa_engine::JsResult<boa_engine::JsValue> {
                        let rest = args;
                        let (boa_arg_0, rest): (JsThis<JsObject>, &[boa_engine::JsValue]) =
                            boa_engine::interop::TryFromJsArgument::try_from_js_argument(
                                this, rest, context,
                            )?;
                        let result = Self::foo(boa_arg_0, context);
                        boa_engine::TryIntoJsResult::try_into_js_result(result, context)
                    },
                ),
            );
            Ok(())
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
