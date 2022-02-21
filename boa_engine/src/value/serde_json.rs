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
    pub fn from_json(json: Value, context: &mut Context) -> JsResult<Self> {
        /// Biggest possible integer, as i64.
        const MAX_INT: i64 = i32::MAX as i64;

        match json {
            Value::Null => Ok(Self::Null),
            Value::Bool(b) => Ok(Self::Boolean(b)),
            Value::Number(num) => match num.as_i64() {
                Some(s @ 0..=MAX_INT) => Ok(Self::Integer(s as i32)),
                Some(s) => Ok(Self::Rational(s as f64)),
                None => {
                    if let Some(i) = num.as_u64() {
                        Ok(Self::Rational(i as f64))
                    } else {
                        Ok(Self::Rational(num.as_f64().ok_or_else(|| {
                            context.construct_type_error(format!(
                                "could not convert JSON number {num} to JsValue"
                            ))
                        })?))
                    }
                }
            },
            Value::String(string) => Ok(Self::from(string)),
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
                    js_obj.borrow_mut().insert(key, property);
                }

                Ok(js_obj.into())
            }
        }
    }

    /// Converts the `JsValue` to a [`serde_json::Value`].
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
