use boa_engine::{js_string, object::builtins::JsRegExp, Context, JsResult};

fn main() -> JsResult<()> {
    let context = &mut Context::default();

    let regexp = JsRegExp::new(js_string!("foo"), js_string!("gi"), context)?;

    let test_result = regexp.test(js_string!("football"), context)?;
    assert!(test_result);

    let flags = regexp.flags(context)?;
    assert_eq!(flags, String::from("gi"));

    let src = regexp.source(context)?;
    assert_eq!(src, String::from("foo"));

    let to_string = regexp.to_string(context)?;
    assert_eq!(to_string, String::from("/foo/gi"));

    Ok(())
}
