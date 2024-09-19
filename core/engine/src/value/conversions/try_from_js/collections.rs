//! [`JsValue`] conversions for std collections.

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use boa_macros::js_str;

use crate::object::{JsArray, JsMap};
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

        // JsMap case
        if let Ok(js_map) = JsMap::from_object(object.clone()) {
            let mut map = Self::default();
            let f = |key, value, context: &mut _| {
                map.insert(
                    K::try_from_js(&key, context)?,
                    V::try_from_js(&value, context)?,
                );
                Ok(())
            };
            for_each_elem_in_js_map(js_map, f, context)?;
            return Ok(map);
        }

        // key-valued JsObject case:
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

        // JsMap case
        if let Ok(js_map) = JsMap::from_object(object.clone()) {
            let mut map = Self::default();
            let f = |key, value, context: &mut _| {
                map.insert(
                    K::try_from_js(&key, context)?,
                    V::try_from_js(&value, context)?,
                );
                Ok(())
            };
            for_each_elem_in_js_map(js_map, f, context)?;
            return Ok(map);
        }

        // key-valued JsObject case:
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

fn for_each_elem_in_js_map<F>(js_map: JsMap, mut f: F, context: &mut Context) -> JsResult<()>
where
    F: FnMut(JsValue, JsValue, &mut Context) -> JsResult<()>,
{
    let unexp_obj_err = || {
        JsResult::Err(
            JsNativeError::typ()
                .with_message("MapIterator return unexpected object")
                .into(),
        )
    };

    let iter = js_map.entries(context)?;
    loop {
        let next = iter.next(context)?;
        let Some(iter_obj) = next.as_object() else {
            return unexp_obj_err();
        };

        let done = iter_obj.get(js_str!("done"), context)?;
        let Some(done) = done.as_boolean() else {
            return unexp_obj_err();
        };
        if done {
            break;
        }

        let value = iter_obj.get(js_str!("value"), context)?;
        let Some(js_obj) = value.as_object() else {
            return unexp_obj_err();
        };
        let arr = JsArray::from_object(js_obj.clone())?;

        let key = arr.at(0, context)?;
        let value = arr.at(1, context)?;

        f(key, value, context)?;
    }
    Ok(())
}
