use super::*;

#[test]
fn ut_sunny_day_create_byte_data_block() {
    let mut context = Context::default();

    assert!(create_byte_data_block(100, &mut context).is_ok());
}

#[test]
fn ut_rainy_day_create_byte_data_block() {
    let mut context = Context::default();

    assert!(create_byte_data_block(u64::MAX, &mut context).is_err());
}
