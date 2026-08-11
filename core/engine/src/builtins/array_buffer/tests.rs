use super::AlignedVec;
use crate::object::JsArrayBuffer;
use crate::{TestAction, run_test_actions};

#[test]
fn create_byte_data_block() {
    run_test_actions([TestAction::inspect_context(|context| {
        // Sunny day
        assert!(super::create_byte_data_block(100, None, context).is_ok());

        // Rainy day
        assert!(super::create_byte_data_block(u64::MAX, None, context).is_err());
    })]);
}

#[test]
fn create_shared_byte_data_block() {
    run_test_actions([TestAction::inspect_context(|context| {
        // Sunny day
        assert!(super::shared::create_shared_byte_data_block(100, context).is_ok());

        // Rainy day
        assert!(super::shared::create_shared_byte_data_block(u64::MAX, context).is_err());
    })]);
}

#[test]
fn resize() {
    run_test_actions([TestAction::inspect_context(|context| {
        let data_block = super::create_byte_data_block(100, None, context).unwrap();
        let js_arr = JsArrayBuffer::from_byte_block(data_block, context)
            .unwrap()
            .with_max_byte_length(100);
        let mut arr = js_arr.borrow_mut();

        // Sunny day
        assert_eq!(arr.data_mut().resize(50), Ok(()));

        // Rainy day
        assert!(arr.data_mut().resize(u64::MAX).is_err());
    })]);
}

#[test]
fn get_values() {
    run_test_actions([
        TestAction::run(
            r#"
            var buffer = new ArrayBuffer(12);
            var sample = new DataView(buffer, 0);

            sample.setUint8(0, 127);
            sample.setUint8(1, 255);
            sample.setUint8(2, 255);
            sample.setUint8(3, 255);
            sample.setUint8(4, 128);
            sample.setUint8(5, 0);
            sample.setUint8(6, 0);
            sample.setUint8(7, 0);
            sample.setUint8(8, 1);
            sample.setUint8(9, 0);
            sample.setUint8(10, 0);
            sample.setUint8(11, 0);
        "#,
        ),
        TestAction::assert("sample.getUint32(0, false) == 2147483647"),
        TestAction::assert("sample.getUint32(1, false) == 4294967168"),
        TestAction::assert("sample.getUint32(2, false) == 4294934528"),
        TestAction::assert("sample.getUint32(3, false) == 4286578688"),
        TestAction::assert("sample.getUint32(4, false) == 2147483648"),
        TestAction::assert("sample.getUint32(5, false) == 1"),
        TestAction::assert("sample.getUint32(6, false) == 256"),
        TestAction::assert("sample.getUint32(7, false) == 65536"),
        TestAction::assert("sample.getUint32(8, false) == 16777216"),
        TestAction::assert("sample.getUint32(0, true) == 4294967167"),
        TestAction::assert("sample.getUint32(1, true) == 2164260863"),
        TestAction::assert("sample.getUint32(2, true) == 8454143"),
        TestAction::assert("sample.getUint32(3, true) == 33023"),
        TestAction::assert("sample.getUint32(4, true) == 128"),
        TestAction::assert("sample.getUint32(5, true) == 16777216"),
        TestAction::assert("sample.getUint32(6, true) == 65536"),
        TestAction::assert("sample.getUint32(7, true) == 256"),
        TestAction::assert("sample.getUint32(8, true) == 1"),
    ]);
}

