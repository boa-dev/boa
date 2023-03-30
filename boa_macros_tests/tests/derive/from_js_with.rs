use boa_engine::{value::TryFromJs, Context, JsNativeError, JsResult, JsValue};

#[derive(TryFromJs)]
struct TestStruct {
    inner: bool,
    #[boa(from_js_with = "lossy_float")]
    my_int: i16,
}

fn main() {}

fn lossy_float(value: &JsValue, _context: &mut Context) -> JsResult<i16> {
    match value {
        JsValue::Rational(r) => Ok(r.round() as i16),
        JsValue::Integer(i) => Ok(*i as i16),
        _ => Err(JsNativeError::typ()
            .with_message("cannot convert value to an i16")
            .into()),
    }
}
