use crate::object::JsArrayBuffer;
use crate::Context;

#[test]
fn create_byte_data_block() {
    let context = &mut Context::default();
    // Sunny day
    assert!(super::create_byte_data_block(100, None, context).is_ok());

    // Rainy day
    assert!(super::create_byte_data_block(u64::MAX, None, context).is_err());
}

#[test]
fn create_shared_byte_data_block() {
    let context = &mut Context::default();
    // Sunny day
    assert!(super::shared::create_shared_byte_data_block(100, context).is_ok());

    // Rainy day
    assert!(super::shared::create_shared_byte_data_block(u64::MAX, context).is_err());
}

#[test]
fn resize() {
    let context = &mut Context::default();
    let data_block = super::create_byte_data_block(100, None, context).unwrap();
    let js_arr = JsArrayBuffer::from_byte_block(data_block, context)
        .unwrap()
        .with_max_byte_length(100);
    let mut arr = js_arr.borrow_mut();

    // Sunny day
    assert_eq!(arr.data_mut().resize(50), Ok(()));

    // Rainy day
    assert!(arr.data_mut().resize(u64::MAX).is_err());
}
