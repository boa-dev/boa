//! Module containing the `Response` JavaScript class and its helpers, implemented as
//! [`JsResponse`].
//!
//! See the [Response interface documentation][mdn] for more information.
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Response
use crate::fetch::headers::JsHeaders;
use boa_engine::object::builtins::JsPromise;
use boa_engine::value::{TryFromJs, TryIntoJs};
use boa_engine::{
    Context, JsData, JsNativeError, JsResult, JsString, JsValue, js_error, js_str, js_string,
};
use boa_gc::{Finalize, Gc, Trace};
use boa_interop::boa_macros::boa_class;
use http::StatusCode;

/// The type read-only property of the Response interface contains the type of the
/// response. The type determines whether scripts are able to access the response body
/// and headers.
///
/// See <https://developer.mozilla.org/en-US/docs/Web/API/Response/type>.
#[derive(Debug, Copy, Clone)]
pub enum ResponseType {
    /// This applies in any of the following cases:
    ///
    /// The request is same-origin.
    /// The requested URL's scheme is `data:`.
    /// The request's mode is navigate or websocket.
    ///
    /// With this type, all response headers are exposed except Set-Cookie.
    Basic,

    /// The request was cross-origin and was successfully processed using CORS. With this
    /// type, only CORS-safelisted response headers are exposed.
    Cors,

    /// A network error occurred. The status property is set to 0, body is null, headers
    /// are empty and immutable.
    Error,

    /// A response to a cross-origin request whose mode was set to no-cors. The status
    /// property is set to 0, body is null, headers are empty and immutable.
    Opaque,

    /// A response to a request whose redirect option was set to manual, and which was
    /// redirected by the server. The status property is set to 0, body is null, headers
    /// are empty and immutable.
    OpaqueRedirect,
}

impl ResponseType {
    /// Return the Javascript String representing this response type.
    #[must_use]
    pub fn to_string(self) -> JsString {
        match self {
            ResponseType::Basic => js_string!("basic"),
            ResponseType::Cors => js_string!("cors"),
            ResponseType::Error => js_string!("error"),
            ResponseType::Opaque => js_string!("opaque"),
            ResponseType::OpaqueRedirect => js_string!("opaqueredirect"),
        }
    }
}

impl TryFromJs for ResponseType {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let value_str = value.to_string(context)?;
        if value_str == js_str!("basic") {
            Ok(ResponseType::Basic)
        } else if value_str == js_str!("cors") {
            Ok(ResponseType::Cors)
        } else if value_str == js_str!("error") {
            Ok(ResponseType::Error)
        } else if value_str == js_str!("opaque") {
            Ok(ResponseType::Opaque)
        } else if value_str == js_str!("opaqueredirect") {
            Ok(ResponseType::OpaqueRedirect)
        } else {
            Err(js_error!(TypeError: "Invalid response type value"))
        }
    }
}

impl TryIntoJs for ResponseType {
    fn try_into_js(&self, _: &mut Context) -> JsResult<JsValue> {
        Ok(self.to_string().into())
    }
}

/// The `Response` interface of the Fetch API represents the response to a request.
//
// You can create a new Response object using the `Response` constructor, but you
// are more likely to encounter a `Response` object being returned as the result of
// another API operation.
#[derive(Clone, Debug, Trace, Finalize, JsData)]
pub struct JsResponse {
    url: JsString,

    #[unsafe_ignore_trace]
    r#type: ResponseType,

    #[unsafe_ignore_trace]
    status: Option<StatusCode>,

    headers: JsHeaders,

    body: Gc<Vec<u8>>,
}

impl JsResponse {
    /// Create a new instance from the HTTP response and the URL that requested it.
    #[must_use]
    pub fn basic(url: JsString, inner: http::Response<Vec<u8>>) -> Self {
        let (parts, body) = inner.into_parts();
        let status = Some(parts.status);
        let headers = JsHeaders::from_http(parts.headers);
        let body = Gc::new(body);

        Self {
            url,
            r#type: ResponseType::Basic,
            status,
            headers,
            body,
        }
    }

    /// Create a new instance of response that is an error.
    #[must_use]
    pub fn error() -> Self {
        Self {
            url: js_string!(""),
            r#type: ResponseType::Error,
            status: None,
            headers: JsHeaders::default(),
            body: Gc::new(Vec::new()),
        }
    }

    /// Return a copy of the body.
    #[must_use]
    pub fn body(&self) -> Gc<Vec<u8>> {
        self.body.clone()
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
    #[boa(static)]
    #[boa(rename = "error")]
    fn error_() -> Self {
        Self::error()
    }

    #[boa(constructor)]
    fn constructor(_body: Option<JsValue>, _options: JsResponseOptions) -> Self {
        Self::basic(js_string!(""), http::Response::new(Vec::new()))
    }

    #[boa(getter)]
    fn status(&self) -> u16 {
        // 0 is a special case for error responses.
        self.status.map_or(0, |s| s.as_u16())
    }

    #[boa(getter)]
    fn status_text(&self) -> JsString {
        if let Some(status) = self.status {
            JsString::from(status.canonical_reason().unwrap_or_else(|| status.as_str()))
        } else {
            JsString::default()
        }
    }

    #[boa(getter)]
    fn headers(&self) -> JsHeaders {
        self.headers.clone()
    }

    #[boa(getter)]
    #[boa(rename = "type")]
    fn r#type(&self) -> JsString {
        self.r#type.to_string()
    }

    #[boa(getter)]
    fn url(&self) -> JsString {
        self.url.clone()
    }

    fn text(&self, context: &mut Context) -> JsPromise {
        let body = self.body();
        JsPromise::from_future(
            async move |_| {
                let body = String::from_utf8_lossy(body.as_ref());
                Ok(JsString::from(body).into())
            },
            context,
        )
    }

    fn json(&self, context: &mut Context) -> JsPromise {
        let body = self.body();
        JsPromise::from_future(
            async move |context| {
                let json_string = String::from_utf8_lossy(body.as_ref());
                let json = serde_json::from_str::<serde_json::Value>(&json_string)
                    .map_err(|e| JsNativeError::syntax().with_message(e.to_string()))?;

                JsValue::from_json(&json, &mut context.borrow_mut())
            },
            context,
        )
    }
}
