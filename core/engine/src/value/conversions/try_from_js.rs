//! This module contains the [`TryFromJs`] trait, and conversions to basic Rust types.

use num_bigint::BigInt;
use num_traits::AsPrimitive;

use crate::{js_string, Context, JsBigInt, JsNativeError, JsObject, JsResult, JsString, JsValue};

mod collections;
mod tuples;

/// This trait adds a fallible and efficient conversions from a [`JsValue`] to Rust types.
pub trait TryFromJs: Sized {
    /// This function tries to convert a JavaScript value into `Self`.
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self>;
}

impl JsValue {
    /// This function is the inverse of [`TryFromJs`]. It tries to convert a [`JsValue`] to a given
    /// Rust type.
    pub fn try_js_into<T>(&self, context: &mut Context) -> JsResult<T>
    where
        T: TryFromJs,
    {
        T::try_from_js(self, context)
    }
}

impl TryFromJs for bool {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Boolean(b) => Ok(*b),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a boolean")
                .into()),
        }
    }
}

impl TryFromJs for String {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::String(s) => s.to_std_string().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("could not convert JsString to Rust string: {e}"))
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a String")
                .into()),
        }
    }
}

impl TryFromJs for JsString {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::String(s) => Ok(s.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a String")
                .into()),
        }
    }
}

impl<T> TryFromJs for Option<T>
where
    T: TryFromJs,
{
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Null | JsValue::Undefined => Ok(None),
            value => Ok(Some(T::try_from_js(value, context)?)),
        }
    }
}

impl<T> TryFromJs for Vec<T>
where
    T: TryFromJs,
{
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let JsValue::Object(object) = value else {
            return Err(JsNativeError::typ()
                .with_message("cannot convert value to a Vec")
                .into());
        };

        let length = object
            .get(js_string!("length"), context)?
            .to_length(context)?;
        let length = match usize::try_from(length) {
            Ok(length) => length,
            Err(e) => {
                return Err(JsNativeError::typ()
                    .with_message(format!("could not convert length to usize: {e}"))
                    .into());
            }
        };
        let mut vec = Vec::with_capacity(length);
        for i in 0..length {
            let value = object.get(i, context)?;
            vec.push(T::try_from_js(&value, context)?);
        }

        Ok(vec)
    }
}

impl TryFromJs for JsObject {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Object(o) => Ok(o.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a Object")
                .into()),
        }
    }
}

impl TryFromJs for JsBigInt {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::BigInt(b) => Ok(b.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a BigInt")
                .into()),
        }
    }
}

impl TryFromJs for BigInt {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::BigInt(b) => Ok(b.as_inner().clone()),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a BigInt")
                .into()),
        }
    }
}

impl TryFromJs for JsValue {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        Ok(value.clone())
    }
}

impl TryFromJs for f64 {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => Ok((*i).into()),
            JsValue::Rational(r) => Ok(*r),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a f64")
                .into()),
        }
    }
}

fn from_f64<T>(v: f64) -> Option<T>
where
    T: AsPrimitive<f64>,
    f64: AsPrimitive<T>,
{
    if <f64 as AsPrimitive<T>>::as_(v).as_().to_bits() == v.to_bits() {
        return Some(v.as_());
    }
    None
}

impl TryFromJs for i8 {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a i8: {e}"))
                    .into()
            }),
            JsValue::Rational(f) => from_f64(*f).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("cannot convert value to a i8")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a i8")
                .into()),
        }
    }
}

impl TryFromJs for u8 {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a u8: {e}"))
                    .into()
            }),
            JsValue::Rational(f) => from_f64(*f).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("cannot convert value to a u8")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a u8")
                .into()),
        }
    }
}

impl TryFromJs for i16 {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a i16: {e}"))
                    .into()
            }),
            JsValue::Rational(f) => from_f64(*f).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("cannot convert value to a i16")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a i16")
                .into()),
        }
    }
}

impl TryFromJs for u16 {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a iu16: {e}"))
                    .into()
            }),
            JsValue::Rational(f) => from_f64(*f).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("cannot convert value to a u16")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a u16")
                .into()),
        }
    }
}

impl TryFromJs for i32 {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => Ok(*i),
            JsValue::Rational(f) => from_f64(*f).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("cannot convert value to a i32")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a i32")
                .into()),
        }
    }
}

impl TryFromJs for u32 {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a u32: {e}"))
                    .into()
            }),
            JsValue::Rational(f) => from_f64(*f).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("cannot convert value to a u32")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a u32")
                .into()),
        }
    }
}

impl TryFromJs for i64 {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => Ok((*i).into()),
            JsValue::Rational(f) => from_f64(*f).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("cannot convert value to a i64")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a i64")
                .into()),
        }
    }
}

impl TryFromJs for u64 {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a u64: {e}"))
                    .into()
            }),
            JsValue::Rational(f) => from_f64(*f).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("cannot convert value to a u64")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a u64")
                .into()),
        }
    }
}

impl TryFromJs for usize {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a usize: {e}"))
                    .into()
            }),
            JsValue::Rational(f) => from_f64(*f).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("cannot convert value to a usize")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a usize")
                .into()),
        }
    }
}

impl TryFromJs for i128 {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => Ok((*i).into()),
            JsValue::Rational(f) => from_f64(*f).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("cannot convert value to a i128")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a i128")
                .into()),
        }
    }
}

impl TryFromJs for u128 {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a u128: {e}"))
                    .into()
            }),
            JsValue::Rational(f) => from_f64(*f).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("cannot convert value to a u128")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a u128")
                .into()),
        }
    }
}

