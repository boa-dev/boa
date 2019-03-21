extern crate boa;
use boa::js::string::_create;
use boa::js::object::PROTOTYPE;
use boa::js::value::{ValueData};

#[test]
fn check_string_constructor_is_function() {
    let global = ValueData::new_obj(None);
    let string_constructor = _create(global);
    assert_eq!(string_constructor.is_function(), true);
}