#[test]
fn sort() {
    run_test_actions([
        TestAction::run(
            r#"
            // This cmp function is needed as the harness does not support TypedArray comparison.
            function cmp(a, b) {
                return a.length === b.length && a.every((v, i) => v === b[i]);
            }

            var TypedArrayCtor = [
                Int8Array,
                Uint8Array,
                Int16Array,
                Uint16Array,
                Int32Array,
                Uint32Array,
                Float32Array,
                Float64Array,
            ];

            var descending = TypedArrayCtor.map((ctor) => new ctor([4, 3, 2, 1]).sort());
            var mixed = TypedArrayCtor.map((ctor) => new ctor([3, 4, 1, 2]).sort());
            var repeating = TypedArrayCtor.map((ctor) => new ctor([0, 1, 1, 2, 3, 3, 4]).sort());
        "#,
        ),
        // Descending
        TestAction::assert("cmp(descending[0], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[1], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[2], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[3], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[4], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[5], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[6], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[7], [1, 2, 3, 4])"),
        // Mixed
        TestAction::assert("cmp(mixed[0], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[1], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[2], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[3], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[4], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[5], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[6], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[7], [1, 2, 3, 4])"),
        // Repeating
        TestAction::assert("cmp(repeating[0], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[1], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[2], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[3], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[4], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[5], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[6], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[7], [0, 1, 1, 2, 3, 3, 4])"),
    ]);
}

#[test]
fn sort_negative_zero() {
    run_test_actions([
        TestAction::run(
            r#"
            // This cmp function is needed as the harness does not support TypedArray comparison.
            function cmp(a, b) {
                return a.length === b.length && a.every((v, i) => v === b[i]);
            }

            var TypedArrayCtor = [Float32Array, Float64Array];
            var negativeZero = TypedArrayCtor.map((ctor) => new ctor([1, 0, -0, 2]).sort());
            var infinities = TypedArrayCtor.map((ctor) => new ctor([3, 4, Infinity, -Infinity, 1, 2]).sort());
        "#,
        ),
        TestAction::assert("cmp(negativeZero[0], [-0, 0, 1, 2])"),
        TestAction::assert("cmp(negativeZero[1], [-0, 0, 1, 2])"),
        TestAction::assert("cmp(infinities[0], [-Infinity, 1, 2, 3, 4, Infinity])"),
        TestAction::assert("cmp(infinities[1], [-Infinity, 1, 2, 3, 4, Infinity])"),
    ]);
}

/// Tests `SharedArrayBuffer.prototype.slice` which triggers `copy_shared_to_shared`
/// (the `batched_atomic_copy_forward` path).
#[test]
fn shared_array_buffer_slice() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(16);
            var view = new Uint8Array(sab);
            for (var i = 0; i < 16; i++) view[i] = i + 1;
            var sliced = sab.slice(0);
            var result = new Uint8Array(sliced);
        "#,
        ),
        // Verify all 16 bytes copied correctly (exercises u64 batch + head/tail)
        TestAction::assert("result[0] === 1"),
        TestAction::assert("result[7] === 8"),
        TestAction::assert("result[15] === 16"),
        TestAction::assert("result.length === 16"),
    ]);
}

/// Tests `SharedArrayBuffer.prototype.slice` with a partial range and odd sizes
/// to exercise alignment edge cases in the batched copy.
#[test]
fn shared_array_buffer_slice_partial() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(20);
            var view = new Uint8Array(sab);
            for (var i = 0; i < 20; i++) view[i] = i * 3;

            // Slice with odd offset and size to hit unaligned head/tail
            var sliced = sab.slice(3, 14);
            var result = new Uint8Array(sliced);
        "#,
        ),
        TestAction::assert("result.length === 11"),
        TestAction::assert("result[0] === 9"),
        TestAction::assert("result[10] === 39"),
    ]);
}

/// Tests TypedArray.set from a SharedArrayBuffer-backed array to a regular
/// ArrayBuffer-backed array, triggering `batched_copy_atomic_to_bytes`.
#[test]
fn shared_to_regular_typed_array_set() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(16);
            var src = new Uint8Array(sab);
            for (var i = 0; i < 16; i++) src[i] = 100 + i;

            var ab = new ArrayBuffer(16);
            var dest = new Uint8Array(ab);
            dest.set(src);
        "#,
        ),
        TestAction::assert("dest[0] === 100"),
        TestAction::assert("dest[7] === 107"),
        TestAction::assert("dest[15] === 115"),
    ]);
}

