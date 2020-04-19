use crate::builtins::{console::formatter, value::ValueData};
use gc::Gc;

#[test]
fn formatter_no_args_is_empty_string() {
    assert_eq!(formatter(&[]), "")
}

#[test]
fn formatter_empty_format_string_is_empty_string() {
    let val = Gc::new(ValueData::String("".to_string()));
    let res = formatter(&[val]);
    assert_eq!(res, "");
}

#[test]
fn formatter_format_without_args_renders_verbatim() {
    let val = [Gc::new(ValueData::String("%d %s %% %f".to_string()))];
    let res = formatter(&val);
    assert_eq!(res, "%d %s %% %f");
}

#[test]
fn formatter_empty_format_string_concatenates_rest_of_args() {
    let val = [
        Gc::new(ValueData::String("".to_string())),
        Gc::new(ValueData::String("to powinno zostać".to_string())),
        Gc::new(ValueData::String("połączone".to_string())),
    ];
    let res = formatter(&val);
    assert_eq!(res, " to powinno zostać połączone");
}

#[test]
fn formatter_utf_8_checks() {
    let val = [
        Gc::new(ValueData::String(
            "Są takie chwile %dą %są tu%sów %привет%ź".to_string(),
        )),
        Gc::new(ValueData::Integer(123)),
        Gc::new(ValueData::Number(1.23)),
        Gc::new(ValueData::String("ł".to_string())),
    ];
    let res = formatter(&val);
    assert_eq!(res, "Są takie chwile 123ą 1.23ą tułów %привет%ź");
}

#[test]
fn formatter_trailing_format_leader_renders() {
    let val = [
        Gc::new(ValueData::String("%%%%%".to_string())),
        Gc::new(ValueData::String("|".to_string())),
    ];
    let res = formatter(&val);
    assert_eq!(res, "%%% |")
}

#[test]
fn formatter_float_format_works() {
    let val = [
        Gc::new(ValueData::String("%f".to_string())),
        Gc::new(ValueData::Number(3.1415)),
    ];
    let res = formatter(&val);
    assert_eq!(res, "3.141500")
}
