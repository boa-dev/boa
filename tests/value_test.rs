extern crate boa;
use boa::js::value::*;

#[test]
fn check_is_object() {
    let val = Value::new_obj(None);
    assert_eq!(val.is_object(), true);
}

#[test]
fn check_string_to_value() {
    let s = String::from("Hello");
    let v = s.to_value();
    assert_eq!(v.is_string(), true);
    assert_eq!(v.is_null(), false);
}

#[test]
fn check_get_set_field() {
    let obj = Value::new_obj(None);
    // Create string and convert it to a Value
    let s = String::from("bar").to_value();
    obj.set_field_slice("foo", s);
    assert_eq!(obj.get_field_slice("foo").to_string(), "bar");
}
