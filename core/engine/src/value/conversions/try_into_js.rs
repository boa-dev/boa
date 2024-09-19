use crate::{Context, JsNativeError, JsResult, JsValue};
use boa_string::JsString;

/// This trait adds a conversions from a Rust Type into [`JsValue`].
pub trait TryIntoJs: Sized {
    /// This function tries to convert a `Self` into [`JsValue`].
    fn try_into_js(&self, context: &mut Context) -> JsResult<JsValue>;
}

impl TryIntoJs for bool {
    fn try_into_js(&self, _context: &mut Context) -> JsResult<JsValue> {
        JsResult::Ok(match *self {
            true => JsValue::Boolean(true),
            false => JsValue::Boolean(false),
        })
    }
}

impl TryIntoJs for &str {
    fn try_into_js(&self, _context: &mut Context) -> JsResult<JsValue> {
        JsResult::Ok(JsValue::String(JsString::from(*self)))
    }
}
impl TryIntoJs for String {
    fn try_into_js(&self, _context: &mut Context) -> JsResult<JsValue> {
        JsResult::Ok(JsValue::String(JsString::from(self.as_str())))
    }
}

macro_rules! impl_try_into_js_by_from {
    ($t:ty) => {
        impl TryIntoJs for $t {
            fn try_into_js(&self, _context: &mut Context) -> JsResult<JsValue> {
                JsResult::Ok(JsValue::from(self.clone()))
            }
        }
    };
    [$($ts:ty),+] => {
        $(impl_try_into_js_by_from!($ts);)+
    }
}
impl_try_into_js_by_from![i8, u8, i16, u16, i32, u32, f32, f64];
impl_try_into_js_by_from![
    JsValue,
    JsString,
    crate::JsBigInt,
    crate::JsObject,
    crate::JsSymbol,
    crate::object::JsArray,
    crate::object::JsArrayBuffer,
    crate::object::JsDataView,
    crate::object::JsDate,
    crate::object::JsFunction,
    crate::object::JsGenerator,
    crate::object::JsMapIterator,
    crate::object::JsMap,
    crate::object::JsSetIterator,
    crate::object::JsSet,
    crate::object::JsSharedArrayBuffer,
    crate::object::JsInt8Array,
    crate::object::JsInt16Array,
    crate::object::JsInt32Array,
    crate::object::JsUint8Array,
    crate::object::JsUint16Array,
    crate::object::JsUint32Array,
    crate::object::JsFloat32Array,
    crate::object::JsFloat64Array
];

const MAX_SAFE_INTEGER_I64: i64 = (1 << 53) - 1;
const MIN_SAFE_INTEGER_I64: i64 = -MAX_SAFE_INTEGER_I64;

fn err_outside_safe_range() -> crate::JsError {
    JsNativeError::typ()
        .with_message("cannot convert value into JsValue: the value is outside the safe range")
        .into()
}
fn convert_safe_i64(value: i64) -> JsValue {
    i32::try_from(value).map_or(JsValue::Rational(value as f64), JsValue::Integer)
}

impl TryIntoJs for i64 {
    fn try_into_js(&self, _context: &mut Context) -> JsResult<JsValue> {
        let value = *self;
        if value < MIN_SAFE_INTEGER_I64 || MAX_SAFE_INTEGER_I64 < value {
            JsResult::Err(err_outside_safe_range())
        } else {
            JsResult::Ok(convert_safe_i64(value))
        }
    }
}
impl TryIntoJs for u64 {
    fn try_into_js(&self, _context: &mut Context) -> JsResult<JsValue> {
        let value = *self;
        if (MAX_SAFE_INTEGER_I64 as u64) < value {
            JsResult::Err(err_outside_safe_range())
        } else {
            JsResult::Ok(convert_safe_i64(value as i64))
        }
    }
}
impl TryIntoJs for i128 {
    fn try_into_js(&self, _context: &mut Context) -> JsResult<JsValue> {
        let value = *self;
        if value < (MIN_SAFE_INTEGER_I64 as i128) || (MAX_SAFE_INTEGER_I64 as i128) < value {
            JsResult::Err(err_outside_safe_range())
        } else {
            JsResult::Ok(convert_safe_i64(value as i64))
        }
    }
}
impl TryIntoJs for u128 {
    fn try_into_js(&self, _context: &mut Context) -> JsResult<JsValue> {
        let value = *self;
        if (MAX_SAFE_INTEGER_I64 as u128) < value {
            JsResult::Err(err_outside_safe_range())
        } else {
            JsResult::Ok(convert_safe_i64(value as i64))
        }
    }
}

impl<T> TryIntoJs for Option<T>
where
    T: TryIntoJs,
{
    fn try_into_js(&self, context: &mut Context) -> JsResult<JsValue> {
        match self {
            Some(x) => x.try_into_js(context),
            None => JsResult::Ok(JsValue::Null),
        }
    }
}

impl<T> TryIntoJs for Vec<T>
where
    T: TryIntoJs,
{
    fn try_into_js(&self, context: &mut Context) -> JsResult<JsValue> {
        let arr = crate::object::JsArray::new(context);
        for value in self {
            let value = value.try_into_js(context)?;
            arr.push(value, context)?;
        }
        JsResult::Ok(arr.into())
    }
}

macro_rules! impl_try_into_js_for_tuples {
    ($($names:ident : $ts:ident),+) => {
        impl<$($ts: TryIntoJs,)+> TryIntoJs for ($($ts,)+) {
            fn try_into_js(&self, context: &mut Context) -> JsResult<JsValue> {
                let ($($names,)+) = self;
                let arr = crate::object::JsArray::new(context);
                $(arr.push($names.try_into_js(context)?, context)?;)+
                JsResult::Ok(arr.into())
            }
        }
    };
}

impl_try_into_js_for_tuples!(a: A);
impl_try_into_js_for_tuples!(a: A, b: B);
impl_try_into_js_for_tuples!(a: A, b: B, c: C);
impl_try_into_js_for_tuples!(a: A, b: B, c: C, d: D);
impl_try_into_js_for_tuples!(a: A, b: B, c: C, d: D, e: E);
impl_try_into_js_for_tuples!(a: A, b: B, c: C, d: D, e: E, f: F);
impl_try_into_js_for_tuples!(a: A, b: B, c: C, d: D, e: E, f: F, g: G);
impl_try_into_js_for_tuples!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H);
impl_try_into_js_for_tuples!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I);
impl_try_into_js_for_tuples!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J);
impl_try_into_js_for_tuples!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K);

impl TryIntoJs for () {
    fn try_into_js(&self, _context: &mut Context) -> JsResult<JsValue> {
        JsResult::Ok(JsValue::Null)
    }
}

impl<T, S> TryIntoJs for std::collections::HashSet<T, S>
where
    T: TryIntoJs,
{
    fn try_into_js(&self, context: &mut Context) -> JsResult<JsValue> {
        let set = crate::object::JsSet::new(context);
        for value in self {
            let value = value.try_into_js(context)?;
            set.add(value, context)?;
        }
        JsResult::Ok(set.into())
    }
}

impl<K, V, S> TryIntoJs for std::collections::HashMap<K, V, S>
where
    K: TryIntoJs,
    V: TryIntoJs,
{
    fn try_into_js(&self, context: &mut Context) -> JsResult<JsValue> {
        let map = crate::object::JsMap::new(context);
        for (key, value) in self {
            let key = key.try_into_js(context)?;
            let value = value.try_into_js(context)?;
            map.set(key, value, context)?;
        }
        JsResult::Ok(map.into())
    }
}
