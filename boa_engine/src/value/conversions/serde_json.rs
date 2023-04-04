//! This module implements the conversions from and into [`serde_json::Value`].

use super::JsValue;
use crate::{
    builtins::Array,
    error::JsNativeError,
    object::JsObject,
    property::{PropertyDescriptor, PropertyKey},
    Context, JsResult,
};
use serde_json::{Map, Value};

impl JsValue {
    /// Converts a [`serde_json::Value`] to a `JsValue`.
    ///
    /// # Example
    ///
    /// ```
    /// use boa_engine::{Context, JsValue};
    ///
    /// let data = r#"
    ///     {
    ///         "name": "John Doe",
    ///         "age": 43,
    ///         "phones": [
    ///             "+44 1234567",
    ///             "+44 2345678"
    ///         ]
    ///      }"#;
    ///
    /// let json: serde_json::Value = serde_json::from_str(data).unwrap();
    ///
    /// let mut context = Context::default();
    /// let value = JsValue::from_json(&json, &mut context).unwrap();
    /// #
    /// # assert_eq!(json, value.to_json(&mut context).unwrap());
    /// ```
    pub fn from_json(json: &Value, context: &mut Context<'_>) -> JsResult<Self> {
        /// Biggest possible integer, as i64.
        const MAX_INT: i64 = i32::MAX as i64;

        /// Smallest possible integer, as i64.
        const MIN_INT: i64 = i32::MIN as i64;

        match json {
            Value::Null => Ok(Self::Null),
            Value::Bool(b) => Ok(Self::Boolean(*b)),
            Value::Number(num) => num
                .as_i64()
                .filter(|n| (MIN_INT..=MAX_INT).contains(n))
                .map(|i| Self::Integer(i as i32))
                .or_else(|| num.as_f64().map(Self::Rational))
                .ok_or_else(|| {
                    JsNativeError::typ()
                        .with_message(format!("could not convert JSON number {num} to JsValue"))
                        .into()
                }),
            Value::String(string) => Ok(Self::from(string.as_str())),
            Value::Array(vec) => {
                let mut arr = Vec::with_capacity(vec.len());
                for val in vec {
                    arr.push(Self::from_json(val, context)?);
                }
                Ok(Array::create_array_from_list(arr, context).into())
            }
            Value::Object(obj) => {
                let js_obj = JsObject::with_object_proto(context.intrinsics());
                for (key, value) in obj {
                    let property = PropertyDescriptor::builder()
                        .value(Self::from_json(value, context)?)
                        .writable(true)
                        .enumerable(true)
                        .configurable(true);
                    js_obj.borrow_mut().insert(key.clone(), property);
                }

                Ok(js_obj.into())
            }
        }
    }