/// Tests TypedArray.set from a regular ArrayBuffer-backed array to a
/// SharedArrayBuffer-backed array, triggering `batched_copy_bytes_to_atomic`.
#[test]
fn regular_to_shared_typed_array_set() {
    run_test_actions([
        TestAction::run(
            r#"
            var ab = new ArrayBuffer(16);
            var src = new Uint8Array(ab);
            for (var i = 0; i < 16; i++) src[i] = 200 + i;

            var sab = new SharedArrayBuffer(16);
            var dest = new Uint8Array(sab);
            dest.set(src);
        "#,
        ),
        TestAction::assert("dest[0] === 200"),
        TestAction::assert("dest[7] === 207"),
        TestAction::assert("dest[15] === 215"),
    ]);
}

/// Tests forward `copyWithin` on a SharedArrayBuffer-backed typed array,
/// triggering `copy_shared_to_shared` via `memmove`.
#[test]
fn shared_typed_array_copy_within() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(16);
            var arr = new Uint8Array(sab);
            for (var i = 0; i < 16; i++) arr[i] = i + 1;

            // Forward copy: copies bytes 4..12 to offset 0
            arr.copyWithin(0, 4, 12);
        "#,
        ),
        TestAction::assert("arr[0] === 5"),
        TestAction::assert("arr[7] === 12"),
        TestAction::assert("arr[8] === 9"),
    ]);
}

/// Tests backward `copyWithin` on a SharedArrayBuffer-backed typed array,
/// triggering `copy_shared_to_shared_backwards` when source and dest overlap.
#[test]
fn shared_typed_array_copy_within_backward() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(16);
            var arr = new Uint8Array(sab);
            for (var i = 0; i < 16; i++) arr[i] = i + 1;

            // Backward copy: copies bytes 0..8 to offset 4 (overlapping)
            arr.copyWithin(4, 0, 8);
        "#,
        ),
        TestAction::assert("arr[0] === 1"),
        TestAction::assert("arr[3] === 4"),
        TestAction::assert("arr[4] === 1"),
        TestAction::assert("arr[11] === 8"),
        TestAction::assert("arr[12] === 13"),
    ]);
}

/// Tests zero-length slice to exercise the `count == 0` early return.
#[test]
fn shared_array_buffer_slice_empty() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(16);
            var view = new Uint8Array(sab);
            for (var i = 0; i < 16; i++) view[i] = i + 1;
            var sliced = sab.slice(5, 5);
            var result = new Uint8Array(sliced);
        "#,
        ),
        TestAction::assert("result.length === 0"),
    ]);
}

/// Tests that an externally-backed `ArrayBuffer` aliases the embedder's memory
/// with zero copies: JS writes are visible to the embedder and vice versa.
#[test]
fn external_array_buffer_zero_copy() {
    use crate::{Context, JsValue, Source, js_string, property::Attribute};

    // `AlignedVec` allocations satisfy the 8-byte alignment required for external regions.
    let mut backing: AlignedVec<u8> = AlignedVec::from_iter(0, [0u8; 8]);
    let context = &mut Context::default();

    // SAFETY: `backing` stays alive and unmoved while `context` can reach the buffer.
    let buffer =
        unsafe { JsArrayBuffer::from_external_ptr(backing.as_mut_ptr(), backing.len(), context) };

    assert!(buffer.is_external());
    assert_eq!(buffer.byte_length(), 8);

    context
        .register_global_property(js_string!("buf"), buffer.clone(), Attribute::all())
        .unwrap();

    // A write from the JS side must be visible through the embedder's memory.
    context
        .eval(Source::from_bytes("new Uint8Array(buf)[1] = 42;"))
        .unwrap();
    assert_eq!(backing[1], 42);

    // A write from the embedder's side must be visible to JS.
    backing[2] = 7;
    let value = context
        .eval(Source::from_bytes("new Uint8Array(buf)[2]"))
        .unwrap();
    assert_eq!(value, JsValue::from(7));

    // Direct slice access is not available for external buffers, but copying out is.
    assert!(buffer.data().is_none());
    assert_eq!(buffer.to_vec().as_deref().map(|v| v[1]), Some(42));
}

