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

use boa_engine::value::Convert;
use boa_engine::{
    js_error, js_string, Context, Finalize, JsData, JsResult, JsString, JsValue, Trace,
};
use boa_interop::{js_class, IntoJsFunctionCopied, JsClass};
use std::fmt::Display;

/// The `URL` class represents a (properly parsed) Uniform Resource Locator.
#[derive(Debug, Clone, JsData, Trace, Finalize)]
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

    /// Create a new `URL` object from a `url::Url`.
    pub fn new<T: TryInto<url::Url>>(url: T) -> Result<Self, T::Error> {
        url.try_into().map(Self)
    }

    /// Create a new `URL` object. Meant to be called from the JavaScript constructor.
    ///
    /// # Errors
    /// Any errors that might occur during URL parsing.
    fn js_new(Convert(ref url): Convert<String>, base: Option<Convert<String>>) -> JsResult<Self> {
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
        // Cannot avoid cloning here, unfortunately, as we would need to replace
        // the internal URL with something else.
        url.0.clone()
    }
}

js_class! {
    class Url as "URL" {
        property hash {
            fn get(this: JsClass<Url>) -> JsString {
                if let Some(f) = this.borrow().0.fragment() {
                    JsString::from(format!("#{f}"))
                } else {
                    js_string!("")
                }
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                if value.0.is_empty() {
                    this.borrow_mut().0.set_fragment(None);
                }  else if let Some(fragment) = value.0.strip_prefix('#') {
                    this.borrow_mut().0.set_fragment(Some(fragment));
                } else {
                    this.borrow_mut().0.set_fragment(Some(&value.0));
                }
            }
        }

        property host {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(this.borrow().0.host_str().unwrap_or(""))
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                if value.0.is_empty() {
                    let _ = this.borrow_mut().0.set_host(None);
                } else {
                    let _ = this.borrow_mut().0.set_host(Some(&value.0));
                }
            }
        }

        property hostname {
            fn get(this: JsClass<Url>) -> JsString {
                let host = this.borrow().0.host_str().unwrap_or("").to_string();
                if let Some(port) = this.borrow().0.port() {
                    JsString::from(format!("{host}:{port}"))
                } else {
                    JsString::from(host)
                }
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                if value.0.is_empty() {
                    let _ = this.borrow_mut().0.set_host(None);
                } else {
                    let _ = this.borrow_mut().0.set_host(Some(&value.0));
                }
            }
        }

        property href {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(format!("{}", this.borrow().0))
            }

            fn set(this: JsClass<Url>, value: Convert<String>) -> JsResult<()> {
                let url = url::Url::parse(&value.0)
                    .map_err(|e| js_error!(TypeError: "Failed to parse URL: {}", e))?;
                *this.borrow_mut() = url.into();
                Ok(())
            }
        }

        property origin {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(this.borrow().0.origin().ascii_serialization())
            }
        }

        property password {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(this.borrow().0.password().unwrap_or("").to_string())
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                let _ = this.borrow_mut().0.set_password(Some(&value.0));
            }
        }

        property pathname {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(this.borrow().0.path())
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                this.borrow_mut().0.set_path(&value.0);
            }
        }

        property port {
            fn get(this: JsClass<Url>) -> JsString {
                JsString::from(this.borrow().0.port().map_or(String::new(), |p| p.to_string()).to_string())
            }

            fn set(this: JsClass<Url>, value: Convert<JsString>) {
                if value.0.is_empty() {
                    let _ = this.borrow_mut().0.set_port(None);
                } else if let Ok(value) = value.0.to_std_string_lossy().parse::<u16>() {
                    let _ = this.borrow_mut().0.set_port(Some(value));
                }
            }
        }

        property protocol {
            fn get(this: JsClass<Url>) -> JsString {
                // The URL crate returns without a colon, but the web API requires it.
                JsString::from(format!("{}:", this.borrow().0.scheme()))
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                // Ignore errors.
                let _ = this.borrow_mut().0.set_scheme(&value.0);
            }
        }

        property search {
            fn get(this: JsClass<Url>) -> JsString {
                if let Some(query) = this.borrow().0.query() {
                    JsString::from(format!("?{query}"))
                } else {
                    js_string!("")
                }
            }

            fn set(this: JsClass<Url>, value: Convert<String>) {
                if value.0.is_empty() {
                    this.borrow_mut().0.set_query(None);
                } else if let Some(query) = value.0.strip_prefix('?') {
                    this.borrow_mut().0.set_query(Some(query));
                } else {
                    this.borrow_mut().0.set_query(Some(&value.0));
                }
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
            Self::js_new(url, base)
        }

        init(class: &mut ClassBuilder) -> JsResult<()> {
            let create_object_url = (|| -> JsResult<()> {
                    Err(js_error!(Error: "URL.createObjectURL is not implemented"))
                })
                .into_js_function_copied(class.context());
            let can_parse = (|url: Convert<String>, base: Option<Convert<String>>| {
                    Url::js_new(url, base).is_ok()
                })
                .into_js_function_copied(class.context());
            let parse = (|url: Convert<String>, base: Option<Convert<String>>, context: &mut Context| {
                    Url::js_new(url, base)
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
