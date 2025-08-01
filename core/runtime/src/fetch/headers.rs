//! The `Headers` JavaScript class, implemented as [`JsHeaders`].
//!
//! See <https://developer.mozilla.org/en-US/docs/Web/API/Headers>.
#![allow(clippy::needless_pass_by_value)]

use boa_engine::object::builtins::{JsArray, TypedJsFunction};
use boa_engine::value::{Convert, TryFromJs};
use boa_engine::{
    Context, Finalize, JsData, JsObject, JsResult, JsString, JsValue, Trace, js_error,
};
use boa_interop::JsClass;
use boa_interop::boa_macros::boa_class;
use http::header::HeaderMap as HttpHeaderMap;
use http::{HeaderName, HeaderValue};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::str::FromStr;

/// A callback function for the `forEach` method.
pub type ForEachCallback = TypedJsFunction<(JsString, JsString, JsObject), ()>;

/// Converts a JavaScript string to a valid header name (or error).
///
/// # Errors
/// If the key is not valid ASCII, an error is returned.
#[inline]
fn to_header_name(key: impl AsRef<str>) -> JsResult<HeaderName> {
    HeaderName::from_str(key.as_ref())
        .map_err(|_| js_error!("Cannot convert key to header string as it is not valid ASCII."))
}

/// Converts a JavaScript string to a valid header value (or error).
///
/// # Errors
/// If the value is not valid ASCII, an error is returned.
#[inline]
fn to_header_value(value: impl AsRef<str>) -> JsResult<HeaderValue> {
    value
        .as_ref()
        .parse()
        .map_err(|_| js_error!("Cannot convert value to header string as it is not valid ASCII."))
}

/// A JavaScript wrapper for the `Headers` object.
#[derive(Debug, Default, Clone, JsData, Trace, Finalize)]
pub struct JsHeaders {
    #[unsafe_ignore_trace]
    headers: Rc<RefCell<HttpHeaderMap>>,
}

impl TryFromJs for JsHeaders {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let o = value.to_object(context)?;

        let mut this = JsHeaders::default();
        for k in &o.own_property_keys(context)? {
            let value = o.get(k.clone(), context)?;
            this.append(
                Convert::from(k.to_string()),
                Convert::try_from_js(&value, context)?,
            )?;
        }
        Ok(this)
    }
}

impl JsHeaders {
    /// Creates a [`JsHeaders`] from an internal [`http::HeaderMap`]. Takes ownership
    /// of the inner map.
    #[must_use]
    pub fn from_http(http: HttpHeaderMap) -> Self {
        Self {
            headers: Rc::new(RefCell::new(http)),
        }
    }
}

#[boa_class(rename = "Headers")]
#[boa(rename_all = "camelCase")]
impl JsHeaders {
    #[boa(constructor)]
    fn constructor(init: JsValue, context: &mut Context) -> JsResult<Self> {
        let headers = JsHeaders::default();
        if init.is_undefined() {
            return Ok(headers);
        }

        // `init` can be a simple object literal with String values, an array of name-value
        // pairs, where each pair is a 2-element string array; or an existing Headers object.
        let mut h = headers.headers.borrow_mut();
        if let Some(other_header) = init
            .as_object()
            .as_ref()
            .and_then(JsObject::downcast_ref::<JsHeaders>)
        {
            for (key, value) in other_header.headers.borrow().iter() {
                if h.contains_key(key) {
                    h.append(key, value.clone());
                } else {
                    h.insert(key, value.clone());
                }
            }
        } else if let Ok(init) = Vec::<(String, Convert<String>)>::try_from_js(&init, context) {
            for (k, v) in init {
                let key = to_header_name(k)?;
                let value = to_header_value(&v.0)?;
                if h.contains_key(&key) {
                    h.append(key, value);
                } else {
                    h.insert(key, value);
                }
            }
        } else if let Ok(init) = BTreeMap::<String, Convert<String>>::try_from_js(&init, context) {
            for (k, v) in init {
                let key = to_header_name(k)?;
                let value = to_header_value(&v.0)?;
                if h.contains_key(&key) {
                    h.append(key, value);
                } else {
                    h.insert(key, value);
                }
            }
        } else {
            return Err(js_error!(TypeError: "Cannot convert init to header object."));
        }
        drop(h);

        Ok(headers)
    }

