use super::*;

#[test]
fn ut_check_failed_allocation() {
    let mut context = Context::new();

    assert!(create_byte_data_block(usize::MAX, &mut context).is_err())
}
