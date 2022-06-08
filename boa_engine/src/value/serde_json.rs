//! This module implements the conversions from and into [`serde_json::Value`].

use super::JsValue;
use crate::{
    builtins::Array,
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
    pub fn from_json(json: &Value, context: &mut Context) -> JsResult<Self> {
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
                    context.construct_type_error(format!(
                        "could not convert JSON number {num} to JsValue"
                    ))
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
                let js_obj = context.construct_object();
                for (key, value) in obj {
                    let property = PropertyDescriptor::builder()
                        .value(Self::from_json(value, context)?)
                        .writable(true)
                        .enumerable(true)
                        .configurable(true);
                    js_obj.borrow_mut().insert(key.as_str(), property);
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
    pub fn to_json(&self, context: &mut Context) -> JsResult<Value> {
        match self {
            Self::Null => Ok(Value::Null),
            Self::Undefined => todo!("undefined to JSON"),
            &Self::Boolean(b) => Ok(b.into()),
            Self::String(string) => Ok(string.as_str().into()),
            &Self::Rational(rat) => Ok(rat.into()),
            &Self::Integer(int) => Ok(int.into()),
            Self::BigInt(_bigint) => context.throw_type_error("cannot convert bigint to JSON"),
            Self::Object(obj) => {
                if obj.is_array() {
                    let len = obj.length_of_array_like(context)?;
                    let mut arr = Vec::with_capacity(len);

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
                            PropertyKey::String(string) => string.as_str().to_owned(),
                            PropertyKey::Index(i) => i.to_string(),
                            PropertyKey::Symbol(_sym) => {
                                return context.throw_type_error("cannot convert Symbol to JSON")
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
            Self::Symbol(_sym) => context.throw_type_error("cannot convert Symbol to JSON"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::object::JsArray;
    use crate::{Context, JsValue};

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
        assert_eq!(obj.get("name", &mut context).unwrap(), "John Doe".into());
        assert_eq!(obj.get("age", &mut context).unwrap(), 43_i32.into());
        assert_eq!(obj.get("minor", &mut context).unwrap(), false.into());
        assert_eq!(obj.get("adult", &mut context).unwrap(), true.into());
        {
            let extra = obj.get("extra", &mut context).unwrap();
            let extra = extra.as_object().unwrap();
            assert!(extra.get("address", &mut context).unwrap().is_null());
        }
        {
            let phones = obj.get("phones", &mut context).unwrap();
            let phones = phones.as_object().unwrap();

            let arr = JsArray::from_object(phones.clone(), &mut context).unwrap();
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
            .eval(
                r#"
                1000000 + 500
            "#,
            )
            .unwrap();
        let add: u32 = serde_json::from_value(add.to_json(&mut context).unwrap()).unwrap();
        assert_eq!(add, 1_000_500);

        let sub = context
            .eval(
                r#"
                1000000 - 500
            "#,
            )
            .unwrap();
        let sub: u32 = serde_json::from_value(sub.to_json(&mut context).unwrap()).unwrap();
        assert_eq!(sub, 999_500);

        let mult = context
            .eval(
                r#"
                1000000 * 500
            "#,
            )
            .unwrap();
        let mult: u32 = serde_json::from_value(mult.to_json(&mut context).unwrap()).unwrap();
        assert_eq!(mult, 500_000_000);

        let div = context
            .eval(
                r#"
                1000000 / 500
            "#,
            )
            .unwrap();
        let div: u32 = serde_json::from_value(div.to_json(&mut context).unwrap()).unwrap();
        assert_eq!(div, 2000);

        let rem = context
            .eval(
                r#"
                233894 % 500
            "#,
            )
            .unwrap();
        let rem: u32 = serde_json::from_value(rem.to_json(&mut context).unwrap()).unwrap();
        assert_eq!(rem, 394);

        let pow = context
            .eval(
                r#"
                36 ** 5
            "#,
            )
            .unwrap();

        let pow: u32 = serde_json::from_value(pow.to_json(&mut context).unwrap()).unwrap();

        assert_eq!(pow, 60466176);
    }
}
