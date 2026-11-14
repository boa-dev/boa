use super::*;
use boa_engine::context::HostHooks;
use boa_engine::object::builtins::JsArrayBuffer;
use std::rc::Rc;

struct LimitedBufferHooks(u64);

impl HostHooks for LimitedBufferHooks {
    fn max_buffer_size(&self, _context: &mut Context) -> u64 {
        self.0
    }
}

#[test]
fn array_buffer_deserialization_respects_max_buffer_size() {
    let hooks = Rc::new(LimitedBufferHooks(16));
    let mut context = Context::builder().host_hooks(hooks).build().unwrap();

    // Create an ArrayBuffer value store with 32 bytes (exceeds 16 byte limit)
    let payload = AlignedVec::from_slice(0, &[0x42; 32]);
    let store = JsValueStore::new(ValueStoreInner::ArrayBuffer(payload));

    let result = store.try_into_js(&mut context);
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("cannot allocate a buffer that exceeds the maximum buffer size"),
        "expected max buffer size RangeError, got: {err_str}"
    );
}

#[test]
fn array_buffer_deserialization_within_max_buffer_size() {
    let hooks = Rc::new(LimitedBufferHooks(16));
    let mut context = Context::builder().host_hooks(hooks).build().unwrap();

    let data = [1, 2, 3, 4, 5, 6, 7, 8];
    let payload = AlignedVec::from_slice(0, &data);
    let store = JsValueStore::new(ValueStoreInner::ArrayBuffer(payload));

    let result = store.try_into_js(&mut context).unwrap();
    let obj = result.as_object().unwrap();
    let ab = JsArrayBuffer::from_object(obj.clone()).unwrap();
    assert_eq!(ab.byte_length(), 8);
    assert_eq!(ab.data().unwrap().as_ref(), &data);
}

#[test]
fn array_buffer_roundtrip_clone() {
    let mut context = Context::default();

    let data = [10, 20, 30, 40, 50];
    let original = JsArrayBuffer::new(data.len(), &mut context).unwrap();
    original.data_mut().unwrap().copy_from_slice(&data);

    let js_val: JsValue = original.into();
    let store = JsValueStore::try_from_js(&js_val, &mut context, vec![]).unwrap();
    let cloned_val = store.try_into_js(&mut context).unwrap();

    let cloned_obj = cloned_val.as_object().unwrap();
    let cloned_ab = JsArrayBuffer::from_object(cloned_obj.clone()).unwrap();

    assert_eq!(cloned_ab.byte_length(), 5);
    assert_eq!(cloned_ab.data().unwrap().as_ref(), &data);
}

#[test]
fn empty_array_buffer_roundtrip() {
    let mut context = Context::default();

    let original = JsArrayBuffer::new(0, &mut context).unwrap();
    let js_val: JsValue = original.into();
    let store = JsValueStore::try_from_js(&js_val, &mut context, vec![]).unwrap();
    let cloned_val = store.try_into_js(&mut context).unwrap();

    let cloned_obj = cloned_val.as_object().unwrap();
    let cloned_ab = JsArrayBuffer::from_object(cloned_obj.clone()).unwrap();

    assert_eq!(cloned_ab.byte_length(), 0);
}