/// Tests the safe `from_external_data` constructor over a static region of atomics,
/// including an 8-byte-wide view to exercise the aligned access paths.
#[test]
fn external_array_buffer_from_data() {
    use crate::{Context, JsValue, Source, js_string, property::Attribute};
    use portable_atomic::AtomicU8;
    use std::sync::atomic::Ordering;

    #[repr(align(8))]
    struct Backing([AtomicU8; 16]);
    static BACKING: Backing = Backing([const { AtomicU8::new(0) }; 16]);

    let context = &mut Context::default();
    let buffer = JsArrayBuffer::from_external_data(&BACKING.0, context);

    assert!(buffer.is_external());
    assert_eq!(buffer.byte_length(), 16);

    context
        .register_global_property(js_string!("buf"), buffer, Attribute::all())
        .unwrap();

    context
        .eval(Source::from_bytes(
            "new Float64Array(buf)[1] = 1.5; new Uint8Array(buf)[0] = 3;",
        ))
        .unwrap();

    assert_eq!(BACKING.0[0].load(Ordering::Relaxed), 3);
    let mut float_bytes = [0u8; 8];
    for (i, b) in float_bytes.iter_mut().enumerate() {
        *b = BACKING.0[8 + i].load(Ordering::Relaxed);
    }
    assert_eq!(f64::from_ne_bytes(float_bytes).to_bits(), 1.5f64.to_bits());

    let value = context
        .eval(Source::from_bytes("new Float64Array(buf)[1]"))
        .unwrap();
    assert_eq!(value, JsValue::from(1.5));
}

