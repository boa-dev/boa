#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        {
            let a = function_call();
            console.log("hello");
        }
        another_statement();
        "#,
    );
    // TODO: Once block labels are implemtned, this should be tested:
    // super::super::test_formatting(
    //     r#"
    //     block_name: {
    //         let a = function_call();
    //         console.log("hello");
    //     }
    //     another_statement();
    //     "#,
    // );
}
