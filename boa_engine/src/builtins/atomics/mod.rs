//! Boa's implementation of ECMAScript's global `Atomics` object.
//!
//! The `Atomics` object contains synchronization methods to orchestrate multithreading
//! on contexts that live in separate threads.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-atomics-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics

mod futex;

use std::sync::atomic::Ordering;

use crate::{
    builtins::BuiltInObject,
    context::intrinsics::Intrinsics,
    object::{JsObject, Object},
    property::Attribute,
    realm::Realm,
    string::common::StaticJsStrings,
    symbol::JsSymbol,
    sys::time::Duration,
    value::IntegerOrInfinity,
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
};
use boa_gc::GcRef;
use boa_profiler::Profiler;

use super::{
    array_buffer::BufferRef,
    typed_array::{
        Atomic, ContentType, Element, IntegerIndexed, TypedArrayElement, TypedArrayKind,
    },
    BuiltInBuilder, IntrinsicObject,
};

/// Javascript `Atomics` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Atomics;

impl IntrinsicObject for Atomics {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_method(Atomics::add, "add", 3)
            .static_method(Atomics::bit_and, "and", 3)
            .static_method(Atomics::compare_exchange, "compareExchange", 4)
            .static_method(Atomics::swap, "exchange", 3)
            .static_method(Atomics::is_lock_free, "isLockFree", 1)
            .static_method(Atomics::load, "load", 2)
            .static_method(Atomics::bit_or, "or", 3)
            .static_method(Atomics::store, "store", 3)
            .static_method(Atomics::sub, "sub", 3)
            .static_method(Atomics::wait, "wait", 4)
            .static_method(Atomics::notify, "notify", 3)
            .static_method(Atomics::bit_xor, "xor", 3)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().atomics()
    }
}

impl BuiltInObject for Atomics {
    const NAME: JsString = StaticJsStrings::ATOMICS;
}

