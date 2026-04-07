//! The `Request` JavaScript class and adjacent types, implemented as [`JsRequest`].
//!
//! See the [Request interface documentation][mdn] for more information.
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request
use super::HttpRequest;
use super::headers::JsHeaders;
use boa_engine::value::{Convert, TryFromJs};
use boa_engine::{
    Finalize, JsData, JsObject, JsResult, JsString, JsValue, Trace, boa_class, js_error,
};
use either::Either;
use std::mem;

/// A [RequestInit][mdn] object. This is a JavaScript object (not a
/// class) that can be used as options for creating a [`JsRequest`].
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/RequestInit
// TODO: This class does not contain all fields that are defined in the spec.
#[derive(Debug, Clone, TryFromJs, Trace, Finalize)]
pub struct RequestInit {
    body: Option<JsValue>,
    headers: Option<JsHeaders>,
    method: Option<Convert<JsString>>,
    signal: Option<JsObject>,
}

impl RequestInit {
    /// Takes the abort signal from the options, if present.
    pub fn take_signal(&mut self) -> Option<JsObject> {
        self.signal.take()
    }

    /// Create an [`http::request::Builder`] object and return both the
    /// body specified by JavaScript and the builder.
    ///
    /// # Errors
    /// If the body is not a valid type, an error is returned.
    pub fn into_request_builder(
        mut self,
        request: Option<HttpRequest<Vec<u8>>>,
    ) -> JsResult<HttpRequest<Vec<u8>>> {
        let mut builder = HttpRequest::builder();
        let mut is_get_or_head_method = true;
        let mut inherited_is_get_or_head_method = true;
        let mut inherited_body = None;
        let mut request_body: Option<Vec<u8>> = None;
        if let Some(r) = request {
            let (parts, body) = r.into_parts();
            is_get_or_head_method = matches!(parts.method, http::Method::GET | http::Method::HEAD);
            inherited_is_get_or_head_method = is_get_or_head_method;
            // https://fetch.spec.whatwg.org/#dom-request - "Let inputBody be input's request's body if input is a Request object; otherwise null."
            inherited_body = Some(body);
            builder = builder
                .method(parts.method)
                .uri(parts.uri)
                .version(parts.version);

            for (key, value) in &parts.headers {
                builder = builder.header(key, value);
            }
        }

        if let Some(headers) = self.headers.take() {
            for (k, v) in headers.as_header_map().borrow().iter() {
                builder = builder.header(k, v);
            }
        }

        if let Some(Convert(ref method)) = self.method.take() {
            let method = method.to_std_string().map_err(
                |_| js_error!(TypeError: "Request constructor: {} is an invalid method", method.to_std_string_escaped()),
            )?;
            // 25. If init["method"] exists, then:
            //     1. Let method be init["method"].
            //     2. If method is not a method or method is a forbidden method, throw a TypeError.
            //     3. Normalize method.
            //     4. Set request's method to method.
            // https://fetch.spec.whatwg.org/#dom-request
            if method.eq_ignore_ascii_case("CONNECT")
                || method.eq_ignore_ascii_case("TRACE")
                || method.eq_ignore_ascii_case("TRACK")
            {
                return Err(js_error!(
                    TypeError: "'{}' HTTP method is unsupported.",
                    method
                ));
            }

            is_get_or_head_method =
                method.eq_ignore_ascii_case("GET") || method.eq_ignore_ascii_case("HEAD");

            builder = builder.method(method.as_str());
        }

        // Fetch Standard §5.4 Request constructor:
        // If either init["body"] exists and is non-null or inputBody is non-null,
        // and request's method is GET or HEAD, then throw a TypeError.
        // https://fetch.spec.whatwg.org/#dom-request
        if is_get_or_head_method
            && (self.body.is_some()
                || inherited_body
                    .as_ref()
                    .is_some_and(|body| !body.is_empty() || !inherited_is_get_or_head_method))
        {
            return Err(js_error!(TypeError: "Request with GET/HEAD method cannot have body."));
        }

        if let Some(body) = &self.body {
            // TODO: add more support types.
            if let Some(body) = body.as_string() {
                let body = body.to_std_string().map_err(
                    |_| js_error!(TypeError: "Request constructor: body is not a valid string"),
                )?;
                request_body = Some(body.into_bytes());
            } else {
                return Err(
                    js_error!(TypeError: "Request constructor: body is not a supported type"),
                );
            }
        } else if let Some(body) = inherited_body {
            request_body = Some(body);
        }

        let request = builder
            .body(request_body.unwrap_or_default())
            .map_err(|_| js_error!(Error: "Cannot construct request"))?;
        Ok(request)
    }
}

