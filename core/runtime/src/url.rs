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

#[cfg(test)]
mod tests;

use boa_engine::value::Convert;
use boa_engine::{
    js_error, js_string, Context, Finalize, JsData, JsResult, JsString, JsValue, Trace,
};
use boa_interop::{js_class, IntoJsFunctionCopied, JsClass};
use std::fmt::Display;

/// The `URL` class represents a (properly parsed) Uniform Resource Locator.
#[derive(Debug, Clone, JsData, Trace, Finalize)]
#[boa_gc(unsafe_no_drop)]
pub struct Url(#[unsafe_ignore_trace] url::Url);

impl Url {
    /// Register the `URL` class into the realm.
    ///
    /// # Errors
    /// This will error if the context or realm cannot register the class.
    pub fn register(context: &mut Context) -> JsResult<()> {
        context.register_global_class::<Self>()?;
        Ok(())
    }

    /// Create a new `URL` object. Meant to be called from the JavaScript constructor.
    ///
    /// # Errors
    /// Any errors that might occur during URL parsing.
    fn js_new(Convert(ref url): Convert<String>, base: Option<&Convert<String>>) -> JsResult<Self> {
        if let Some(Convert(base)) = base {
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

js_class! {
    class Url as "URL" {
        property hash {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(url::quirks::hash(&this.borrow().0))
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                url::quirks::set_hash(&mut this.borrow_mut().0, &value.0);
            }
        }

        property hostname {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(url::quirks::hostname(&this.borrow().0))
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                let _ = url::quirks::set_hostname(&mut this.borrow_mut().0, &value.0);
            }
        }

        property host {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(url::quirks::host(&this.borrow().0))
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                let _ = url::quirks::set_host(&mut this.borrow_mut().0, &value.0);
            }
        }

        property href {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(url::quirks::href(&this.borrow().0))
            }

            fn set(this: JsClass<Url>, value: Convert<String>) -> JsResult<()> {
                url::quirks::set_href(&mut this.borrow_mut().0, &value.0)
                    .map_err(|e| js_error!(TypeError: "Failed to set href: {}", e))
            }
        }

        property origin {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(url::quirks::origin(&this.borrow().0))
            }
        }

        property password {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(url::quirks::password(&this.borrow().0))
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                let _ = url::quirks::set_password(&mut this.borrow_mut().0, &value.0);
            }
        }

        property pathname {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(url::quirks::pathname(&this.borrow().0))
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                let () = url::quirks::set_pathname(&mut this.borrow_mut().0, &value.0);
            }
        }

        property port {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(url::quirks::port(&this.borrow().0))
            }

            fn set(this: JsClass<Url>, value: Convert<JsString>) {
                let _ = url::quirks::set_port(&mut this.borrow_mut().0, &value.0.to_std_string_lossy());
            }
        }

        property protocol {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(url::quirks::protocol(&this.borrow().0))
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                let _ = url::quirks::set_protocol(&mut this.borrow_mut().0, &value.0);
            }
        }

        property search {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(url::quirks::search(&this.borrow().0))
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                url::quirks::set_search(&mut this.borrow_mut().0, &value.0);
            }
        }

        property search_params as "searchParams" {
            fn get() -> JsResult<()> {
                Err(js_error!(Error: "URL.searchParams is not implemented"))
            }
        }

        property username {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(this.borrow().0.username())
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                let _ = this.borrow_mut().0.set_username(&value.0);
            }
        }

        constructor(url: Convert<String>, base: Option<Convert<String>>) {
            Self::js_new(url, base.as_ref())
        }

        init(class: &mut ClassBuilder) -> JsResult<()> {
            let create_object_url = (|| -> JsResult<()> {
                    Err(js_error!(Error: "URL.createObjectURL is not implemented"))
                })
                .into_js_function_copied(class.context());
            let can_parse = (|url: Convert<String>, base: Option<Convert<String>>| {
                    Url::js_new(url, base.as_ref()).is_ok()
                })
                .into_js_function_copied(class.context());
            let parse = (|url: Convert<String>, base: Option<Convert<String>>, context: &mut Context| {
                    Url::js_new(url, base.as_ref())
                        .map_or(Ok(JsValue::null()), |u| Url::from_data(u, context).map(JsValue::from))
                })
                .into_js_function_copied(class.context());
            let revoke_object_url = (|| -> JsResult<()> {
                    Err(js_error!(Error: "URL.revokeObjectURL is not implemented"))
                })
                .into_js_function_copied(class.context());

            class
                .static_method(js_string!("createObjectURL"), 1, create_object_url)
                .static_method(js_string!("canParse"), 2, can_parse)
                .static_method(js_string!("parse"), 2, parse)
                .static_method(js_string!("revokeObjectUrl"), 1, revoke_object_url);

            Ok(())
        }

        fn to_string as "toString"(this: JsClass<Url>) -> JsString {
            JsString::from(format!("{}", this.borrow().0))
        }

        fn to_json as "toJSON"(this: JsClass<Url>) -> JsString {
            JsString::from(format!("{}", this.borrow().0))
        }
    }
}