    /// Appends a new value onto an existing header inside a Headers object,
    /// or adds the header if it does not already exist.
    ///
    /// # Errors
    /// If the key or value is not valid ASCII, an error is returned.
    pub fn append(&mut self, key: Convert<String>, value: Convert<String>) -> JsResult<()> {
        let key = to_header_name(key.as_ref())?;
        let value = to_header_value(value.as_ref())?;
        if !self.headers.borrow_mut().append(&key, value.clone()) {
            self.headers.borrow_mut().insert(key, value);
        }
        Ok(())
    }

    /// Deletes a header from a Headers object.
    ///
    /// # Errors
    /// If the key is not valid ASCII, an error is returned.
    pub fn delete(&mut self, key: Convert<String>) -> JsResult<()> {
        let key = to_header_name(key.as_ref())?;
        self.headers.borrow_mut().remove(key);
        Ok(())
    }

    /// Returns an iterator allowing to go through all key/value pairs contained in this object.
    // TODO: This should return a JsIterator, but not such thing exists yet.
    pub fn entries(&self, context: &mut Context) -> JsValue {
        JsArray::from_iter(
            self.headers
                .borrow()
                .iter()
                .map(|(k, v)| {
                    let k: JsValue = JsString::from(k.as_str()).into();
                    let v: JsValue = JsString::from(v.to_str().unwrap_or_default()).into();
                    JsArray::from_iter([k, v], context).into()
                })
                .collect::<Vec<_>>(),
            context,
        )
        .into()
    }

    /// Executes a provided function once for each key/value pair in the Headers object.
    ///
    /// # Errors
    /// If the callback function returns an error, it is returned.
    #[allow(clippy::needless_pass_by_value)]
    #[boa(method)]
    pub fn for_each(
        this: JsClass<Self>,
        callback: ForEachCallback,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<()> {
        let object = this.inner().upcast();
        let this_arg = this_arg.unwrap_or_default();
        for (k, v) in this.clone_inner().headers.borrow().iter() {
            let k = JsString::from(k.as_str());
            let v = JsString::from(v.to_str().unwrap_or(""));
            callback.call_with_this(&this_arg, context, (v, k, object.clone()))?;
        }
        Ok(())
    }

    /// Returns a byte string of all the values in a header within a Headers object
    /// with a given name. If the requested header doesn't exist in the Headers
    /// object, it returns null.
    ///
    /// # Errors
    /// If the key is not valid ASCII, an error is returned.
    pub fn get(&self, key: JsValue, context: &mut Context) -> JsResult<JsValue> {
        let key: Convert<String> = Convert::try_from_js(&key, context)?;
        let name = to_header_name(key.as_ref())?;
        let value = self
            .headers
            .borrow()
            .get_all(name.clone())
            .into_iter()
            .map(|v| v.to_str().unwrap_or(""))
            // Use an Option<String> to accumulate the values into a single string,
            // if there are any. Otherwise, we return None.
            // Cannot use `join(",")` as we need to return undefined if none is found.
            .fold(None, |mut acc, v| {
                let str = acc.get_or_insert_with(String::new);
                if !str.is_empty() {
                    str.push(',');
                }
                str.push_str(v);
                acc
            });

        Ok(value.map_or_else(JsValue::null, |v| JsString::from(v).into()))
    }

    /// Returns an array containing the values of all Set-Cookie headers associated with a response.
    fn get_set_cookie(&self) -> Vec<JsString> {
        self.headers
            .borrow()
            .get_all("Set-Cookie")
            .into_iter()
            .map(|v| JsString::from(v.to_str().unwrap_or("")))
            .collect()
    }

    /// Returns a boolean stating whether a Headers object contains a certain header.
    ///
    /// # Errors
    /// If the key isn't a valid header name, this will error.
    pub fn has(&self, key: Convert<String>) -> JsResult<bool> {
        let key = to_header_name(key.as_ref())?;
        Ok(self.headers.borrow().get(key).is_some())
    }

    /// Returns an iterator allowing you to go through all keys of the key/value pairs
    /// contained in this object.
    #[allow(clippy::unused_self)]
    fn keys(&self) -> Vec<JsString> {
        self.headers
            .borrow()
            .keys()
            .map(|k| JsString::from(k.as_str()))
            .collect()
    }

    /// Sets a new value for an existing header inside a Headers object, or adds the
    /// header if it does not already exist.
    fn set(&mut self, key: Convert<String>, value: Convert<String>) -> JsResult<()> {
        let key = to_header_name(key.as_ref())?;
        let value = to_header_value(value.as_ref())?;
        self.headers.borrow_mut().insert(key, value);
        Ok(())
    }

    fn values(&self) -> Vec<JsString> {
        self.headers
            .borrow()
            .values()
            .map(|v| JsString::from(v.to_str().unwrap_or("")))
            .collect()
    }
}
