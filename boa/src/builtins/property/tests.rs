use super::*;

#[test]
fn is_property_key_test() {
    let v = Value::string("Boop");
    assert!(Property::is_property_key(&v));

    let v = Value::boolean(true);
    assert!(!Property::is_property_key(&v));
}
