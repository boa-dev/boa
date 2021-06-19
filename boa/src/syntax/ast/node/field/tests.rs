#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        a.field_name;
        a[5];
        a["other_field_name"];
        "#,
    );
}
