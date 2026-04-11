//! The `Request` JavaScript class and adjacent types, implemented as [`JsRequest`].
//!
//! See the [Request interface documentation][mdn] for more information.
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request
use super::HttpRequest;
use super::body;
use super::headers::JsHeaders;
use boa_engine::object::builtins::JsPromise;
use boa_engine::value::{Convert, TryFromJs};
use boa_engine::{
    Context, Finalize, JsData, JsObject, JsResult, JsString, JsValue, Trace, boa_class, js_error,
};
use either::Either;
use std::cell::Cell;
use std::mem;
use std::rc::Rc;

/// A [RequestInit][mdn] object. This is a JavaScript object (not a
/// class) that can be used as options for creating a [`JsRequest`].
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/RequestInit
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

    pub(crate) fn has_body(&self) -> bool {
        self.body.is_some()
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
#[derive(Debug, JsData, Trace, Finalize)]
pub struct JsRequest {
    #[unsafe_ignore_trace]
    inner: HttpRequest<Vec<u8>>,
    signal: Option<JsObject>,
    #[unsafe_ignore_trace]
    has_body: bool,
    #[unsafe_ignore_trace]
    body_used: Cell<bool>,
}

impl Clone for JsRequest {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            signal: self.signal.clone(),
            has_body: self.has_body,
            body_used: Cell::new(self.body_used.get()),
        }
    }
}

impl JsRequest {
    fn new(inner: HttpRequest<Vec<u8>>, signal: Option<JsObject>, has_body: bool) -> Self {
        Self {
            inner,
            signal,
            has_body,
            body_used: Cell::new(false),
        }
    }

    /// Get the inner `http::Request` object. This drops the body (if any).
    pub fn into_inner(mut self) -> HttpRequest<Vec<u8>> {
        mem::replace(&mut self.inner, HttpRequest::new(Vec::new()))
    }

    fn into_parts(mut self) -> (HttpRequest<Vec<u8>>, Option<JsObject>, bool) {
        let request = mem::replace(&mut self.inner, HttpRequest::new(Vec::new()));
        let signal = self.signal.take();
        (request, signal, self.has_body)
    }

    /// Get a reference to the inner `http::Request` object.
    pub fn inner(&self) -> &HttpRequest<Vec<u8>> {
        &self.inner
    }

    pub(crate) fn signal(&self) -> Option<JsObject> {
        self.signal.clone()
    }

    pub(crate) fn has_body(&self) -> bool {
        self.has_body
    }

    pub(crate) fn ensure_body_unused(&self) -> JsResult<()> {
        if self.is_body_used() {
            return Err(js_error!(TypeError: "Body has already been used"));
        }
        Ok(())
    }

    pub(crate) fn mark_body_used(&self) {
        if self.has_body {
            self.body_used.set(true);
        }
    }

    fn consume_body(&self) -> JsResult<Rc<Vec<u8>>> {
        self.ensure_body_unused()?;
        self.mark_body_used();
        Ok(Rc::new(self.inner.body().clone()))
    }

    fn is_body_used(&self) -> bool {
        self.has_body && self.body_used.get()
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
        let body_overridden = options.as_ref().is_some_and(RequestInit::has_body);

        let (request, signal, has_body) = match input {
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
                (request, None, false)
            }
            Either::Right(r) => {
                if !body_overridden {
                    r.ensure_body_unused()?;
                }
                r.into_parts()
            }
        };

        if let Some(mut options) = options {
            let signal = options.take_signal().or(signal);
            let inner = options.into_request_builder(Some(request))?;
            Ok(Self::new(inner, signal, body_overridden || has_body))
        } else {
            Ok(Self::new(request, signal, has_body))
        }
    }
}

impl From<HttpRequest<Vec<u8>>> for JsRequest {
    fn from(inner: HttpRequest<Vec<u8>>) -> Self {
        let has_body = !inner.body().is_empty();
        Self::new(inner, None, has_body)
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
        let body_overridden = options.as_ref().is_some_and(RequestInit::has_body);
        let mut source_request = None;
        let input = match input {
            Either::Right(r) => {
                if let Ok(request_obj) = r.clone().downcast::<JsRequest>() {
                    {
                        let request_ref = request_obj.borrow();
                        let request = request_ref.data();
                        if !body_overridden {
                            request.ensure_body_unused()?;
                        }
                        source_request = Some(request_obj.clone());
                    }

                    let request = request_obj.borrow();
                    Either::Right(request.data().clone())
                } else {
                    return Err(js_error!(TypeError: "invalid input argument"));
                }
            }
            Either::Left(i) => Either::Left(i),
        };
        let request = JsRequest::create_from_js(input, options)?;

        if !body_overridden && let Some(source_request) = source_request {
            source_request.borrow().data().mark_body_used();
        }

        Ok(request)
    }

    #[boa(getter)]
    fn body_used(&self) -> bool {
        self.is_body_used()
    }

    #[boa(rename = "clone")]
    fn clone_request(&self) -> JsResult<Self> {
        self.ensure_body_unused()?;
        Ok(self.clone())
    }

    fn bytes(&self, context: &mut Context) -> JsResult<JsPromise> {
        Ok(body::bytes(self.consume_body()?, context))
    }

    fn text(&self, context: &mut Context) -> JsResult<JsPromise> {
        Ok(body::text(self.consume_body()?, context))
    }

    fn json(&self, context: &mut Context) -> JsResult<JsPromise> {
        Ok(body::json(self.consume_body()?, context))
    }
}
