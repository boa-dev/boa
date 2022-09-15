use super::*;

#[test]
fn ut_sunny_day_create_byte_data_block() {
    assert!(create_byte_data_block(100).is_ok());
}

#[test]
fn ut_rainy_day_create_byte_data_block() {
    assert!(create_byte_data_block(u64::MAX).is_err());
}
