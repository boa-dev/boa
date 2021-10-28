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

#[test]
fn is_view() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "ArrayBuffer.isView(new Int32Array())"), "true");
}

#[test]
fn slice() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(16);
        const int32View = new Int32Array(buffer);

        int32View[1] = 42;
        const sliced = new Int32Array(buffer.slice(4, 12));
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "sliced[0]"), "42");
}
