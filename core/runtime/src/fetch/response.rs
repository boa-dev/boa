//! Module containing the `Response` JavaScript class and its helpers, implemented as
//! [`JsResponse`].
//!
//! See the [Response interface documentation][mdn] for more information.
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Response

use crate::fetch::headers::JsHeaders;
use boa_engine::object::builtins::{JsPromise, JsUint8Array};
use boa_engine::value::{Convert, TryFromJs, TryIntoJs};
use boa_engine::{
    Context, JsData, JsNativeError, JsResult, JsString, JsValue, boa_class, js_error, js_str,
    js_string,
};
use boa_gc::{Finalize, Trace};
use http::StatusCode;
use std::rc::Rc;

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
    /// The request's mode is `navigate` or `websocket`.
    ///
    /// With this type, all response headers are exposed except Set-Cookie.
    Basic,

    /// The request was cross-origin and was successfully processed using CORS. With this
    /// type, only CORS-safelisted response headers are exposed.
    Cors,

    /// A network error occurred. The status property is set to 0, `body` is null, headers
    /// are empty and immutable.
    Error,

    /// A response to a cross-origin request whose mode was set to no-cors. The status
    /// property is set to 0, `body` is null, headers are empty and immutable.
    Opaque,

    /// A response to a request whose redirect option was set to manual and which was
    /// redirected by the server. The status property is set to 0, `body` is null, headers
    /// are empty and immutable.
    OpaqueRedirect,
}

impl ResponseType {
    /// Return the JavaScript String representing this response type.
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

    #[unsafe_ignore_trace]
    body: Rc<Vec<u8>>,
}

impl JsResponse {
    /// Create a new instance from the HTTP response and the URL that requested it.
    #[must_use]
    pub fn basic(url: JsString, inner: http::Response<Vec<u8>>) -> Self {
        let (parts, body) = inner.into_parts();
        let status = Some(parts.status);
        let headers = JsHeaders::from_http(parts.headers);
        let body = Rc::new(body);

        Self {
            url,
            r#type: ResponseType::Basic,
            status,
            headers,
            body,
        }
    }

    /// Create a new instance of [`JsResponse`] that is an error.
    #[must_use]
    pub fn error() -> Self {
        Self {
            url: js_string!(""),
            r#type: ResponseType::Error,
            status: None,
            headers: JsHeaders::default(),
            body: Rc::new(Vec::new()),
        }
    }

    /// Return a copy of the body.
    #[must_use]
    pub fn body(&self) -> Rc<Vec<u8>> {
        self.body.clone()
    }
}

/// Parse an optional [`JsResponseOptions`] argument.
fn parse_response_options(init: &JsValue, context: &mut Context) -> JsResult<JsResponseOptions> {
    if init.is_null_or_undefined() {
        Ok(JsResponseOptions::default())
    } else {
        JsResponseOptions::try_from_js(init, context)
    }
}

/// Build an `http::Response<Vec<u8>>` from the parsed pieces.
fn build_response(
    status: u16,
    headers: &JsHeaders,
    body: Vec<u8>,
) -> JsResult<http::Response<Vec<u8>>> {
    let mut builder = http::Response::builder().status(status);
    for (k, v) in headers.as_header_map().borrow().iter() {
        builder = builder.header(k, v);
    }
    builder
        .body(body)
        .map_err(|e| JsNativeError::error().with_message(e.to_string()).into())
}

/// Options used in the construction of a `Response` object.
#[derive(Debug, Default, Clone, TryFromJs, TryIntoJs, Trace, Finalize, JsData)]
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

    /// Creates a `Response` with a JSON-serialized body and `Content-Type: application/json`.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-response-json> and
    /// <https://developer.mozilla.org/en-US/docs/Web/API/Response/json_static>.
    #[boa(static)]
    #[boa(rename = "json")]
    fn json_static(data: JsValue, init: JsValue, context: &mut Context) -> JsResult<Self> {
        let json_val = data.to_json(context)?.ok_or_else(|| {
            JsNativeError::typ().with_message("value cannot be serialized to JSON")
        })?;
        let json_bytes = serde_json::to_vec(&json_val)
            .map_err(|e| JsNativeError::error().with_message(e.to_string()))?;

        let mut options = parse_response_options(&init, context)?;
        let status = options.status.unwrap_or(200);
        let mut headers = std::mem::take(&mut options.headers).unwrap_or_default();

        // Set Content-Type if the caller did not already specify one.
        if !headers.has(Convert::from("content-type".to_string()))? {
            headers.append(
                Convert::from("content-type".to_string()),
                Convert::from("application/json".to_string()),
            )?;
        }

        Ok(Self::basic(
            js_string!(""),
            build_response(status, &headers, json_bytes)?,
        ))
    }

    /// Creates a `Response` using the constructor.
    ///
    /// See <https://developer.mozilla.org/en-US/docs/Web/API/Response/Response>.
    #[boa(constructor)]
    fn constructor(body: Option<JsValue>, init: JsValue, context: &mut Context) -> JsResult<Self> {
        let body_bytes: Vec<u8> = match body {
            None => Vec::new(),
            Some(ref val) if val.is_null() || val.is_undefined() => Vec::new(),
            Some(val) => val.to_string(context)?.to_std_string_escaped().into_bytes(),
        };

        let mut options = parse_response_options(&init, context)?;
        let status = options.status.unwrap_or(200);
        let headers = std::mem::take(&mut options.headers).unwrap_or_default();

        Ok(Self::basic(
            js_string!(""),
            build_response(status, &headers, body_bytes)?,
        ))
    }

    /// Returns the HTTP status code of the response.
    #[boa(getter)]
    #[must_use]
    pub fn status(&self) -> u16 {
        // 0 is a special case for error responses.
        self.status.map_or(0, |s| s.as_u16())
    }

    #[boa(getter)]
    fn ok(&self) -> bool {
        let status = self.status();
        (200..=299).contains(&status)
    }

    #[boa(getter)]
    fn status_text(&self) -> JsString {
        if let Some(status) = self.status {
            JsString::from(status.canonical_reason().unwrap_or_else(|| status.as_str()))
        } else {
            JsString::default()
        }
    }

    /// Returns the headers associated with the response.
    #[boa(getter)]
    #[must_use]
    pub fn headers(&self) -> JsHeaders {
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

    fn bytes(&self, context: &mut Context) -> JsPromise {
        let body = self.body.clone();
        JsPromise::from_async_fn(
            async move |context| {
                JsUint8Array::from_iter(body.iter().copied(), &mut context.borrow_mut())
                    .map(Into::into)
            },
            context,
        )
    }

    fn text(&self, context: &mut Context) -> JsPromise {
        let body = self.body.clone();
        JsPromise::from_async_fn(
            async move |_| {
                let body = String::from_utf8_lossy(body.as_ref());
                Ok(JsString::from(body).into())
            },
            context,
        )
    }

    fn json(&self, context: &mut Context) -> JsPromise {
        let body = self.body.clone();
        JsPromise::from_async_fn(
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
