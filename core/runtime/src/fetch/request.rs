//! The `Request` JavaScript class and adjacent types, implemented as [`JsRequest`].
//!
//! See the [Request interface documentation][mdn] for more information.
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request
use super::HttpRequest;
use super::headers::JsHeaders;
use boa_engine::object::builtins::JsPromise;
use boa_engine::value::{Convert, TryFromJs};
use boa_engine::{
    Context, Finalize, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue, Trace,
    boa_class, js_error,
};
use either::Either;
use std::cell::RefCell;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;

/// The body of a [`JsRequest`], which may be either already-read bytes or a
/// pending async future that will produce the bytes on first access.
///
/// Stored behind an `Rc<RefCell<…>>` so that:
/// - multiple clones of the same [`JsRequest`] share one read (the first
///   awaiter stores `Ready`; subsequent callers reuse it), and
/// - body-consuming JS methods (`text`, `json`, `formData`) can be called from
///   async closures that capture the `Rc` by value.
enum BodyState {
    /// Body bytes are already available (constructed synchronously from JS or
    /// produced by a previous call to a body-consuming method).
    Ready(Vec<u8>),
    /// Body bytes have not been read yet; awaiting the future produces them.
    Pending(Pin<Box<dyn Future<Output = Vec<u8>> + 'static>>),
}

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
    /// Returns `true` if a `body` field was explicitly provided in the init object.
    #[must_use]
    pub fn has_body(&self) -> bool {
        self.body.is_some()
    }

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
        let mut request_body = Vec::new();
        if let Some(r) = request {
            let (parts, body) = r.into_parts();
            builder = builder
                .method(parts.method)
                .uri(parts.uri)
                .version(parts.version);

            for (key, value) in &parts.headers {
                builder = builder.header(key, value);
            }
            request_body = body;
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

            builder = builder.method(method.as_str());
        }

        if let Some(body) = &self.body {
            // TODO: add more support types.
            if let Some(body) = body.as_string() {
                let body = body.to_std_string().map_err(
                    |_| js_error!(TypeError: "Request constructor: body is not a valid string"),
                )?;
                request_body = body.into_bytes();
            } else {
                return Err(
                    js_error!(TypeError: "Request constructor: body is not a supported type"),
                );
            }
        }

        builder
            .body(request_body)
            .map_err(|_| js_error!(Error: "Cannot construct request"))
    }
}

/// The JavaScript `Request` class.
///
/// The `Request` interface of the [Fetch API][mdn] represents a resource request.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API
#[derive(Clone, JsData, Trace, Finalize)]
pub struct JsRequest {
    /// Request metadata (method, URI, headers). The body field inside is always
    /// empty (`Vec::new()`); the actual body is stored in `body` below.
    #[unsafe_ignore_trace]
    inner: HttpRequest<Vec<u8>>,
    signal: Option<JsObject>,
    /// The body, which may be lazily awaited on first access.
    ///
    /// Shared via `Rc` so that [`JsRequest::clone_request`] and all
    /// body-consuming methods can access the same underlying data without
    /// duplicating or double-reading it.
    #[unsafe_ignore_trace]
    body: Rc<RefCell<BodyState>>,
}

impl std::fmt::Debug for JsRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsRequest")
            .field("method", &self.inner.method())
            .field("uri", &self.inner.uri())
            .finish_non_exhaustive()
    }
}

impl JsRequest {
    /// Get the inner `http::Request` object.
    ///
    /// If the body is `Ready`, it is included in the returned request.
    /// If the body is still `Pending` (not yet awaited), the returned request
    /// has an empty body — the pending future is dropped.
    pub fn into_inner(self) -> HttpRequest<Vec<u8>> {
        let body_bytes = match &*self.body.borrow() {
            BodyState::Ready(b) => b.clone(),
            BodyState::Pending(_) => Vec::new(),
        };
        let (parts, _) = self.inner.clone().into_parts();
        HttpRequest::from_parts(parts, body_bytes)
    }

