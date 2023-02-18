use boa_engine::{value::TryFromJs, Context, JsNativeError, JsResult, JsValue, Runtime, Source};

/// You can easily derive `TryFromJs` for structures with base Rust types.
///
/// By default, the conversion will only work if the type is directly representable by the Rust
/// type.
#[derive(Debug, TryFromJs)]
#[allow(dead_code)]
struct TestStruct {
    inner: bool,
    hello: String,
    // You can override the conversion of an attribute.
    #[boa(from_js_with = "lossy_conversion")]
    my_float: i16,
}

fn main() {
    let js_str = r#"
    let x = {
        inner: false,
        hello: "World",
        my_float: 2.9,
    };
    
    x;
    "#;
    let js = Source::from_bytes(js_str);
    let runtime = &Runtime::default();
    let mut context = Context::builder(runtime).build().unwrap();
    let res = context.eval_script(js).unwrap();

    let str = TestStruct::try_from_js(&res, &mut context)
        .map_err(|e| e.to_string())
        .unwrap();

    println!("{str:?}");
}

/// Converts the value lossly
fn lossy_conversion(value: &JsValue, _context: &mut Context) -> JsResult<i16> {
    match value {
        JsValue::Rational(r) => Ok(r.round() as i16),
        JsValue::Integer(i) => Ok(*i as i16),
        _ => Err(JsNativeError::typ()
            .with_message("cannot convert value to an i16")
            .into()),
    }
}
