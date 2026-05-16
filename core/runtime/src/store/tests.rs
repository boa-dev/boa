use super::JsValueStore;
use boa_engine::object::builtins::{JsArrayBuffer, JsDataView};
use boa_engine::value::TryIntoJs;
use boa_engine::Context;

#[test]
fn dataview_clone_non_zero_offset() {
    let mut context = Context::default();

    // Create an ArrayBuffer of 32 bytes
    let array_buffer = JsArrayBuffer::new(32, &mut context).unwrap();

    // Create DataView covering 16 bytes starting at offset 8
    let data_view = JsDataView::from_js_array_buffer(
        array_buffer.clone(),
        Some(8),
        Some(16),
        &mut context,
    )
    .unwrap();

    // Store
    let store = JsValueStore::try_from_js(&data_view.into(), &mut context, vec![]).unwrap();

    // Restore
    let restored = store.try_into_js(&mut context).unwrap();

    // Assertions
    let restored_dv = JsDataView::from_object(restored.as_object().unwrap().clone()).unwrap();

    assert_eq!(restored_dv.byte_offset(&mut context).unwrap(), 8);
    assert_eq!(restored_dv.byte_length(&mut context).unwrap(), 16);

    let restored_buffer_val = restored_dv.buffer(&mut context).unwrap();
    let restored_buffer =
        JsArrayBuffer::from_object(restored_buffer_val.as_object().unwrap().clone()).unwrap();

    // The underlying buffer must be correctly cloned and sized to the original buffer size (32)
    let buffer_len = restored_buffer.borrow().data().bytes().unwrap().len();
    assert_eq!(buffer_len, 32);

    // Also, it should NOT be the exact same buffer instance
    assert_ne!(
        std::ptr::from_ref(array_buffer.as_object().as_ref()).addr(),
        std::ptr::from_ref(restored_buffer.as_object().as_ref()).addr()
    );
}

#[test]
fn dataview_transfer() {
    let mut context = Context::default();

    // Create an ArrayBuffer of 32 bytes
    let array_buffer = JsArrayBuffer::new(32, &mut context).unwrap();

    // Create DataView
    let data_view = JsDataView::from_js_array_buffer(
        array_buffer.clone(),
        Some(4),
        Some(8),
        &mut context,
    )
    .unwrap();

    // Store, but transfer the ArrayBuffer
    let store = JsValueStore::try_from_js(
        &data_view.into(),
        &mut context,
        vec![array_buffer.clone().into()],
    )
    .unwrap();

    // Original buffer should be detached
    assert!(array_buffer.borrow().data().bytes().is_none());

    // Restore
    let restored = store.try_into_js(&mut context).unwrap();

    let restored_dv = JsDataView::from_object(restored.as_object().unwrap().clone()).unwrap();
    assert_eq!(restored_dv.byte_offset(&mut context).unwrap(), 4);
    assert_eq!(restored_dv.byte_length(&mut context).unwrap(), 8);

    let restored_buffer_val = restored_dv.buffer(&mut context).unwrap();
    let restored_buffer =
        JsArrayBuffer::from_object(restored_buffer_val.as_object().unwrap().clone()).unwrap();
    let buffer_len = restored_buffer.borrow().data().bytes().unwrap().len();
    assert_eq!(buffer_len, 32);
}