#[test]
fn integer_floating_js_value_to_integer() {
    let context = &mut Context::default();

    assert_eq!(i8::try_from_js(&JsValue::from(4.0), context), Ok(4));
    assert_eq!(u8::try_from_js(&JsValue::from(4.0), context), Ok(4));
    assert_eq!(i16::try_from_js(&JsValue::from(4.0), context), Ok(4));
    assert_eq!(u16::try_from_js(&JsValue::from(4.0), context), Ok(4));
    assert_eq!(i32::try_from_js(&JsValue::from(4.0), context), Ok(4));
    assert_eq!(u32::try_from_js(&JsValue::from(4.0), context), Ok(4));
    assert_eq!(i64::try_from_js(&JsValue::from(4.0), context), Ok(4));
    assert_eq!(u64::try_from_js(&JsValue::from(4.0), context), Ok(4));

    // Floating with fractional part
    let result = i32::try_from_js(&JsValue::from(4.000_000_000_000_001), context);
    assert!(result.is_err());

    // NaN
    let result = i32::try_from_js(&JsValue::nan(), context);
    assert!(result.is_err());

    // +Infinity
    let result = i32::try_from_js(&JsValue::positive_infinity(), context);
    assert!(result.is_err());

    // -Infinity
    let result = i32::try_from_js(&JsValue::negative_infinity(), context);
    assert!(result.is_err());
}

#[test]
fn value_into_vec() {
    use boa_engine::{run_test_actions, TestAction};
    use indoc::indoc;

    #[derive(Debug, PartialEq, Eq, boa_macros::TryFromJs)]
    struct TestStruct {
        inner: bool,
        my_int: i16,
        my_vec: Vec<String>,
    }

    run_test_actions([
        TestAction::assert_with_op(
            indoc! {r#"
            let value = {
                inner: true,
                my_int: 11,
                my_vec: ["a", "b", "c"]
            };
            value
        "#},
            |value, context| {
                let value = TestStruct::try_from_js(&value, context);

                match value {
                    Ok(value) => {
                        value
                            == TestStruct {
                                inner: true,
                                my_int: 11,
                                my_vec: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                            }
                    }
                    _ => false,
                }
            },
        ),
        TestAction::assert_with_op(
            indoc!(
                r#"
            let wrong = {
                inner: false,
                my_int: 22,
                my_vec: [{}, "e", "f"]
            };
            wrong"#
            ),
            |value, context| {
                let Err(value) = TestStruct::try_from_js(&value, context) else {
                    return false;
                };
                assert!(value.to_string().contains("TypeError"));
                true
            },
        ),
    ]);
}

#[test]
fn value_into_tuple() {
    use boa_engine::{run_test_actions, TestAction};
    use indoc::indoc;

    run_test_actions([
        TestAction::assert_with_op(indoc! {r#" [42, "hello", true] "#}, |value, context| {
            type TestType = (i32, String, bool);
            TestType::try_from_js(&value, context).unwrap() == (42, "hello".to_string(), true)
        }),
        TestAction::assert_with_op(indoc! {r#" [42, "hello", true] "#}, |value, context| {
            type TestType = (i32, String, Option<bool>, Option<u8>);
            TestType::try_from_js(&value, context).unwrap()
                == (42, "hello".to_string(), Some(true), None)
        }),
        TestAction::assert_with_op(indoc! {r#" [] "#}, |value, context| {
            type TestType = (
                Option<bool>,
                Option<bool>,
                Option<bool>,
                Option<bool>,
                Option<bool>,
                Option<bool>,
                Option<bool>,
                Option<bool>,
                Option<bool>,
                Option<bool>,
            );
            TestType::try_from_js(&value, context).unwrap()
                == (None, None, None, None, None, None, None, None, None, None)
        }),
        TestAction::assert_with_op(indoc!(r#"[42, "hello", {}]"#), |value, context| {
            type TestType = (i32, String, bool);
            let Err(value) = TestType::try_from_js(&value, context) else {
                return false;
            };
            assert!(value.to_string().contains("TypeError"));
            true
        }),
        TestAction::assert_with_op(indoc!(r#"[42, "hello"]"#), |value, context| {
            type TestType = (i32, String, bool);
            let Err(value) = TestType::try_from_js(&value, context) else {
                return false;
            };
            assert!(value.to_string().contains("TypeError"));
            true
        }),
    ]);
}

#[test]
fn value_into_map() {
    use boa_engine::{run_test_actions, TestAction};
    use indoc::indoc;

    run_test_actions([
        TestAction::assert_with_op(indoc! {r#" ({ a: 1, b: 2, c: 3 }) "#}, |value, context| {
            let value = std::collections::BTreeMap::<String, i32>::try_from_js(&value, context);

            match value {
                Ok(value) => {
                    value
                        == vec![
                            ("a".to_string(), 1),
                            ("b".to_string(), 2),
                            ("c".to_string(), 3),
                        ]
                        .into_iter()
                        .collect::<std::collections::BTreeMap<String, i32>>()
                }
                _ => false,
            }
        }),
        TestAction::assert_with_op(indoc! {r#" ({ a: 1, b: 2, c: 3 }) "#}, |value, context| {
            let value = std::collections::HashMap::<String, i32>::try_from_js(&value, context);

            match value {
                Ok(value) => {
                    value
                        == std::collections::HashMap::from_iter(
                            vec![
                                ("a".to_string(), 1),
                                ("b".to_string(), 2),
                                ("c".to_string(), 3),
                            ]
                            .into_iter()
                            .collect::<std::collections::BTreeMap<String, i32>>(),
                        )
                }
                _ => false,
            }
        }),
    ]);
}
