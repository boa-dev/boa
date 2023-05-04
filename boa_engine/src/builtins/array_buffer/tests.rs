#[test]
fn create_byte_data_block() {
    // Sunny day
    assert!(super::create_byte_data_block(100).is_ok());

    // Rainy day
    assert!(super::create_byte_data_block(u64::MAX).is_err());
}
