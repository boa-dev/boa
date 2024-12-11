use crate::Context;

#[test]
fn create_byte_data_block() {
    let context = &mut Context::default();
    // Sunny day
    assert!(super::create_byte_data_block(100, None, context).is_ok());

    // Rainy day
    assert!(super::create_byte_data_block(usize::MAX, None, context).is_err());
}

#[test]
fn create_shared_byte_data_block() {
    let context = &mut Context::default();
    // Sunny day
    assert!(super::shared::create_shared_byte_data_block(100, context).is_ok());

    // Rainy day
    assert!(super::shared::create_shared_byte_data_block(usize::MAX, context).is_err());
}
