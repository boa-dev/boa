use crate::{builtins::console::formatter, Context, JsValue};

#[test]
fn formatter_no_args_is_empty_string() {
    let mut context = Context::default();
    assert_eq!(formatter(&[], &mut context).unwrap(), "");
}

#[test]
fn formatter_empty_format_string_is_empty_string() {
    let mut context = Context::default();
    let val = JsValue::new("");
    assert_eq!(formatter(&[val], &mut context).unwrap(), "");
}

#[test]
fn formatter_format_without_args_renders_verbatim() {
    let mut context = Context::default();
    let val = [JsValue::new("%d %s %% %f")];
    let res = formatter(&val, &mut context).unwrap();
    assert_eq!(res, "%d %s %% %f");
}

#[test]
fn formatter_empty_format_string_concatenates_rest_of_args() {
    let mut context = Context::default();

    let val = [
        JsValue::new(""),
        JsValue::new("to powinno zostać"),
        JsValue::new("połączone"),
    ];
    let res = formatter(&val, &mut context).unwrap();
    assert_eq!(res, " to powinno zostać połączone");
}

#[test]
fn formatter_utf_8_checks() {
    let mut context = Context::default();

    let val = [
        JsValue::new("Są takie chwile %dą %są tu%sów %привет%ź".to_string()),
        JsValue::new(123),
        JsValue::new(1.23),
        JsValue::new("ł"),
    ];
    let res = formatter(&val, &mut context).unwrap();
    assert_eq!(res, "Są takie chwile 123ą 1.23ą tułów %привет%ź");
}

#[test]
fn formatter_trailing_format_leader_renders() {
    let mut context = Context::default();

    let val = [JsValue::new("%%%%%"), JsValue::new("|")];
    let res = formatter(&val, &mut context).unwrap();
    assert_eq!(res, "%%% |");
}

#[test]
#[allow(clippy::approx_constant)]
fn formatter_float_format_works() {
    let mut context = Context::default();

    let val = [JsValue::new("%f"), JsValue::new(3.1415)];
    let res = formatter(&val, &mut context).unwrap();
    assert_eq!(res, "3.141500");
}
