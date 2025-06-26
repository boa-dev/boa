//! The [`Headers`] JavaScript class.
//!
//! See <https://developer.mozilla.org/en-US/docs/Web/API/Headers>.
use boa_engine::object::builtins::{JsArray, TypedJsFunction};
use boa_engine::value::Convert;
use boa_engine::{
    js_error, Context, Finalize, JsData, JsObject, JsResult, JsString, JsValue, Trace,
};
use boa_interop::{js_class, JsClass};
use http::header::HeaderMap as HttpHeaderMap;
use http::{HeaderName, HeaderValue};
use std::str::FromStr;

/// A callback function for the `forEach` method.
pub type ForEachCallback = TypedJsFunction<(JsString, JsString, JsObject), ()>;

/// Converts a JavaScript string to a valid header name (or error).
///
/// # Errors
/// If the key is not valid ASCII, an error is returned.
#[inline]
fn to_header_name(key: &JsString) -> JsResult<HeaderName> {
    key.to_std_string()
        .map_err(|_| ())
        .and_then(|s| HeaderName::from_str(&s).map_err(|_| ()))
        .map_err(|()| js_error!("Cannot convert key to header string as it is not valid ASCII."))
}

/// Converts a JavaScript string to a valid header value (or error).
///
/// # Errors
/// If the value is not valid ASCII, an error is returned.
#[inline]
fn to_header_value(value: &JsString) -> JsResult<HeaderValue> {
    value
        .to_std_string()
        .map_err(|_| ())
        .and_then(|s| s.parse().map_err(|_| ()))
        .map_err(|()| js_error!("Cannot convert value to header string as it is not valid ASCII."))
}

/// A JavaScript wrapper for the `Headers` object.
#[derive(Debug, Clone, JsData, Trace, Finalize)]
pub struct JsHeaders {
    #[unsafe_ignore_trace]
    headers: HttpHeaderMap,
}

impl JsHeaders {
    /// Appends a new value onto an existing header inside a Headers object,
    /// or adds the header if it does not already exist.
    ///
    /// # Errors
    /// If the key or value is not valid ASCII, an error is returned.
    pub fn append(&mut self, key: &JsString, value: &JsString) -> JsResult<()> {
        let key = to_header_name(key)?;
        let value = to_header_value(value)?;
        self.headers.append(key, value);
        Ok(())
    }

    /// Deletes a header from a Headers object.
    ///
    /// # Errors
    /// If the key is not valid ASCII, an error is returned.
    pub fn delete(&mut self, key: &JsString) -> JsResult<()> {
        let key = to_header_name(key)?;
        self.headers.remove(key);
        Ok(())
    }

    /// Returns an iterator allowing to go through all key/value pairs contained in this object.
    // TODO: This should return a JsIterator, but not such thing exists yet.
    pub fn entries(&self, context: &mut Context) -> JsValue {
        JsArray::from_iter(
            self.headers
                .iter()
                .map(|(k, v)| {
                    let k: JsValue = JsString::from(k.as_str()).into();
                    let v: JsValue = JsString::from(v.to_str().unwrap_or("")).into();
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
    pub fn for_each(
        &self,
        callback: ForEachCallback,
        this_arg: Option<JsValue>,
        object: &JsObject<Self>,
        context: &mut Context,
    ) -> JsResult<()> {
        let object = object.clone().upcast();
        let this_arg = this_arg.unwrap_or_default();
        for (k, v) in &self.headers {
            let k = JsString::from(k.as_str());
            let v = JsString::from(v.to_str().unwrap_or(""));
            callback.call_with_this(&this_arg, context, (v, k, object.clone()))?;
        }
        Ok(())
    }

    /// Returns a byte string of all the values of a header within a Headers object
    /// with a given name. If the requested header doesn't exist in the Headers
    /// object, it returns null.
    ///
    /// # Errors
    /// If the key is not valid ASCII, an error is returned.
    pub fn get(&self, key: &JsString) -> JsResult<Option<JsString>> {
        let key = to_header_name(key)?;
        let value = self
            .headers
            .get_all(key)
            .into_iter()
            .map(|v| v.to_str().unwrap_or(""));

        // Use an Option<String> to accumulate the values into a single string,
        // if there are any. Otherwise, we return None.
        let value = value.fold(None, |mut acc, v| {
            let str = acc.get_or_insert_with(String::new);
            if !str.is_empty() {
                str.push(',');
            }
            str.push_str(v);
            acc
        });
        Ok(value.map(JsString::from))
    }
}

js_class! {
    class JsHeaders as "Headers" {
        constructor() {
            Ok(JsHeaders {
                headers: HttpHeaderMap::new(),
            })
        }

        fn append(
            this: JsClass<JsHeaders>,
            name: Convert<JsString>,
            value: Convert<JsString>,
        ) -> JsResult<()> {
            this.borrow_mut().append(&name.0, &value.0)
        }

        fn delete(
            this: JsClass<JsHeaders>,
            name: Convert<JsString>,
        ) -> JsResult<()> {
            this.borrow_mut().delete(&name.0)
        }

        fn entries(
            this: JsClass<JsHeaders>,
            context: &mut Context,
        ) -> JsValue {
            this.borrow().entries(context)
        }

        fn for_each as "forEach"(
            this: JsClass<JsHeaders>,
            callback: ForEachCallback,
            this_arg: Option<JsValue>,
            context: &mut Context,
        ) -> JsResult<()> {
            this.borrow().for_each(callback, this_arg, this.inner(), context)
        }

        fn get(
            this: JsClass<JsHeaders>,
            name: Convert<JsString>,
        ) -> JsResult<JsValue> {
            this.borrow()
                .get(&name.0)
                .map(|v| v.map_or(JsValue::null(), JsValue::from))
        }

        fn get_set_cookie as "getSetCookie"(
            _this: JsClass<JsHeaders>,
            _name: Convert<JsString>,
        ) -> JsResult<JsValue> {
            unimplemented!("Headers.prototype.get")
        }

        fn has(
            _this: JsClass<JsHeaders>,
            _name: Convert<JsString>,
        ) -> JsResult<bool> {
            unimplemented!("Headers.prototype.has")
        }

        fn keys(
            _this: JsClass<JsHeaders>,
        ) -> JsValue {
            unimplemented!("Headers.prototype.keys")
        }

        fn set(
            _this: JsClass<JsHeaders>,
            _name: Convert<JsString>,
            _value: Convert<JsString>,
        ) -> JsResult<()> {
            unimplemented!("Headers.prototype.set")
        }

        fn values(
            _this: JsClass<JsHeaders>,
        ) -> JsValue {
            unimplemented!("Headers.prototype.values")
        }
    }
}
