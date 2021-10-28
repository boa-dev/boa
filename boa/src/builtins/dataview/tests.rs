use crate::{forward, Context};

#[test]
fn constructor() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);

        const view1 = new DataView(buffer);
        const view2 = new DataView(buffer, 12, 4);
        view1.setInt8(12, 42);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "view2.getInt8(0)"), "42");
    assert_eq!(forward(&mut context, "view1.getInt8(12)"), "42");
}
