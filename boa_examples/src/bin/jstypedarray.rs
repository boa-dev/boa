// This example shows how to manipulate a Javascript array using Rust code.

use boa_engine::{
    object::{FunctionBuilder, JsUint8Array},
    property::Attribute,
    Context, JsResult, JsValue,
};

fn main() -> JsResult<()> {
    // We create a new `Context` to create a new Javascript executor.
    let context = &mut Context::default();

    let data: Vec<u8> = (0..=255).collect();

    let array = JsUint8Array::from_iter(data, context)?;

    assert_eq!(array.get(0, context)?, JsValue::new(0));

    let mut sum = 0;

    for i in 0..=255 {
        assert_eq!(array.at(i, context)?, JsValue::new(i));
        sum += i;
    }

    let callback = FunctionBuilder::native(context, |_this, args, context| {
        let accumulator = args.get(0).cloned().unwrap_or_default();
        let value = args.get(1).cloned().unwrap_or_default();

        accumulator.add(&value, context)
    })
    .build();

    assert_eq!(
        array.reduce(callback, Some(JsValue::new(0)), context)?,
        JsValue::new(sum)
    );

    context.register_global_property(
        "myUint8Array",
        array,
        Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
    );

    Ok(())
}
