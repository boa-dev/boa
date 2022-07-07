// This example shows how to manipulate a Javascript array using Rust code.

use boa_engine::{
    object::{JsArrayBuffer, JsUint32Array, JsUint8Array},
    property::Attribute,
    Context, JsResult, JsValue,
};

fn main() -> JsResult<()> {
    // We create a new `Context` to create a new Javascript executor.
    let context = &mut Context::default();

    // This create an array buffer of byte length 4
    let array_buffer = JsArrayBuffer::new(4, context)?;

    // We can now create an typed array to access the data.
    let uint32_typed_array = JsUint32Array::from_array_buffer(array_buffer, context)?;

    let value = 0x12345678u32;
    uint32_typed_array.set(0, value, true, context)?;

    assert_eq!(uint32_typed_array.get(0, context)?, JsValue::new(value));

    // We can also create array buffers from a user defined block of data.
    //
    // NOTE: The block data will not be cloned.
    let blob_of_data: Vec<u8> = (0..=255).collect();
    let array_buffer = JsArrayBuffer::from_byte_block(blob_of_data, context)?;

    // This the byte length of the new array buffer will be the length of block of data.
    let byte_length = array_buffer.byte_length(context);
    assert_eq!(byte_length, 256);

    // We can now create an typed array to access the data.
    let uint8_typed_array = JsUint8Array::from_array_buffer(array_buffer.clone(), context)?;

    for i in 0..byte_length {
        assert_eq!(uint8_typed_array.get(i, context)?, JsValue::new(i));
    }

    // We can also register it as a global property
    context.register_global_property(
        "myArrayBuffer",
        array_buffer,
        Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
    );

    Ok(())
}
