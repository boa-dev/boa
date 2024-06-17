//! [`JsValue`] conversions for std collections.

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use crate::value::TryFromJs;
use crate::{Context, JsNativeError, JsResult, JsValue};

impl<K, V> TryFromJs for BTreeMap<K, V>
where
    K: TryFromJs + Ord,
    V: TryFromJs,
{
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let JsValue::Object(object) = value else {
            return Err(JsNativeError::typ()
                .with_message("cannot convert value to a BTreeMap")
                .into());
        };

        let keys = object.__own_property_keys__(context)?;

        keys.into_iter()
            .map(|key| {
                let js_value = object.get(key.clone(), context)?;
                let js_key: JsValue = key.into();

                let key = K::try_from_js(&js_key, context)?;
                let value = V::try_from_js(&js_value, context)?;

                Ok((key, value))
            })
            .collect()
    }
}

impl<K, V, S> TryFromJs for HashMap<K, V, S>
where
    K: TryFromJs + Ord + Hash,
    V: TryFromJs,
    S: std::hash::BuildHasher + Default,
{
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let JsValue::Object(object) = value else {
            return Err(JsNativeError::typ()
                .with_message("cannot convert value to a BTreeMap")
                .into());
        };

        let keys = object.__own_property_keys__(context)?;

        keys.into_iter()
            .map(|key| {
                let js_value = object.get(key.clone(), context)?;
                let js_key: JsValue = key.into();

                let key = K::try_from_js(&js_key, context)?;
                let value = V::try_from_js(&js_value, context)?;

                Ok((key, value))
            })
            .collect()
    }
}
