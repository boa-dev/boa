use crate::syntax::ast::node::test_formatting;

#[test]
fn fmt() {
    // Blocks do not store their label, so we cannot test with
    // the outer block having a label.
    //
    // TODO: Once block labels are implemented, this test should
    // include them:
    //
    // ```
    // outer: {
    //     while (true) {
    //         break outer;
    //     }
    //     skipped_call();
    // }
    // ```
    test_formatting(
        r#"
        {
            while (true) {
                break outer;
            }
            skipped_call();
        }
        while (true) {
            break;
        }
        "#,
    );
}
