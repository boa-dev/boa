//! Module containing the `Response` JavaScript class and its helpers.
use crate::fetch::headers::JsHeaders;
use boa_engine::object::builtins::JsPromise;
use boa_engine::value::{TryFromJs, TryIntoJs};
use boa_engine::{js_string, Context, JsData, JsString, JsValue};
use boa_gc::{Finalize, Trace};
use boa_interop::boa_macros::boa_class;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

/// The `Response` interface of the Fetch API represents the response to a request.
//
// You can create a new Response object using the `Response` constructor, but you
// are more likely to encounter a `Response` object being returned as the result of
// another API operation.
#[derive(Clone, Debug, Trace, Finalize, JsData)]
pub struct JsResponse {
    url: JsString,

    #[unsafe_ignore_trace]
    inner: Rc<RefCell<http::Response<Option<Vec<u8>>>>>,
}

impl JsResponse {
    /// Create a new instance from the HTTP response and the URL that requested it.
    #[must_use]
    pub fn new(url: JsString, inner: http::Response<Option<Vec<u8>>>) -> Self {
        Self {
            url,
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    /// Return a copy of the inner response.
    #[must_use]
    pub fn inner(&self) -> Rc<RefCell<http::Response<Option<Vec<u8>>>>> {
        self.inner.clone()
    }
}

/// Options used in the construction of a `Response` object.
#[derive(Debug, Clone, TryFromJs, TryIntoJs, Trace, Finalize, JsData)]
#[boa(rename_all = "camelCase")]
pub struct JsResponseOptions {
    status: Option<u16>,
    status_text: Option<JsString>,
    headers: Option<JsHeaders>,
}

#[boa_class(rename = "Response")]
#[boa(rename_all = "camelCase")]
impl JsResponse {
    #[boa(constructor)]
    fn constructor(_body: Option<JsValue>, _options: JsResponseOptions) -> Self {
        Self {
            url: js_string!(""),
            inner: Rc::new(RefCell::new(http::Response::new(Some(Vec::new())))),
        }
    }

    #[boa(getter)]
    fn status(&self) -> u16 {
        self.inner.borrow().status().as_u16()
    }

    #[boa(getter)]
    fn status_text(&self) -> JsString {
        let status = self.inner.borrow().status();

        JsString::from(status.canonical_reason().unwrap_or_else(|| status.as_str()))
    }

    #[boa(getter)]
    fn url(&self) -> JsString {
        self.url.clone()
    }

    #[boa(getter)]
    fn text(&self, context: &mut Context) -> JsPromise {
        let body = JsString::from(
            self.inner
                .borrow()
                .body()
                .as_ref()
                .map_or_else(|| Cow::Borrowed(""), |body| String::from_utf8_lossy(body)),
        );

        JsPromise::resolve(body, context)
    }
}
