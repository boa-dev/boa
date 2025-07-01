//! Boa's implementation of JavaScript's `URL` Web API class.
//!
//! The `URL` class can be instantiated from any global object.
//! This relies on the `url` feature.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [WHATWG `URL` specification][spec]
//!
//! [spec]: https://url.spec.whatwg.org/
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/URL
#![cfg(feature = "url")]
#![allow(clippy::needless_pass_by_value)]

#[cfg(test)]
mod tests;

use boa_engine::class::{Class, ClassBuilder};
use boa_engine::realm::Realm;
use boa_engine::value::Convert;
use boa_engine::{js_error, Context, Finalize, JsData, JsResult, JsString, JsValue, Trace};
use boa_interop::boa_macros::boa_class;
use std::fmt::Display;

/// The `URL` class represents a (properly parsed) Uniform Resource Locator.
#[derive(Debug, Clone, JsData, Trace, Finalize)]
#[boa_gc(unsafe_no_drop)]
pub struct Url(#[unsafe_ignore_trace] url::Url);

impl Url {
    /// Register the `URL` class into the realm. Pass `None` for the realm to
    /// register globally.
    ///
    /// # Errors
    /// This will error if the context or realm cannot register the class.
    pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
        if let Some(realm) = realm {
            let mut class_builder = ClassBuilder::new::<Self>(context);
            Url::init(&mut class_builder)?;
            let class = class_builder.build();
            realm.register_class::<Self>(class);
        } else {
            context.register_global_class::<Self>()?;
        }

        Ok(())
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<url::Url> for Url {
    fn from(url: url::Url) -> Self {
        Self(url)
    }
}

impl From<Url> for url::Url {
    fn from(url: Url) -> url::Url {
        url.0
    }
}

#[boa_class(rename = "URL")]
#[boa(rename_all = "camelCase")]
impl Url {
    /// Create a new `URL` object. Meant to be called from the JavaScript constructor.
    ///
    /// # Errors
    /// Any errors that might occur during URL parsing.
    #[boa(constructor)]
    pub fn new(Convert(ref url): Convert<String>, base: Option<Convert<String>>) -> JsResult<Self> {
        if let Some(Convert(ref base)) = base {
            let base_url = url::Url::parse(base)
                .map_err(|e| js_error!(TypeError: "Failed to parse base URL: {}", e))?;
            if base_url.cannot_be_a_base() {
                return Err(js_error!(TypeError: "Base URL {} cannot be a base", base));
            }

            let url = base_url
                .join(url)
                .map_err(|e| js_error!(TypeError: "Failed to parse URL: {}", e))?;
            Ok(Self(url))
        } else {
            let url = url::Url::parse(url)
                .map_err(|e| js_error!(TypeError: "Failed to parse URL: {}", e))?;
            Ok(Self(url))
        }
    }

    #[boa(getter)]
    fn hash(&self) -> JsString {
        JsString::from(url::quirks::hash(&self.0))
    }

    #[boa(setter)]
    #[boa(rename = "hash")]
    fn set_hash(&mut self, value: Convert<String>) {
        url::quirks::set_hash(&mut self.0, &value.0);
    }

    #[boa(getter)]
    fn hostname(&self) -> JsString {
        JsString::from(url::quirks::hostname(&self.0))
    }

    #[boa(setter)]
    #[boa(rename = "hostname")]
    fn set_hostname(&mut self, value: Convert<String>) {
        let _ = url::quirks::set_hostname(&mut self.0, &value.0);
    }

    #[boa(getter)]
    fn host(&self) -> JsString {
        JsString::from(url::quirks::host(&self.0))
    }

    #[boa(setter)]
    #[boa(rename = "host")]
    fn set_host(&mut self, value: Convert<String>) {
        let _ = url::quirks::set_host(&mut self.0, &value.0);
    }

    #[boa(getter)]
    fn href(&self) -> JsString {
        JsString::from(url::quirks::href(&self.0))
    }

    #[boa(setter)]
    #[boa(rename = "href")]
    fn set_href(&mut self, value: Convert<String>) -> JsResult<()> {
        url::quirks::set_href(&mut self.0, &value.0)
            .map_err(|e| js_error!(TypeError: "Failed to set href: {}", e))
    }

    #[boa(getter)]
    fn origin(&self) -> JsString {
        JsString::from(url::quirks::origin(&self.0))
    }

    #[boa(getter)]
    fn password(&self) -> JsString {
        JsString::from(url::quirks::password(&self.0))
    }

    #[boa(setter)]
    #[boa(rename = "password")]
    fn set_password(&mut self, value: Convert<String>) {
        let _ = url::quirks::set_password(&mut self.0, &value.0);
    }

    #[boa(getter)]
    fn pathname(&self) -> JsString {
        JsString::from(url::quirks::pathname(&self.0))
    }

    #[boa(setter)]
    #[boa(rename = "pathname")]
    fn set_pathname(&mut self, value: Convert<String>) {
        let () = url::quirks::set_pathname(&mut self.0, &value.0);
    }

    #[boa(getter)]
    fn port(&self) -> JsString {
        JsString::from(url::quirks::port(&self.0))
    }

    #[boa(setter)]
    #[boa(rename = "port")]
    fn set_port(&mut self, value: Convert<JsString>) {
        let _ = url::quirks::set_port(&mut self.0, &value.0.to_std_string_lossy());
    }

    #[boa(getter)]
    fn protocol(&self) -> JsString {
        JsString::from(url::quirks::protocol(&self.0))
    }

    #[boa(setter)]
    #[boa(rename = "protocol")]
    fn set_protocol(&mut self, value: Convert<String>) {
        let _ = url::quirks::set_protocol(&mut self.0, &value.0);
    }

    #[boa(getter)]
    fn search(&self) -> JsString {
        JsString::from(url::quirks::search(&self.0))
    }

    #[boa(setter)]
    #[boa(rename = "search")]
    fn set_search(&mut self, value: Convert<String>) {
        url::quirks::set_search(&mut self.0, &value.0);
    }

    #[boa(getter)]
    fn search_params() -> JsResult<()> {
        Err(js_error!(Error: "URL.searchParams is not implemented"))
    }

    #[boa(getter)]
    fn username(&self) -> JsString {
        JsString::from(self.0.username())
    }

    #[boa(setter)]
    #[boa(rename = "username")]
    fn set_username(&mut self, value: Convert<String>) {
        let _ = self.0.set_username(&value.0);
    }

    fn to_string(&self) -> JsString {
        JsString::from(format!("{}", self.0))
    }

    #[boa(rename = "toJSON")]
    fn to_json(&self) -> JsString {
        JsString::from(format!("{}", self.0))
    }

    #[boa(static)]
    fn create_object_url() -> JsResult<()> {
        Err(js_error!(Error: "URL.createObjectURL is not implemented"))
    }

    #[boa(static)]
    fn can_parse(url: Convert<String>, base: Option<Convert<String>>) -> bool {
        Url::new(url, base).is_ok()
    }

    #[boa(static)]
    fn parse(
        url: Convert<String>,
        base: Option<Convert<String>>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Url::new(url, base).map_or(Ok(JsValue::null()), |u| {
            Url::from_data(u, context).map(JsValue::from)
        })
    }

    #[boa(static)]
    fn revoke_object_url() -> JsResult<()> {
        Err(js_error!(Error: "URL.revokeObjectURL is not implemented"))
    }
}