macro_rules! atomic_op {
    ($(#[$attr:meta])* $name:ident) => {
        $(#[$attr])* fn $name(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {

            let array = args.get_or_undefined(0);
            let index = args.get_or_undefined(1);
            let value = args.get_or_undefined(2);

            let ii = validate_integer_typed_array(array, false)?;
            let pos = validate_atomic_access(&ii, index, context)?;
            let value = ii.kind().get_element(value, context)?;

            // revalidate
            let mut buffer = ii.viewed_array_buffer().borrow_mut();
            let mut buffer = buffer
                .as_buffer_mut()
                .expect("integer indexed object must contain a valid buffer");
            let Some(mut data) = buffer.data_mut() else {
                return Err(JsNativeError::typ()
                    .with_message("cannot execute atomic operation in detached buffer")
                    .into());
            };
            let data = data.subslice_mut(pos..);

            // SAFETY: The integer indexed object guarantees that the buffer is aligned.
            // The call to `validate_atomic_access` guarantees that the index is in-bounds.
            let value: TypedArrayElement = unsafe {
                match value {
                    TypedArrayElement::Int8(num) => {
                        i8::read_mut(data).$name(num, Ordering::SeqCst).into()
                    }
                    TypedArrayElement::Uint8(num) => {
                        u8::read_mut(data).$name(num, Ordering::SeqCst).into()
                    }
                    TypedArrayElement::Int16(num) => i16::read_mut(data)
                        .$name(num, Ordering::SeqCst)
                        .into(),
                    TypedArrayElement::Uint16(num) => u16::read_mut(data)
                        .$name(num, Ordering::SeqCst)
                        .into(),
                    TypedArrayElement::Int32(num) => i32::read_mut(data)
                        .$name(num, Ordering::SeqCst)
                        .into(),
                    TypedArrayElement::Uint32(num) => u32::read_mut(data)
                        .$name(num, Ordering::SeqCst)
                        .into(),
                    TypedArrayElement::BigInt64(num) => i64::read_mut(data)
                        .$name(num, Ordering::SeqCst)
                        .into(),
                    TypedArrayElement::BigUint64(num) => u64::read_mut(data)
                        .$name(num, Ordering::SeqCst)
                        .into(),
                    TypedArrayElement::Uint8Clamped(_)
                    | TypedArrayElement::Float32(_)
                    | TypedArrayElement::Float64(_) => unreachable!(
                        "must have been filtered out by the call to `validate_integer_typed_array`"
                    ),
                }
            };

            Ok(value.into())
        }
    };
}

impl Atomics {
    /// [`Atomics.isLockFree ( size )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-atomics.islockfree
    fn is_lock_free(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let n be ? ToIntegerOrInfinity(size).
        let n = args.get_or_undefined(0).to_integer_or_infinity(context)?;

        // 2. Let AR be the Agent Record of the surrounding agent.
        Ok(match n.as_integer() {
            // 3. If n = 1, return AR.[[IsLockFree1]].
            Some(1) => <<u8 as Element>::Atomic as Atomic>::is_lock_free(),
            // 4. If n = 2, return AR.[[IsLockFree2]].
            Some(2) => <<u16 as Element>::Atomic as Atomic>::is_lock_free(),
            // 5. If n = 4, return true.
            Some(4) => true,
            // 6. If n = 8, return AR.[[IsLockFree8]].
            Some(8) => <<u64 as Element>::Atomic as Atomic>::is_lock_free(),
            // 7. Return false.
            _ => false,
        }
        .into())
    }

    /// [`Atomics.load ( typedArray, index )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-atomics.load
    fn load(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let array = args.get_or_undefined(0);
        let index = args.get_or_undefined(1);

        // 1. Let indexedPosition be ? ValidateAtomicAccessOnIntegerTypedArray(typedArray, index).
        let ii = validate_integer_typed_array(array, false)?;
        let pos = validate_atomic_access(&ii, index, context)?;

        // 2. Perform ? RevalidateAtomicAccess(typedArray, indexedPosition).
        let buffer = ii.viewed_array_buffer().borrow();
        let buffer = buffer
            .as_buffer()
            .expect("integer indexed object must contain a valid buffer");
        let Some(data) = buffer.data() else {
            return Err(JsNativeError::typ()
                .with_message("cannot execute atomic operation in detached buffer")
                .into());
        };
        let data = data.subslice(pos..);

        // 3. Let buffer be typedArray.[[ViewedArrayBuffer]].
        // 4. Let elementType be TypedArrayElementType(typedArray).
        // 5. Return GetValueFromBuffer(buffer, indexedPosition, elementType, true, seq-cst).
        // SAFETY: The integer indexed object guarantees that the buffer is aligned.
        // The call to `validate_atomic_access` guarantees that the index is in-bounds.
        let value = unsafe { data.get_value(ii.kind(), Ordering::SeqCst) };

        Ok(value.into())
    }

    /// [`Atomics.store ( typedArray, index, value )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-atomics.store
    fn store(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let array = args.get_or_undefined(0);
        let index = args.get_or_undefined(1);
        let value = args.get_or_undefined(2);

        // 1. Let indexedPosition be ? ValidateAtomicAccessOnIntegerTypedArray(typedArray, index).
        let ii = validate_integer_typed_array(array, false)?;
        let pos = validate_atomic_access(&ii, index, context)?;

        // bit of a hack to preserve the converted value
        // 2. If typedArray.[[ContentType]] is bigint, let v be ? ToBigInt(value).
        let converted: JsValue = if ii.kind().content_type() == ContentType::BigInt {
            value.to_bigint(context)?.into()
        } else {
            // 3. Otherwise, let v be 𝔽(? ToIntegerOrInfinity(value)).
            match value.to_integer_or_infinity(context)? {
                IntegerOrInfinity::PositiveInfinity => f64::INFINITY,
                IntegerOrInfinity::Integer(i) => i as f64,
                IntegerOrInfinity::NegativeInfinity => f64::NEG_INFINITY,
            }
            .into()
        };
        let value = ii.kind().get_element(&converted, context)?;

        // 4. Perform ? RevalidateAtomicAccess(typedArray, indexedPosition).
        let mut buffer = ii.viewed_array_buffer().borrow_mut();
        let mut buffer = buffer
            .as_buffer_mut()
            .expect("integer indexed object must contain a valid buffer");
        let Some(mut buffer) = buffer.data_mut() else {
            return Err(JsNativeError::typ()
                .with_message("cannot execute atomic operation in detached buffer")
                .into());
        };
        let mut data = buffer.subslice_mut(pos..);

        // 5. Let buffer be typedArray.[[ViewedArrayBuffer]].
        // 6. Let elementType be TypedArrayElementType(typedArray).
        // 7. Perform SetValueInBuffer(buffer, indexedPosition, elementType, v, true, seq-cst).
        // SAFETY: The integer indexed object guarantees that the buffer is aligned.
        // The call to `validate_atomic_access` guarantees that the index is in-bounds.
        unsafe {
            data.set_value(value, Ordering::SeqCst);
        }

        // 8. Return v.
        Ok(converted)
    }

    /// [`Atomics.compareExchange ( typedArray, index, expectedValue, replacementValue )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-atomics.compareexchange
    fn compare_exchange(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let array = args.get_or_undefined(0);
        let index = args.get_or_undefined(1);
        let expected = args.get_or_undefined(2);
        let replacement = args.get_or_undefined(3);

        // 1. Let indexedPosition be ? ValidateAtomicAccessOnIntegerTypedArray(typedArray, index).
        // 2. Let buffer be typedArray.[[ViewedArrayBuffer]].
        // 3. Let block be buffer.[[ArrayBufferData]].
        let ii = validate_integer_typed_array(array, false)?;
        let pos = validate_atomic_access(&ii, index, context)?;
        let typed_array_kind = ii.kind();

        // 4. If typedArray.[[ContentType]] is bigint, then
        //     a. Let expected be ? ToBigInt(expectedValue).
        //     b. Let replacement be ? ToBigInt(replacementValue).
        // 5. Else,
        //     a. Let expected be 𝔽(? ToIntegerOrInfinity(expectedValue)).
        //     b. Let replacement be 𝔽(? ToIntegerOrInfinity(replacementValue)).
        let exp = typed_array_kind.get_element(expected, context)?.to_bytes();
        let rep = typed_array_kind
            .get_element(replacement, context)?
            .to_bytes();

        // 6. Perform ? RevalidateAtomicAccess(typedArray, indexedPosition).
        let mut buffer = ii.viewed_array_buffer().borrow_mut();
        let mut buffer = buffer
            .as_buffer_mut()
            .expect("integer indexed object must contain a valid buffer");
        let Some(mut data) = buffer.data_mut() else {
            return Err(JsNativeError::typ()
                .with_message("cannot execute atomic operation in detached buffer")
                .into());
        };
        let data = data.subslice_mut(pos..);

        // 7. Let elementType be TypedArrayElementType(typedArray).
        // 8. Let elementSize be TypedArrayElementSize(typedArray).
        // 9. Let isLittleEndian be the value of the [[LittleEndian]] field of the surrounding agent's Agent Record.
        // 10. Let expectedBytes be NumericToRawBytes(elementType, expected, isLittleEndian).
        // 11. Let replacementBytes be NumericToRawBytes(elementType, replacement, isLittleEndian).
        // 12. If IsSharedArrayBuffer(buffer) is true, then
        //     a. Let rawBytesRead be AtomicCompareExchangeInSharedBlock(block, indexedPosition, elementSize, expectedBytes, replacementBytes).
        // 13. Else,
        //     a. Let rawBytesRead be a List of length elementSize whose elements are the sequence of elementSize bytes starting with block[indexedPosition].
        //     b. If ByteListEqual(rawBytesRead, expectedBytes) is true, then
        //         i. Store the individual bytes of replacementBytes into block, starting at block[indexedPosition].
        // 14. Return RawBytesToNumeric(elementType, rawBytesRead, isLittleEndian).
        // SAFETY: The integer indexed object guarantees that the buffer is aligned.
        // The call to `validate_atomic_access` guarantees that the index is in-bounds.
        let value: TypedArrayElement = unsafe {
            match typed_array_kind {
                TypedArrayKind::Int8 => i8::read_mut(data)
                    .compare_exchange(exp as i8, rep as i8, Ordering::SeqCst)
                    .into(),
                TypedArrayKind::Uint8 => u8::read_mut(data)
                    .compare_exchange(exp as u8, rep as u8, Ordering::SeqCst)
                    .into(),
                TypedArrayKind::Int16 => i16::read_mut(data)
                    .compare_exchange(exp as i16, rep as i16, Ordering::SeqCst)
                    .into(),
                TypedArrayKind::Uint16 => u16::read_mut(data)
                    .compare_exchange(exp as u16, rep as u16, Ordering::SeqCst)
                    .into(),
                TypedArrayKind::Int32 => i32::read_mut(data)
                    .compare_exchange(exp as i32, rep as i32, Ordering::SeqCst)
                    .into(),
                TypedArrayKind::Uint32 => u32::read_mut(data)
                    .compare_exchange(exp as u32, rep as u32, Ordering::SeqCst)
                    .into(),
                TypedArrayKind::BigInt64 => i64::read_mut(data)
                    .compare_exchange(exp as i64, rep as i64, Ordering::SeqCst)
                    .into(),
                TypedArrayKind::BigUint64 => u64::read_mut(data)
                    .compare_exchange(exp, rep, Ordering::SeqCst)
                    .into(),
                TypedArrayKind::Uint8Clamped
                | TypedArrayKind::Float32
                | TypedArrayKind::Float64 => unreachable!(
                    "must have been filtered out by the call to `validate_integer_typed_array`"
                ),
            }
        };

        Ok(value.into())
    }

    // =========== Atomics.ops start ===========

    // Most of the operations here follow the same list of steps:
    //
    // AtomicReadModifyWrite ( typedArray, index, value, op )
    // <https://tc39.es/ecma262/#sec-atomicreadmodifywrite>
    //
    // 1. Let buffer be ? ValidateIntegerTypedArray(typedArray).
    // 2. Let indexedPosition be ? ValidateAtomicAccess(typedArray, index).
    // 3. If typedArray.[[ContentType]] is BigInt, let v be ? ToBigInt(value).
    // 4. Otherwise, let v be 𝔽(? ToIntegerOrInfinity(value)).
    // 5. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
    // 6. NOTE: The above check is not redundant with the check in ValidateIntegerTypedArray because the call to ToBigInt or ToIntegerOrInfinity on the preceding lines can have arbitrary side effects, which could cause the buffer to become detached.
    // 7. Let elementType be TypedArrayElementType(typedArray).
    // 8. Return GetModifySetValueInBuffer(buffer, indexedPosition, elementType, v, op).
    //
    // However, our impementation differs significantly from this, which is why these steps are
    // just here for documentation purposes.

    atomic_op! {
        /// [`Atomics.add ( typedArray, index, value )`][spec]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-atomics.add
        add
    }

    atomic_op! {
        /// [`Atomics.and ( typedArray, index, value )`][spec]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-atomics.and
        bit_and
    }

    atomic_op! {
        /// [`Atomics.exchange ( typedArray, index, value )`][spec]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-atomics.exchange
        swap
    }

    atomic_op! {
        /// [`Atomics.or ( typedArray, index, value )`][spec]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-atomics.or
        bit_or
    }

    atomic_op! {
        /// [`Atomics.sub ( typedArray, index, value )`][spec]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-atomics.sub
        sub
    }

    atomic_op! {
        /// [`Atomics.xor ( typedArray, index, value )`][spec]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-atomics.xor
        bit_xor
    }

    /// [`Atomics.wait ( typedArray, index, value, timeout )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-atomics.wait
    fn wait(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let array = args.get_or_undefined(0);
        let index = args.get_or_undefined(1);
        let value = args.get_or_undefined(2);
        let timeout = args.get_or_undefined(3);

        // 1. Let buffer be ? ValidateIntegerTypedArray(typedArray, true).
        let ii = validate_integer_typed_array(array, true)?;
        let buffer = ii.viewed_array_buffer().borrow();
        let buffer = buffer
            .as_buffer()
            .expect("integer indexed object must contain a valid buffer");

        // 2. If IsSharedArrayBuffer(buffer) is false, throw a TypeError exception.
        let BufferRef::SharedBuffer(buffer) = buffer else {
            return Err(JsNativeError::typ()
                .with_message("cannot use `ArrayBuffer` for an atomic wait")
                .into());
        };

        // 3. Let indexedPosition be ? ValidateAtomicAccess(typedArray, index).
        let offset = validate_atomic_access(&ii, index, context)?;

        // spec expects the evaluation of this first, then the timeout.
        let value = if ii.kind() == TypedArrayKind::BigInt64 {
            // 4. If typedArray.[[TypedArrayName]] is "BigInt64Array", let v be ? ToBigInt64(value).
            value.to_big_int64(context)?
        } else {
            // 5. Otherwise, let v be ? ToInt32(value).
            i64::from(value.to_i32(context)?)
        };

        // moving above since we need to make a generic call next.

        // 6. Let q be ? ToNumber(timeout).
        // 7. If q is either NaN or +∞𝔽, let t be +∞; else if q is -∞𝔽, let t be 0; else let t be max(ℝ(q), 0).
        let mut timeout = timeout.to_number(context)?;
        // convert to nanoseconds to discard any excessively big timeouts.
        timeout = timeout.clamp(0.0, f64::INFINITY) * 1000.0 * 1000.0;
        let timeout = if timeout.is_nan() || timeout.is_infinite() || timeout > u64::MAX as f64 {
            None
        } else {
            Some(Duration::from_nanos(timeout as u64))
        };

        // 8. Let B be AgentCanSuspend().
        // 9. If B is false, throw a TypeError exception.
        if !context.can_block() {
            return Err(JsNativeError::typ()
                .with_message("agent cannot be suspended")
                .into());
        }

        // SAFETY: the validity of `addr` is verified by our call to `validate_atomic_access`.
        let result = unsafe {
            if ii.kind() == TypedArrayKind::BigInt64 {
                futex::wait(buffer, offset, value, timeout)?
            } else {
                // value must fit into `i32` since it came from an `i32` above.
                futex::wait(buffer, offset, value as i32, timeout)?
            }
        };

        Ok(match result {
            futex::AtomicsWaitResult::NotEqual => "not-equal",
            futex::AtomicsWaitResult::TimedOut => "timed-out",
            futex::AtomicsWaitResult::Ok => "ok",
        }
        .into())
    }

    /// [`Atomics.notify ( typedArray, index, count )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-atomics.notify
    fn notify(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let array = args.get_or_undefined(0);
        let index = args.get_or_undefined(1);
        let count = args.get_or_undefined(2);

        // 1. Let indexedPosition be ? ValidateAtomicAccessOnIntegerTypedArray(typedArray, index, true).
        let ii = validate_integer_typed_array(array, true)?;
        let offset = validate_atomic_access(&ii, index, context)?;

        // 2. If count is undefined, then
        let count = if count.is_undefined() {
            // a. Let c be +∞.
            u64::MAX
        } else {
            // 3. Else,
            //     a. Let intCount be ? ToIntegerOrInfinity(count).
            //     b. Let c be max(intCount, 0).
            match count.to_integer_or_infinity(context)? {
                IntegerOrInfinity::PositiveInfinity => u64::MAX,
                IntegerOrInfinity::Integer(i) => i64::max(i, 0) as u64,
                IntegerOrInfinity::NegativeInfinity => 0,
            }
        };

        // 4. Let buffer be typedArray.[[ViewedArrayBuffer]].
        // 5. Let block be buffer.[[ArrayBufferData]].
        // 6. If IsSharedArrayBuffer(buffer) is false, return +0𝔽.
        let buffer = ii.viewed_array_buffer();
        let buffer = buffer.borrow();
        let Some(shared) = buffer.as_shared_array_buffer() else {
            return Ok(0.into());
        };

        let count = futex::notify(shared, offset, count)?;

        // 12. Let n be the number of elements in S.
        // 13. Return 𝔽(n).
        Ok(count.into())
    }
}

/// [`ValidateIntegerTypedArray ( typedArray [ , waitable ] )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-validateintegertypedarray
fn validate_integer_typed_array(
    array: &JsValue,
    waitable: bool,
) -> JsResult<GcRef<'_, IntegerIndexed>> {
    // 1. If waitable is not present, set waitable to false.
    // 2. Perform ? ValidateTypedArray(typedArray).
    let ii = array
        .as_object()
        .and_then(|o| GcRef::try_map(o.borrow(), Object::as_typed_array))
        .ok_or_else(|| JsNativeError::typ().with_message("value is not a typed array object"))?;
    if ii.is_detached() {
        return Err(JsNativeError::typ()
            .with_message("Buffer of the typed array is detached")
            .into());
    }

    // 3. Let buffer be typedArray.[[ViewedArrayBuffer]].

    if waitable {
        // 4. If waitable is true, then
        // a. If typedArray.[[TypedArrayName]] is neither "Int32Array" nor "BigInt64Array", throw a TypeError exception.
        if ![TypedArrayKind::Int32, TypedArrayKind::BigInt64].contains(&ii.kind()) {
            return Err(JsNativeError::typ()
                .with_message("can only atomically wait using Int32 or BigInt64 arrays")
                .into());
        }
    } else {
        // 5. Else,
        //     a. Let type be TypedArrayElementType(typedArray).
        //     b. If IsUnclampedIntegerElementType(type) is false and IsBigIntElementType(type) is
        //        false, throw a TypeError exception.
        if !ii.kind().supports_atomic_ops() {
            return Err(JsNativeError::typ()
                .with_message(
                    "platform doesn't support atomic operations on the provided `TypedArray`",
                )
                .into());
        }
    }

    // 6. Return buffer.
    Ok(ii)
}

/// [`ValidateAtomicAccess ( iieoRecord, requestIndex )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-validateatomicaccess
fn validate_atomic_access(
    array: &IntegerIndexed,
    request_index: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<usize> {
    // 1. Let length be typedArray.[[ArrayLength]].
    let length = array.array_length();

    // 2. Let accessIndex be ? ToIndex(requestIndex).
    let access_index = request_index.to_index(context)?;

    // 3. Assert: accessIndex ≥ 0.
    //    ensured by the type.

    // 4. If accessIndex ≥ length, throw a RangeError exception.
    if access_index >= length {
        return Err(JsNativeError::range()
            .with_message("index for typed array outside of bounds")
            .into());
    }

    // 5. Let elementSize be TypedArrayElementSize(typedArray).
    let element_size = array.kind().element_size();

    // 6. Let offset be typedArray.[[ByteOffset]].
    let offset = array.byte_offset();

    // 7. Return (accessIndex × elementSize) + offset.
    Ok(((access_index * element_size) + offset) as usize)
}