    /// Converts the `JsValue` to a [`serde_json::Value`].
    ///
    /// # Example
    ///
    /// ```
    /// use boa_engine::{Context, JsValue};
    ///
    /// let data = r#"
    ///     {
    ///         "name": "John Doe",
    ///         "age": 43,
    ///         "phones": [
    ///             "+44 1234567",
    ///             "+44 2345678"
    ///         ]
    ///      }"#;
    ///
    /// let json: serde_json::Value = serde_json::from_str(data).unwrap();
    ///
    /// let mut context = Context::default();
    /// let value = JsValue::from_json(&json, &mut context).unwrap();
    ///
    /// let back_to_json = value.to_json(&mut context).unwrap();
    /// #
    /// # assert_eq!(json, back_to_json);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the `JsValue` is `Undefined`.
    pub fn to_json(&self, context: &mut Context<'_>) -> JsResult<Value> {
        match self {
            Self::Null => Ok(Value::Null),
            Self::Undefined => todo!("undefined to JSON"),
            &Self::Boolean(b) => Ok(b.into()),
            Self::String(string) => Ok(string.to_std_string_escaped().into()),
            &Self::Rational(rat) => Ok(rat.into()),
            &Self::Integer(int) => Ok(int.into()),
            Self::BigInt(_bigint) => Err(JsNativeError::typ()
                .with_message("cannot convert bigint to JSON")
                .into()),
            Self::Object(obj) => {
                if obj.is_array() {
                    let len = obj.length_of_array_like(context)?;
                    let mut arr = Vec::with_capacity(len as usize);

                    let obj = obj.borrow();

                    for k in 0..len as u32 {
                        let val = obj.properties().get(&k.into()).map_or(Self::Null, |desc| {
                            desc.value().cloned().unwrap_or(Self::Null)
                        });
                        arr.push(val.to_json(context)?);
                    }

                    Ok(Value::Array(arr))
                } else {
                    let mut map = Map::new();
                    for (key, property) in obj.borrow().properties().iter() {
                        let key = match &key {
                            PropertyKey::String(string) => string.to_std_string_escaped(),
                            PropertyKey::Index(i) => i.to_string(),
                            PropertyKey::Symbol(_sym) => {
                                return Err(JsNativeError::typ()
                                    .with_message("cannot convert Symbol to JSON")
                                    .into())
                            }
                        };

                        let value = match property.value() {
                            Some(val) => val.to_json(context)?,
                            None => Value::Null,
                        };

                        map.insert(key, value);
                    }

                    Ok(Value::Object(map))
                }
            }
            Self::Symbol(_sym) => Err(JsNativeError::typ()
                .with_message("cannot convert Symbol to JSON")
                .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use serde_json::json;

    use crate::object::JsArray;
    use crate::{run_test_actions, TestAction};
    use crate::{string::utf16, JsValue};

    #[test]
    fn ut_json_conversions() {
        const DATA: &str = indoc! {r#"
            {
                "name": "John Doe",
                "age": 43,
                "minor": false,
                "adult": true,
                "extra": {
                    "address": null
                },
                "phones": [
                    "+44 1234567",
                    -45,
                    {},
                    true
                ]
            }
        "#};

        run_test_actions([TestAction::inspect_context(|ctx| {
            let json: serde_json::Value = serde_json::from_str(DATA).unwrap();
            assert!(json.is_object());

            let value = JsValue::from_json(&json, ctx).unwrap();
            let obj = value.as_object().unwrap();
            assert_eq!(obj.get(utf16!("name"), ctx).unwrap(), "John Doe".into());
            assert_eq!(obj.get(utf16!("age"), ctx).unwrap(), 43_i32.into());
            assert_eq!(obj.get(utf16!("minor"), ctx).unwrap(), false.into());
            assert_eq!(obj.get(utf16!("adult"), ctx).unwrap(), true.into());
            {
                let extra = obj.get(utf16!("extra"), ctx).unwrap();
                let extra = extra.as_object().unwrap();
                assert!(extra.get(utf16!("address"), ctx).unwrap().is_null());
            }
            {
                let phones = obj.get(utf16!("phones"), ctx).unwrap();
                let phones = phones.as_object().unwrap();

                let arr = JsArray::from_object(phones.clone()).unwrap();
                assert_eq!(arr.at(0, ctx).unwrap(), "+44 1234567".into());
                assert_eq!(arr.at(1, ctx).unwrap(), JsValue::from(-45_i32));
                assert!(arr.at(2, ctx).unwrap().is_object());
                assert_eq!(arr.at(3, ctx).unwrap(), true.into());
            }

            assert_eq!(json, value.to_json(ctx).unwrap());
        })]);
    }

    #[test]
    fn integer_ops_to_json() {
        run_test_actions([
            TestAction::assert_with_op("1000000 + 500", |v, ctx| {
                v.to_json(ctx).unwrap() == json!(1_000_500)
            }),
            TestAction::assert_with_op("1000000 - 500", |v, ctx| {
                v.to_json(ctx).unwrap() == json!(999_500)
            }),
            TestAction::assert_with_op("1000000 * 500", |v, ctx| {
                v.to_json(ctx).unwrap() == json!(500_000_000)
            }),
            TestAction::assert_with_op("1000000 / 500", |v, ctx| {
                v.to_json(ctx).unwrap() == json!(2_000)
            }),
            TestAction::assert_with_op("233894 % 500", |v, ctx| {
                v.to_json(ctx).unwrap() == json!(394)
            }),
            TestAction::assert_with_op("36 ** 5", |v, ctx| {
                v.to_json(ctx).unwrap() == json!(60_466_176)
            }),
        ]);
    }
}