/// The JavaScript `Request` class.
///
/// The `Request` interface of the [Fetch API][mdn] represents a resource request.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API
#[derive(Clone, Debug, JsData, Trace, Finalize)]
pub struct JsRequest {
    #[unsafe_ignore_trace]
    inner: HttpRequest<Vec<u8>>,
    signal: Option<JsObject>,
}

impl JsRequest {
    /// Get the inner `http::Request` object. This drops the body (if any).
    pub fn into_inner(mut self) -> HttpRequest<Vec<u8>> {
        mem::replace(&mut self.inner, HttpRequest::new(Vec::new()))
    }

    /// Split this request into its HTTP request and abort signal.
    fn into_parts(mut self) -> (HttpRequest<Vec<u8>>, Option<JsObject>) {
        let request = mem::replace(&mut self.inner, HttpRequest::new(Vec::new()));
        let signal = self.signal.take();
        (request, signal)
    }

    /// Get a reference to the inner `http::Request` object.
    pub fn inner(&self) -> &HttpRequest<Vec<u8>> {
        &self.inner
    }

    /// Get the abort signal associated with this request, if any.
    pub(crate) fn signal(&self) -> Option<JsObject> {
        self.signal.clone()
    }

    /// Get the URI of the request.
    pub fn uri(&self) -> &http::Uri {
        self.inner.uri()
    }

    /// Create a [`JsRequest`] instance from JavaScript arguments, similar to
    /// calling its constructor in JavaScript.
    ///
    /// # Errors
    /// If the URI is invalid, an error is returned.
    pub fn create_from_js(
        input: Either<JsString, JsRequest>,
        options: Option<RequestInit>,
    ) -> JsResult<Self> {
        let (request, signal) = match input {
            Either::Left(uri) => {
                let uri = http::Uri::try_from(
                    uri.to_std_string()
                        .map_err(|_| js_error!(URIError: "URI cannot have unpaired surrogates"))?,
                )
                .map_err(|_| js_error!(URIError: "Invalid URI"))?;
                let request = http::request::Request::builder()
                    .uri(uri)
                    .body(Vec::<u8>::new())
                    .map_err(|_| js_error!(Error: "Cannot construct request"))?;
                (request, None)
            }
            Either::Right(r) => r.into_parts(),
        };

        if let Some(mut options) = options {
            let signal = options.take_signal().or(signal);
            let inner = options.into_request_builder(Some(request))?;
            Ok(Self { inner, signal })
        } else {
            Ok(Self {
                inner: request,
                signal,
            })
        }
    }
}

impl From<HttpRequest<Vec<u8>>> for JsRequest {
    fn from(inner: HttpRequest<Vec<u8>>) -> Self {
        Self {
            inner,
            signal: None,
        }
    }
}

#[boa_class(rename = "Request")]
#[boa(rename_all = "camelCase")]
impl JsRequest {
    /// # Errors
    /// Will return an error if the URL or any underlying error occurred in the
    /// context.
    #[boa(constructor)]
    pub fn constructor(
        input: Either<JsString, JsObject>,
        options: Option<RequestInit>,
    ) -> JsResult<Self> {
        // Need to use a match as `Either::map_right` does not have an equivalent
        // `Either::map_right_ok`.
        let input = match input {
            Either::Right(r) => {
                if let Ok(request) = r.clone().downcast::<JsRequest>() {
                    Either::Right(request.borrow().data().clone())
                } else {
                    return Err(js_error!(TypeError: "invalid input argument"));
                }
            }
            Either::Left(i) => Either::Left(i),
        };
        JsRequest::create_from_js(input, options)
    }

    #[boa(rename = "clone")]
    fn clone_request(&self) -> Self {
        self.clone()
    }
}