    /// Split this request into its HTTP head, abort signal, and body state.
    fn into_parts(
        mut self,
    ) -> (
        HttpRequest<Vec<u8>>,
        Option<JsObject>,
        Rc<RefCell<BodyState>>,
    ) {
        let request = mem::replace(&mut self.inner, HttpRequest::new(Vec::new()));
        let signal = self.signal.take();
        let body = Rc::clone(&self.body);
        (request, signal, body)
    }

    /// Get a reference to the inner `http::Request` object.
    /// Note: the body in the returned request is always empty; use
    /// [`Self::body_bytes`] to access the body.
    pub fn inner(&self) -> &HttpRequest<Vec<u8>> {
        &self.inner
    }

    /// Returns the body bytes when the body is already `Ready`, or `None` if
    /// the body is still `Pending` (not yet resolved from a lazy future).
    pub fn body_bytes(&self) -> Option<Vec<u8>> {
        match &*self.body.borrow() {
            BodyState::Ready(b) => Some(b.clone()),
            BodyState::Pending(_) => None,
        }
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
        // `source_body` carries the body state from an input JsRequest, so that
        // `new Request(existingReq)` preserves a Pending body rather than losing it.
        let (request, signal, source_body) = match input {
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
                (request, None, None)
            }
            Either::Right(r) => {
                let (request, signal, body) = r.into_parts();
                (request, signal, Some(body))
            }
        };

        if let Some(mut options) = options {
            let signal = options.take_signal().or(signal);
            // If options explicitly provides a body, use it; otherwise inherit source_body.
            let has_body = options.has_body();
            let mut inner = options.into_request_builder(Some(request))?;
            let body = if has_body {
                let bytes = mem::take(inner.body_mut());
                Rc::new(RefCell::new(BodyState::Ready(bytes)))
            } else {
                source_body.unwrap_or_else(|| Rc::new(RefCell::new(BodyState::Ready(Vec::new()))))
            };
            return Ok(Self {
                inner,
                signal,
                body,
            });
        }

        // No options: propagate source body or default to empty Ready.
        let body =
            source_body.unwrap_or_else(|| Rc::new(RefCell::new(BodyState::Ready(Vec::new()))));
        Ok(Self {
            inner: request,
            signal,
            body,
        })
    }

    /// Create a [`JsRequest`] whose body is resolved lazily by awaiting
    /// `body_future` on first access.
    ///
    /// Use this when the body is available as an async stream (e.g. an
    /// incoming HTTP request in a WASI component) and you want to avoid
    /// blocking until the body is actually needed by the JS handler.
    pub fn with_lazy_body(
        head: HttpRequest<Vec<u8>>,
        body_future: impl Future<Output = Vec<u8>> + 'static,
    ) -> Self {
        Self {
            inner: head,
            signal: None,
            body: Rc::new(RefCell::new(BodyState::Pending(Box::pin(body_future)))),
        }
    }
}

impl From<HttpRequest<Vec<u8>>> for JsRequest {
    fn from(mut inner: HttpRequest<Vec<u8>>) -> Self {
        // Split the body out of inner so that body bytes live only in `body`.
        let bytes = mem::take(inner.body_mut());
        Self {
            inner,
            signal: None,
            body: Rc::new(RefCell::new(BodyState::Ready(bytes))),
        }
    }
}

/// Helper: resolve the `Rc<RefCell<BodyState>>` to bytes, awaiting the pending
/// future if needed, and caching the result in the `RefCell`.
///
/// This is the shared async core used by `text()`, `json()`, and `formData()`.
async fn resolve_body(body_cell: Rc<RefCell<BodyState>>) -> Vec<u8> {
    // Fast path: body is already ready.
    {
        let guard = body_cell.borrow();
        if let BodyState::Ready(ref bytes) = *guard {
            return bytes.clone();
        }
    }

    // Slow path: take the pending future, await it outside the borrow, then
    // store the result back as `Ready` so subsequent calls are cheap.
    let fut = {
        let mut guard = body_cell.borrow_mut();
        match mem::replace(&mut *guard, BodyState::Ready(Vec::new())) {
            BodyState::Pending(f) => f,
            BodyState::Ready(_) => {
                // Another concurrent caller already resolved it; we just
                // stored an empty Ready above — restore it properly.
                // This branch should be unreachable in practice (single-threaded
                // Boa event loop), but is handled defensively.
                return Vec::new();
            }
        }
    };

    let bytes = fut.await;
    *body_cell.borrow_mut() = BodyState::Ready(bytes.clone());
    bytes
}

