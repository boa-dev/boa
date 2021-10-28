use super::ArrayBuffer;
use crate::{forward, Context, JsValue};

#[test]
fn constructor() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(8);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "buffer.byteLength"), "8");
}
