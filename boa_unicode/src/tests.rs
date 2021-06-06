#[test]
fn check_unicode_version() {
    assert_eq!(
        super::UNICODE_VERSION,
        unicode_general_category::UNICODE_VERSION
    );
}