// ------ Boa class implementation ------

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

    /// Clones the request.
    ///
    /// The body state is shared via `Rc`: if the body future has not yet been
    /// awaited, the first of the two to call `text()` / `json()` / `formData()`
    /// will await it and cache the result for the other.
    #[boa(rename = "clone")]
    fn clone_request(&self) -> Self {
        self.clone()
    }

    /// Returns the HTTP method of the request.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-request-method>
    #[boa(getter)]
    fn method(&self) -> JsString {
        JsString::from(self.inner.method().as_str())
    }

    /// Returns the URL of the request.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-request-url>
    #[boa(getter)]
    fn url(&self) -> JsString {
        JsString::from(self.inner.uri().to_string().as_str())
    }

    /// Returns the headers associated with the request.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-request-headers>
    #[boa(getter)]
    fn headers(&self) -> JsHeaders {
        JsHeaders::from_http(self.inner.headers().clone())
    }

    /// Reads the request body as a UTF-8 string.
    ///
    /// Returns a `Promise` that resolves to a string.  If the body has not yet
    /// been received from the network it is awaited transparently.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-body-text>
    fn text(&self, context: &mut Context) -> JsPromise {
        let body_cell = Rc::clone(&self.body);
        JsPromise::from_async_fn(
            async move |_| {
                let bytes = resolve_body(body_cell).await;
                let text = String::from_utf8_lossy(&bytes);
                Ok(JsString::from(text.as_ref()).into())
            },
            context,
        )
    }

    /// Reads the request body and parses it as JSON.
    ///
    /// Returns a `Promise` that resolves to the parsed JavaScript value.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-body-json>
    fn json(&self, context: &mut Context) -> JsPromise {
        let body_cell = Rc::clone(&self.body);
        JsPromise::from_async_fn(
            async move |context| {
                let bytes = resolve_body(body_cell).await;
                let json_str = String::from_utf8_lossy(&bytes);
                let json = serde_json::from_str::<serde_json::Value>(&json_str)
                    .map_err(|e| JsNativeError::syntax().with_message(e.to_string()))?;
                JsValue::from_json(&json, &mut context.borrow_mut())
            },
            context,
        )
    }

    /// Reads the request body and parses it as `application/x-www-form-urlencoded`.
    ///
    /// Returns a `Promise` that resolves to a plain JS object with the form
    /// fields.  When the same key appears multiple times the last value wins.
    ///
    /// Only `application/x-www-form-urlencoded` is supported.  Multipart form
    /// data is not supported and will cause the promise to reject with a
    /// `TypeError`.
    ///
    /// See <https://fetch.spec.whatwg.org/#dom-body-formdata>
    fn form_data(&self, context: &mut Context) -> JsPromise {
        let body_cell = Rc::clone(&self.body);
        let content_type = self
            .inner
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);

        JsPromise::from_async_fn(
            async move |context| {
                let is_url_encoded = content_type
                    .as_deref()
                    .is_none_or(|ct| ct.starts_with("application/x-www-form-urlencoded"));

                if !is_url_encoded {
                    return Err(JsNativeError::typ()
                        .with_message(
                            "formData() only supports application/x-www-form-urlencoded bodies",
                        )
                        .into());
                }

                let bytes = resolve_body(body_cell).await;

                let ctx = &mut context.borrow_mut();
                let form_obj = JsObject::default(ctx.intrinsics());

                for (key, value) in form_urlencoded::parse(&bytes) {
                    form_obj.set(
                        JsString::from(key.as_ref()),
                        JsString::from(value.as_ref()),
                        false,
                        ctx,
                    )?;
                }

                Ok(form_obj.into())
            },
            context,
        )
    }
}