/// Tests that detaching an externally-backed `ArrayBuffer` releases the engine's
/// reference into the region and returns a copy of its contents, while resizing
/// and transferring still fail.
#[test]
fn external_array_buffer_detach_releases_region() {
    use crate::{Context, JsValue};

    let mut backing: AlignedVec<u8> = AlignedVec::from_iter(0, [1u8, 2, 3, 4, 5, 6, 7, 8]);
    let context = &mut Context::default();

    // SAFETY: `backing` stays alive and unmoved while `context` can reach the buffer.
    let buffer =
        unsafe { JsArrayBuffer::from_external_ptr(backing.as_mut_ptr(), backing.len(), context) };

    // Resizing an externally-backed buffer must fail.
    assert!(buffer.borrow_mut().data_mut().resize(4).is_err());
    assert_eq!(buffer.byte_length(), 8);

    // Detaching must succeed, returning a copy of the region's contents and dropping
    // the engine's reference into the region.
    let contents = buffer.detach(&JsValue::undefined()).unwrap();
    assert_eq!(contents.as_slice(), &[1u8, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(buffer.byte_length(), 0);
    assert!(buffer.to_vec().is_none());

    // The embedder still owns the region, which is untouched by the detach.
    assert_eq!(&backing[..], &[1u8, 2, 3, 4, 5, 6, 7, 8]);
}

/// Tests that `ArrayBuffer.prototype.slice` copies data out of an externally-backed
/// buffer into a new, Boa-owned buffer.
#[test]
fn external_array_buffer_slice() {
    use crate::{Context, JsValue, Source, js_string, property::Attribute};

    let mut backing: AlignedVec<u8> = AlignedVec::from_iter(0, [10u8, 11, 12, 13, 14, 15, 16, 17]);
    let context = &mut Context::default();

    // SAFETY: `backing` stays alive and unmoved while `context` can reach the buffer.
    let buffer =
        unsafe { JsArrayBuffer::from_external_ptr(backing.as_mut_ptr(), backing.len(), context) };

    context
        .register_global_property(js_string!("buf"), buffer, Attribute::all())
        .unwrap();

    let value = context
        .eval(Source::from_bytes(
            "var sliced = new Uint8Array(buf.slice(2, 6)); sliced[0] + sliced[3]",
        ))
        .unwrap();
    assert_eq!(value, JsValue::from(12 + 15));

    // The slice is an independent, Boa-owned buffer: writes to it must not be
    // visible through the external region.
    context.eval(Source::from_bytes("sliced[0] = 99;")).unwrap();
    assert_eq!(backing[2], 12);
}

/// Tests that constructing an external buffer over a misaligned region panics.
#[test]
#[should_panic(expected = "external buffer memory must be aligned")]
fn external_array_buffer_misaligned_panics() {
    use crate::Context;

    let mut backing: AlignedVec<u8> = AlignedVec::from_iter(0, [0u8; 8]);
    let context = &mut Context::default();

    // SAFETY: the pointer is valid for `len - 1` bytes; the constructor must panic
    // before the buffer is ever used because the base address is misaligned.
    let _buffer = unsafe {
        JsArrayBuffer::from_external_ptr(backing.as_mut_ptr().add(1), backing.len() - 1, context)
    };
}

/// Tests that an externally-backed `SharedArrayBuffer` aliases the embedder's
/// memory with zero copies.
#[test]
fn external_shared_array_buffer_zero_copy() {
    use crate::{
        Context, JsValue, Source, js_string, object::builtins::JsSharedArrayBuffer,
        property::Attribute,
    };

    let mut backing: AlignedVec<u8> = AlignedVec::from_iter(0, [0u8; 8]);
    let context = &mut Context::default();

    // SAFETY: `backing` stays alive and unmoved while `context` can reach the buffer.
    let buffer = unsafe {
        JsSharedArrayBuffer::from_external_ptr(backing.as_mut_ptr(), backing.len(), context)
    };

    assert!(buffer.is_external());
    assert_eq!(buffer.byte_length(), 8);

    context
        .register_global_property(js_string!("sab"), buffer, Attribute::all())
        .unwrap();

    // A write from the JS side must be visible through the embedder's memory.
    context
        .eval(Source::from_bytes("new Uint8Array(sab)[0] = 99;"))
        .unwrap();
    assert_eq!(backing[0], 99);

    // A write from the embedder's side must be visible to JS.
    backing[3] = 123;
    let value = context
        .eval(Source::from_bytes("new Uint8Array(sab)[3]"))
        .unwrap();
    assert_eq!(value, JsValue::from(123));

    // Externally-backed shared buffers are fixed-length.
    let growable = context.eval(Source::from_bytes("sab.growable")).unwrap();
    assert_eq!(growable, JsValue::from(false));
}

/// Tests the safe `SharedArrayBuffer::from_external_data` constructor, including
/// `Atomics` operations over the external region.
#[test]
fn external_shared_array_buffer_from_data() {
    use crate::{
        Context, JsValue, Source, js_string, object::builtins::JsSharedArrayBuffer,
        property::Attribute,
    };
    use portable_atomic::AtomicU8;
    use std::sync::atomic::Ordering;

    #[repr(align(8))]
    struct Backing([AtomicU8; 8]);
    static BACKING: Backing = Backing([const { AtomicU8::new(0) }; 8]);

    let context = &mut Context::default();
    let buffer = JsSharedArrayBuffer::from_external_data(&BACKING.0, context);

    assert!(buffer.is_external());

    context
        .register_global_property(js_string!("sab"), buffer, Attribute::all())
        .unwrap();

    let value = context
        .eval(Source::from_bytes(
            "var ta = new Int32Array(sab); Atomics.add(ta, 0, 7); Atomics.load(ta, 0)",
        ))
        .unwrap();
    assert_eq!(value, JsValue::from(7));
    assert_eq!(BACKING.0[0].load(Ordering::Relaxed), 7);
}
