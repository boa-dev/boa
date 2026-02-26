//! Boa's implementation of JavaScript's `URLSearchParams` Web API class.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [WHATWG `URLSearchParams` specification][spec]
//!
//! [spec]: https://url.spec.whatwg.org/#urlsearchparams
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams
#![allow(clippy::needless_pass_by_value)]

use boa_engine::realm::Realm;
use boa_engine::value::Convert;
use boa_engine::{
    Context, Finalize, JsData, JsResult, JsString, JsValue, Trace, boa_class, boa_module, js_error,
};

/// The `URLSearchParams` class provides utility methods to work with the query
/// string of a URL.
#[derive(Debug, Clone, JsData, Trace, Finalize)]
#[boa_gc(unsafe_no_drop)]
pub struct UrlSearchParams {
    #[unsafe_ignore_trace]
    pairs: Vec<(String, String)>,
}

impl UrlSearchParams {
    /// Register the `URLSearchParams` class into the realm.
    ///
    /// # Errors
    /// This will error if the context or realm cannot register the class.
    pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
        js_module::boa_register(realm, context)
    }

    /// Create a new `UrlSearchParams` from a query string (without leading `?`).
    pub(crate) fn from_query(query: &str) -> Self {
        let pairs = url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();
        Self { pairs }
    }

    /// Serialize the pairs back to a query string.
    fn serialize(&self) -> String {
        url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(&self.pairs)
            .finish()
    }
}

#[boa_class(rename = "URLSearchParams")]
#[boa(rename_all = "camelCase")]
impl UrlSearchParams {
    /// Create a new `URLSearchParams` object.
    ///
    /// # Errors
    /// None — construction always succeeds.
    #[boa(constructor)]
    pub fn new(init: Option<Convert<String>>) -> Self {
        let query = init.map_or(String::new(), |c| c.0.clone());
        let query = query.strip_prefix('?').unwrap_or(&query);
        Self::from_query(query)
    }

    /// Appends a new name-value pair.
    fn append(&mut self, name: Convert<String>, value: Convert<String>) {
        self.pairs.push((name.0.clone(), value.0.clone()));
    }

    /// Deletes all pairs with the given name, optionally filtering by value.
    fn delete(&mut self, name: Convert<String>, value: Option<Convert<String>>) {
        let name = &name.0;
        match value {
            Some(Convert(ref val)) => self.pairs.retain(|(n, v)| n != name || v != val),
            None => self.pairs.retain(|(n, _)| n != name),
        }
    }

    /// Returns the first value associated with the given name, or null.
    fn get(&self, name: Convert<String>) -> JsValue {
        let name = &name.0;
        self.pairs
            .iter()
            .find(|(n, _)| n == name)
            .map_or(JsValue::null(), |(_, v)| {
                JsValue::from(JsString::from(v.as_str()))
            })
    }

    /// Returns all values associated with the given name.
    fn get_all(&self, name: Convert<String>, context: &mut Context) -> JsResult<JsValue> {
        let name = &name.0;
        let values: Vec<JsValue> = self
            .pairs
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(_, v)| JsValue::from(JsString::from(v.as_str())))
            .collect();
        let arr = boa_engine::object::builtins::JsArray::from_iter(values, context);
        Ok(arr.into())
    }

    /// Returns whether a pair with the given name (and optionally value) exists.
    fn has(&self, name: Convert<String>, value: Option<Convert<String>>) -> bool {
        let name = &name.0;
        match value {
            Some(Convert(ref val)) => self.pairs.iter().any(|(n, v)| n == name && v == val),
            None => self.pairs.iter().any(|(n, _)| n == name),
        }
    }

    /// Sets the value for a given name. Removes all other pairs with the same name.
    fn set(&mut self, name: Convert<String>, value: Convert<String>) {
        let name_str = name.0.clone();
        let value_str = value.0.clone();
        let mut found = false;
        self.pairs.retain_mut(|(n, v)| {
            if *n == name_str {
                if found {
                    return false;
                }
                found = true;
                *v = value_str.clone();
            }
            true
        });
        if !found {
            self.pairs.push((name_str, value_str));
        }
    }

    /// Sorts all pairs by name, preserving relative order of pairs with the same name.
    fn sort(&mut self) {
        self.pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
    }

    /// Returns the number of search parameters.
    #[boa(getter)]
    fn size(&self) -> usize {
        self.pairs.len()
    }

    /// Returns the query string representation.
    fn to_string(&self) -> JsString {
        JsString::from(self.serialize())
    }

    /// Calls a callback for each name-value pair.
    fn for_each(
        &self,
        callback: JsValue,
        this_arg: JsValue,
        context: &mut Context,
    ) -> JsResult<()> {
        let callback = callback
            .as_callable()
            .ok_or_else(|| js_error!(TypeError: "callback is not a function"))?;
        for (name, value) in &self.pairs {
            callback.call(
                &this_arg,
                &[
                    JsValue::from(JsString::from(value.as_str())),
                    JsValue::from(JsString::from(name.as_str())),
                ],
                context,
            )?;
        }
        Ok(())
    }

    /// Returns an array of [name, value] arrays (entries iterator substitute).
    fn entries(&self, context: &mut Context) -> JsResult<JsValue> {
        let entries: Vec<JsValue> = self
            .pairs
            .iter()
            .map(|(n, v)| {
                let pair = boa_engine::object::builtins::JsArray::from_iter(
                    [
                        JsValue::from(JsString::from(n.as_str())),
                        JsValue::from(JsString::from(v.as_str())),
                    ],
                    context,
                );
                pair.into()
            })
            .collect();
        let arr = boa_engine::object::builtins::JsArray::from_iter(entries, context);
        Ok(arr.into())
    }

    /// Returns an array of all parameter names.
    fn keys(&self, context: &mut Context) -> JsResult<JsValue> {
        let keys: Vec<JsValue> = self
            .pairs
            .iter()
            .map(|(n, _)| JsValue::from(JsString::from(n.as_str())))
            .collect();
        let arr = boa_engine::object::builtins::JsArray::from_iter(keys, context);
        Ok(arr.into())
    }

    /// Returns an array of all parameter values.
    fn values(&self, context: &mut Context) -> JsResult<JsValue> {
        let values: Vec<JsValue> = self
            .pairs
            .iter()
            .map(|(_, v)| JsValue::from(JsString::from(v.as_str())))
            .collect();
        let arr = boa_engine::object::builtins::JsArray::from_iter(values, context);
        Ok(arr.into())
    }
}

/// JavaScript module containing the `URLSearchParams` class.
#[boa_module]
pub mod js_module {
    type UrlSearchParams = super::UrlSearchParams;
}
