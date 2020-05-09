use crate::builtins::{console::formatter, value::Value};

#[test]
fn formatter_no_args_is_empty_string() {
    assert_eq!(formatter(&[]), "")
}

#[test]
fn formatter_empty_format_string_is_empty_string() {
    let val = Value::string("".to_string());
    let res = formatter(&[val]);
    assert_eq!(res, "");
}

#[test]
fn formatter_format_without_args_renders_verbatim() {
    let val = [Value::string("%d %s %% %f".to_string())];
    let res = formatter(&val);
    assert_eq!(res, "%d %s %% %f");
}

#[test]
fn formatter_empty_format_string_concatenates_rest_of_args() {
    let val = [
        Value::string("".to_string()),
        Value::string("to powinno zostać".to_string()),
        Value::string("połączone".to_string()),
    ];
    let res = formatter(&val);
    assert_eq!(res, " to powinno zostać połączone");
}

#[test]
fn formatter_utf_8_checks() {
    let val = [
        Value::string("Są takie chwile %dą %są tu%sów %привет%ź".to_string()),
        Value::integer(123),
        Value::rational(1.23),
        Value::string("ł".to_string()),
    ];
    let res = formatter(&val);
    assert_eq!(res, "Są takie chwile 123ą 1.23ą tułów %привет%ź");
}

#[test]
fn formatter_trailing_format_leader_renders() {
    let val = [
        Value::string("%%%%%".to_string()),
        Value::string("|".to_string()),
    ];
    let res = formatter(&val);
    assert_eq!(res, "%%% |")
}

#[test]
#[allow(clippy::approx_constant)]
fn formatter_float_format_works() {
    let val = [Value::string("%f".to_string()), Value::rational(3.1415)];
    let res = formatter(&val);
    assert_eq!(res, "3.141500")
}
