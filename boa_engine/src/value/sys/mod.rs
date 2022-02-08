//! This module contains various implementations of the [`JsValue`] type, and
//! type redefinitions to select the requested [`JsValue`] implementation at
//! compile time, using features.

use crate::{object::JsObject, JsBigInt, JsString, JsSymbol};
use std::mem::ManuallyDrop;

// Minimum required definition for a correct `JsValue` implementation:

// pub const fn null() -> Self;
// pub const fn undefined() -> Self;
// pub fn boolean(bool) -> Self;
// pub fn integer(i32) -> Self;
// pub fn rational(f64) -> Self;
// pub const fn nan() -> Self;
// pub fn string(JsString) -> Self;
// pub fn bigint(JsBigInt) -> Self;
// pub fn symbol(JsSymbol) -> Self;
// pub fn object(JsObject) -> Self;

// pub fn as_boolean(&self) -> Option<bool>;
// pub fn as_integer(&self) -> Option<i32>;
// pub fn as_rational(&self) -> Option<f64>;
// pub fn as_string(&self) -> Option<Ref<'_, JsString>>;
// pub fn as_bigint(&self) -> Option<Ref<'_, JsBigInt>>;
// pub fn as_symbol(&self) -> Option<Ref<'_, JsSymbol>>;
// pub fn as_object(&self) -> Option<Ref<'_, JsObject>>;

// pub fn is_null(&self) -> bool;
// pub fn is_undefined(&self) -> bool;
// pub fn is_boolean(&self) -> bool;
// pub fn is_integer(&self) -> bool;
// pub fn is_rational(&self) -> bool;
// pub fn is_nan(&self) -> bool;
// pub fn is_string(&self) -> bool;
// pub fn is_bigint(&self) -> bool;
// pub fn is_symbol(&self) -> bool;
// pub fn is_object(&self) -> bool;

// pub fn variant(&self) -> JsVariant<'_>;

// Ref<'a, T> type

cfg_if::cfg_if! {
    if #[cfg(all(feature = "nan_boxing", not(doc)))] {
        cfg_if::cfg_if! {
            if #[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))] {

                #[path = "nan_boxed.rs"]
                mod r#impl;

            } else {
                compile_error!("This platform doesn't support NaN-boxing.");
            }
        }
    } else {
        #[path = "default.rs"]
        mod r#impl;
    }
}

pub use r#impl::*;

/// Return value of the [`JsValue::variant`] method.
///
/// Represents either a primitive value ([`bool`], [`f64`], [`i32`]) or a reference
/// to a heap allocated value ([`JsString`], [`JsSymbol`]).
///
/// References to heap allocated values are represented by [`Ref`], since
/// more exotic implementations of [`JsValue`] such as nan-boxed ones cannot
/// effectively return references.
#[derive(Debug)]
pub enum JsVariant<'a> {
    Null,
    Undefined,
    Boolean(bool),
    Integer32(i32),
    Float64(f64),
    String(Ref<'a, JsString>),
    BigInt(Ref<'a, JsBigInt>),
    Symbol(Ref<'a, JsSymbol>),
    Object(Ref<'a, JsObject>),
}

/// This abstracts over every pointer type and the required conversions
/// for some of the [`JsValue`] implementations.
///
/// # Safety
///
/// Non-exhaustive list of situations that could cause undefined behaviour:
/// - Returning an invalid `*mut ()`.
/// - Returning a `ManuallyDrop<Self>` that doesn't correspond with the provided
/// `ptr`.
/// - Dropping `ty` before returning its pointer.
pub(crate) unsafe trait PointerType {
    unsafe fn from_void_ptr(ptr: *mut ()) -> ManuallyDrop<Self>;

    unsafe fn into_void_ptr(ty: ManuallyDrop<Self>) -> *mut ();
}
