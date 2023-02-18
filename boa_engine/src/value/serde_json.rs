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
                let js_obj = JsObject::with_object_proto(context);
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
    use boa_parser::Source;

    use crate::object::JsArray;
    use crate::{string::utf16, Context, JsValue};

    #[test]
    fn ut_json_conversions() {
        let data = r#"
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
          }"#;

        let json: serde_json::Value = serde_json::from_str(data).unwrap();
        assert!(json.is_object());

        let mut context = Context::default();
        let value = JsValue::from_json(&json, &mut context).unwrap();

        let obj = value.as_object().unwrap();
        assert_eq!(
            obj.get(utf16!("name"), &mut context).unwrap(),
            "John Doe".into()
        );
        assert_eq!(obj.get(utf16!("age"), &mut context).unwrap(), 43_i32.into());
        assert_eq!(
            obj.get(utf16!("minor"), &mut context).unwrap(),
            false.into()
        );
        assert_eq!(obj.get(utf16!("adult"), &mut context).unwrap(), true.into());
        {
            let extra = obj.get(utf16!("extra"), &mut context).unwrap();
            let extra = extra.as_object().unwrap();
            assert!(extra
                .get(utf16!("address"), &mut context)
                .unwrap()
                .is_null());
        }
        {
            let phones = obj.get(utf16!("phones"), &mut context).unwrap();
            let phones = phones.as_object().unwrap();

            let arr = JsArray::from_object(phones.clone()).unwrap();
            assert_eq!(arr.at(0, &mut context).unwrap(), "+44 1234567".into());
            assert_eq!(arr.at(1, &mut context).unwrap(), JsValue::from(-45_i32));
            assert!(arr.at(2, &mut context).unwrap().is_object());
            assert_eq!(arr.at(3, &mut context).unwrap(), true.into());
        }

        assert_eq!(json, value.to_json(&mut context).unwrap());
    }

    #[test]
    fn integer_ops_to_json() {
        let mut context = Context::default();

        let add = context
            .eval_script(Source::from_bytes(
                r#"
                1000000 + 500
            "#,
            ))
            .unwrap();
        let add: u32 = serde_json::from_value(add.to_json(&mut context).unwrap()).unwrap();
        assert_eq!(add, 1_000_500);

        let sub = context
            .eval_script(Source::from_bytes(
                r#"
                1000000 - 500
            "#,
            ))
            .unwrap();
        let sub: u32 = serde_json::from_value(sub.to_json(&mut context).unwrap()).unwrap();
        assert_eq!(sub, 999_500);

        let mult = context
            .eval_script(Source::from_bytes(
                r#"
                1000000 * 500
            "#,
            ))
            .unwrap();
        let mult: u32 = serde_json::from_value(mult.to_json(&mut context).unwrap()).unwrap();
        assert_eq!(mult, 500_000_000);

        let div = context
            .eval_script(Source::from_bytes(
                r#"
                1000000 / 500
            "#,
            ))
            .unwrap();
        let div: u32 = serde_json::from_value(div.to_json(&mut context).unwrap()).unwrap();
        assert_eq!(div, 2000);

        let rem = context
            .eval_script(Source::from_bytes(
                r#"
                233894 % 500
            "#,
            ))
            .unwrap();
        let rem: u32 = serde_json::from_value(rem.to_json(&mut context).unwrap()).unwrap();
        assert_eq!(rem, 394);

        let pow = context
            .eval_script(Source::from_bytes(
                r#"
                36 ** 5
            "#,
            ))
            .unwrap();

        let pow: u32 = serde_json::from_value(pow.to_json(&mut context).unwrap()).unwrap();

        assert_eq!(pow, 60_466_176);
    }
}
