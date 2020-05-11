use super::*;

#[test]
fn check_is_object() {
    let val = Value::new_object(None);
    assert_eq!(val.is_object(), true);
}

#[test]
fn check_string_to_value() {
    let s = String::from("Hello");
    let v = Value::from(s);
    assert_eq!(v.is_string(), true);
    assert_eq!(v.is_null(), false);
}

#[test]
fn check_undefined() {
    let u = ValueData::Undefined;
    assert_eq!(u.get_type(), "undefined");
    assert_eq!(u.to_string(), "undefined");
}

#[test]
fn check_get_set_field() {
    let obj = Value::new_object(None);
    // Create string and convert it to a Value
    let s = Value::from("bar");
    obj.set_field_slice("foo", s);
    assert_eq!(obj.get_field_slice("foo").to_string(), "bar");
}

#[test]
fn check_integer_is_true() {
    assert_eq!(Value::from(1).is_true(), true);
    assert_eq!(Value::from(0).is_true(), false);
    assert_eq!(Value::from(-1).is_true(), true);
}

#[test]
fn check_number_is_true() {
    assert_eq!(Value::from(1.0).is_true(), true);
    assert_eq!(Value::from(0.1).is_true(), true);
    assert_eq!(Value::from(0.0).is_true(), false);
    assert_eq!(Value::from(-0.0).is_true(), false);
    assert_eq!(Value::from(-1.0).is_true(), true);
    assert_eq!(Value::from(NAN).is_true(), false);
}
