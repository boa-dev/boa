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
use http::{HeaderName, HeaderValue, StatusCode};
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

    /// The default type for responses created via the `Response()` constructor or
    /// `Response.json()`. Equivalent to `Basic` in terms of header exposure.
    Default,

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
            ResponseType::Default => js_string!("default"),
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
        } else if value_str == js_str!("default") {
            Ok(ResponseType::Default)
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

/// A null body status is a status that is 101, 103, 204, or 304.
///
/// See <https://fetch.spec.whatwg.org/#null-body-status>
fn is_null_body_status(status: u16) -> bool {
    matches!(status, 101 | 103 | 204 | 304)
}

/// Validates that a string matches the `reason-phrase` token production.
///
/// ```text
/// reason-phrase = *( HTAB / SP / VCHAR / obs-text )
/// ```
///
/// See <https://httpwg.org/specs/rfc7230.html#rule.reason.phrase>
fn is_valid_reason_phrase(s: &str) -> bool {
    s.bytes()
        .all(|b| matches!(b, 0x09 | 0x20 | 0x21..=0x7E | 0x80..=0xFF))
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

    /// The HTTP status code. 0 is used for error/opaque/opaqueredirect responses.
    ///
    /// See <https://fetch.spec.whatwg.org/#concept-response-status>
    status: u16,

    /// The HTTP status message (reason phrase).
    ///
    /// See <https://fetch.spec.whatwg.org/#concept-response-status-message>
    status_text: JsString,

    headers: JsHeaders,

    #[unsafe_ignore_trace]
    body: Rc<Vec<u8>>,
}

impl JsResponse {
    /// Create a new instance from the HTTP response and the URL that requested it.
    #[must_use]
    pub fn basic(url: JsString, inner: http::Response<Vec<u8>>) -> Self {
        let (parts, body) = inner.into_parts();
        let status = parts.status.as_u16();
        let status_text = JsString::from(parts.status.canonical_reason().unwrap_or(""));
        let headers = JsHeaders::from_http(parts.headers);
        let body = Rc::new(body);

        Self {
            url,
            r#type: ResponseType::Basic,
            status,
            status_text,
            headers,
            body,
        }
    }

    /// Create a new instance of [`JsResponse`] that is an error.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-response-error>
    #[must_use]
    pub fn error() -> Self {
        Self {
            url: js_string!(""),
            r#type: ResponseType::Error,
            // A network error's status is always 0.
            // See https://fetch.spec.whatwg.org/#concept-network-error
            status: 0,
            status_text: JsString::default(),
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

/// Options used in the construction of a `Response` object.
///
/// See <https://fetch.spec.whatwg.org/#dictdef-responseinit>
#[derive(Debug, Default, Clone, TryFromJs, TryIntoJs, Trace, Finalize, JsData)]
#[boa(rename_all = "camelCase")]
pub struct JsResponseOptions {
    status: Option<u16>,
    status_text: Option<JsString>,
    headers: Option<JsHeaders>,
}

/// Initialize a `Response` from a `ResponseInit` dictionary and an optional body.
///
/// This is the shared algorithm used by the `Response()` constructor and
/// `Response.json()`.
///
/// See <https://fetch.spec.whatwg.org/#initialize-a-response>
/// `body_with_type` is a body paired with its MIME type (if any), as produced by
/// "extract a body". `None` means no body was provided.
fn initialize_response(
    init: &JsResponseOptions,
    body_with_type: Option<(Vec<u8>, Option<&str>)>,
) -> JsResult<JsResponse> {
    // Step 1: If init["status"] is not in the range 200 to 599, inclusive, throw a RangeError.
    let status = init.status.unwrap_or(200);
    if !(200..=599).contains(&status) {
        return Err(
            js_error!(RangeError: "The status provided ({}) is outside the range [200, 599].", status),
        );
    }

    // Step 2: If init["statusText"] does not match the reason-phrase token production,
    //         throw a TypeError.
    let status_text = init.status_text.clone().unwrap_or_default();
    let status_text_str = status_text.to_std_string_escaped();
    if !is_valid_reason_phrase(&status_text_str) {
        return Err(
            js_error!(TypeError: "statusText contains characters that are not valid in a reason-phrase."),
        );
    }

    // Steps 3-4: Set response's status and status message.
    // (Stored directly in the JsResponse fields below.)

    // Step 5: If init["headers"] exists, fill response's headers with init["headers"].
    let mut headers = init.headers.clone().unwrap_or_default();

    // Step 6: If body is non-null, then:
    let body = if let Some((body_bytes, body_type)) = body_with_type {
        // Step 6.1: If response's status is a null body status, throw a TypeError.
        if is_null_body_status(status) {
            return Err(
                js_error!(TypeError: "Response body is not allowed for null body status codes (101, 103, 204, 304)."),
            );
        }

        // Step 6.2: Set response's body to body's body.

        // Step 6.3: If body's type is non-null and response's header list does not contain
        //           `Content-Type`, append (`Content-Type`, body's type) to the header list.
        if let Some(content_type) = body_type
            && !headers.has(Convert::from("content-type".to_string()))?
        {
            headers.append(
                Convert::from("content-type".to_string()),
                Convert::from(content_type.to_string()),
            )?;
        }

        Rc::new(body_bytes)
    } else {
        Rc::new(Vec::new())
    };

    Ok(JsResponse {
        url: js_string!(""),
        r#type: ResponseType::Default,
        status,
        status_text,
        headers,
        body,
    })
}

/// Extract a body and its associated MIME type from a JS value.
///
/// This is a simplified implementation of the "extract a body" algorithm.
///
/// See <https://fetch.spec.whatwg.org/#concept-bodyinit-extract>
fn extract_body(val: &JsValue, context: &mut Context) -> JsResult<(Vec<u8>, Option<&'static str>)> {
    // TODO: handle other BodyInit types: Blob, ArrayBuffer, ArrayBufferView,
    //       FormData, URLSearchParams.
    // Currently only USVString is supported.
    //
    // For a USVString, the type is "text/plain;charset=UTF-8".
    // See https://fetch.spec.whatwg.org/#concept-bodyinit-extract step 6.
    let bytes = val.to_string(context)?.to_std_string_escaped().into_bytes();
    Ok((bytes, Some("text/plain;charset=UTF-8")))
}

#[boa_class(rename = "Response")]
#[boa(rename_all = "camelCase")]
impl JsResponse {
    #[boa(static)]
    #[boa(rename = "error")]
    fn error_() -> Self {
        Self::error()
    }

    /// `Response.redirect(url, status)` per Fetch spec §7.4.
    #[boa(static)]
    fn redirect(url: JsValue, status: Option<u16>, context: &mut Context) -> JsResult<Self> {
        let status = status.unwrap_or(302);
        if !matches!(status, 301 | 302 | 303 | 307 | 308) {
            return Err(js_error!(RangeError: "Invalid redirect status: {}", status));
        }
        let url_str = url.to_string(context)?.to_std_string_escaped();
        http::Uri::try_from(url_str.as_str())
            .map_err(|_| js_error!(TypeError: "Invalid URL: {}", url_str))?;

        let status_code = StatusCode::from_u16(status)
            .map_err(|_| js_error!(RangeError: "Invalid status code: {}", status))?;

        let mut headers = http::header::HeaderMap::new();
        headers.insert(
            HeaderName::from_static("location"),
            HeaderValue::try_from(url_str)
                .map_err(|_| js_error!(TypeError: "Invalid URL for header value"))?,
        );

        Ok(Self {
            url: js_string!(""),
            r#type: ResponseType::Basic,
            status: status_code.as_u16(),
            status_text: JsString::from(status_code.canonical_reason().unwrap_or("")),
            headers: JsHeaders::from_http(headers),
            body: Rc::new(Vec::new()),
        })
    }

    /// Creates a `Response` with a JSON-serialized body and `Content-Type: application/json`.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-response-json>
    #[boa(static)]
    #[boa(rename = "json")]
    fn json_static(data: JsValue, init: JsValue, context: &mut Context) -> JsResult<Self> {
        // Step 1: Let bytes be the result of running serialize a JavaScript value to JSON bytes on data.
        let json_val = data.to_json(context)?.ok_or_else(|| {
            JsNativeError::typ().with_message("value cannot be serialized to JSON")
        })?;
        let json_bytes = serde_json::to_vec(&json_val)
            .map_err(|e| JsNativeError::error().with_message(e.to_string()))?;

        // Step 2: Let body be the result of extracting bytes.
        // The MIME type for JSON is "application/json".
        let body_with_type = (json_bytes, Some("application/json"));

        // Step 3: Let responseObject be the result of creating a Response object.
        // Step 4: Perform initialize a response given responseObject, init, and (body, "application/json").
        let options = if init.is_null_or_undefined() {
            JsResponseOptions::default()
        } else {
            JsResponseOptions::try_from_js(&init, context)?
        };

        initialize_response(&options, Some(body_with_type))
    }

    /// Creates a `Response` using the constructor.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-response> and
    /// <https://fetch.spec.whatwg.org/#initialize-a-response>
    #[boa(constructor)]
    fn constructor(body: Option<JsValue>, init: JsValue, context: &mut Context) -> JsResult<Self> {
        // Step 1-2: Set up the response and its headers (handled in initialize_response).

        // Step 3: Let bodyWithType be null.
        // Step 4: If body is non-null, set bodyWithType to the result of extracting body.
        let body_with_type: Option<(Vec<u8>, Option<&'static str>)> = match body {
            None => None,
            Some(ref val) if val.is_null_or_undefined() => None,
            Some(val) => Some(extract_body(&val, context)?),
        };

        // Step 5: Perform initialize a response given this, init, and bodyWithType.
        let options = if init.is_null_or_undefined() {
            JsResponseOptions::default()
        } else {
            JsResponseOptions::try_from_js(&init, context)?
        };

        initialize_response(&options, body_with_type)
    }

    /// Returns the HTTP status code of the response.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-response-status>
    #[boa(getter)]
    #[must_use]
    pub fn status(&self) -> u16 {
        // 0 is used for error/opaque/opaqueredirect responses.
        // See https://fetch.spec.whatwg.org/#concept-network-error
        self.status
    }

    /// Returns the status message corresponding to the status code.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-response-ok>
    #[boa(getter)]
    fn ok(&self) -> bool {
        let status = self.status();
        (200..=299).contains(&status)
    }

    #[boa(getter)]
    fn status_text(&self) -> JsString {
        self.status_text.clone()
    }

    /// Returns the headers associated with the response.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-response-headers>
    #[boa(getter)]
    #[must_use]
    pub fn headers(&self) -> JsHeaders {
        self.headers.clone()
    }

    /// See <https://fetch.spec.whatwg.org/#dom-response-type>
    #[boa(getter)]
    #[boa(rename = "type")]
    fn r#type(&self) -> JsString {
        self.r#type.to_string()
    }

    /// Returns the URL of the response.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-response-url>
    #[boa(getter)]
    fn url(&self) -> JsString {
        // The spec says: return the empty string if this's response's URL is null;
        // otherwise this's response's URL, serialized with exclude fragment set to true.
        // See https://fetch.spec.whatwg.org/#dom-response-url
        let s = self.url.to_std_string_escaped();
        let without_fragment = s.find('#').map_or(s.as_str(), |i| &s[..i]);
        JsString::from(without_fragment)
    }

    /// Returns whether the response is the result of a redirect.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-response-redirected>
    #[boa(getter)]
    #[allow(clippy::unused_self)]
    fn redirected(&self) -> bool {
        // The spec says: return true if this's response's URL list's size is greater than 1.
        // TODO: track the full URL list to implement this properly.
        false
    }

    #[boa(rename = "clone")]
    fn clone_response(&self) -> Self {
        Self {
            url: self.url.clone(),
            r#type: self.r#type,
            status: self.status,
            status_text: self.status_text.clone(),
            headers: self.headers.deep_clone(),
            body: Rc::new((*self.body).clone()),
        }
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
