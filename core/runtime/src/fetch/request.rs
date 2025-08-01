//! The `Request` JavaScript class and adjacent types, implemented as [`JsRequest`].
//!
//! See the [Request interface documentation][mdn] for more information.
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request
use super::HttpRequest;
use boa_engine::value::{Convert, TryFromJs};
use boa_engine::{js_error, Finalize, JsData, JsObject, JsResult, JsString, JsValue, Trace};
use boa_interop::boa_macros::boa_class;
use either::Either;
use std::collections::BTreeMap;
use std::mem;

fn add_headers_to_builder<'a>(
    headers: impl Iterator<Item = (&'a JsString, &'a Convert<JsString>)>,
    mut builder: http::request::Builder,
) -> JsResult<http::request::Builder> {
    for (hkey, Convert(hvalue)) in headers {
        // Make sure key and value can be represented by regular strings.
        // Keys also cannot have any extended characters (>128).
        // Values cannot have unpaired surrogates.
        let key = hkey.to_std_string().map_err(|_| {
            js_error!(TypeError: "Request constructor: {} is an invalid header name", hkey.to_std_string_escaped())
        })?;
        if !key.is_ascii() {
            return Err(
                js_error!(TypeError: "Request constructor: {} is an invalid header name", hkey.to_std_string_escaped()),
            );
        }
        let value = hvalue.to_std_string().map_err(|_| {
            js_error!(
                TypeError: "Request constructor: {:?} is an invalid header value",
                hvalue.to_std_string_escaped()
            )
        })?;

        builder = builder.header(key, value);
    }

    Ok(builder)
}

type VecOrMap<K, V> = Either<Vec<(K, V)>, BTreeMap<K, V>>;

/// A [RequestInit][mdn] object. This is a JavaScript object (not a
/// class) that can be used as options for creating a [`JsRequest`].
///
/// [mdn]:https://developer.mozilla.org/en-US/docs/Web/API/RequestInit
// TODO: This class does not contain all fields that are defined in the spec.
#[derive(Debug, Clone, TryFromJs, Trace, Finalize)]
pub struct RequestInit {
    body: Option<JsValue>,
    headers: Option<VecOrMap<JsString, Convert<JsString>>>,
    method: Option<Convert<JsString>>,
}

impl RequestInit {
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
        if let Some(r) = request {
            let (parts, _body) = r.into_parts();
            builder = builder
                .method(parts.method)
                .uri(parts.uri)
                .version(parts.version);

            for (key, value) in &parts.headers {
                builder = builder.header(key, value);
            }
        }

        if let Some(ref headers) = self.headers.take() {
            match headers {
                Either::Left(headers) => {
                    builder = add_headers_to_builder(headers.iter().map(|(k, v)| (k, v)), builder)?;
                }
                Either::Right(headers) => {
                    builder = add_headers_to_builder(headers.iter(), builder)?;
                }
            }
        }

        if let Some(Convert(ref method)) = self.method.take() {
            builder = builder.method(method.to_std_string().map_err(
                |_| js_error!(TypeError: "Request constructor: {} is an invalid method", method.to_std_string_escaped()),
            )?.as_str());
        }

        let mut request_body = None;
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
        }

        builder
            .body(request_body.unwrap_or_default())
            .map_err(|_| js_error!(Error: "Cannot construct request"))
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
}

impl JsRequest {
    /// Get the inner `http::Request` object. This drops the body (if any).
    pub fn into_inner(mut self) -> HttpRequest<Vec<u8>> {
        mem::replace(&mut self.inner, HttpRequest::new(Vec::new()))
    }

    /// Get a reference to the inner `http::Request` object.
    pub fn inner(&self) -> &HttpRequest<Vec<u8>> {
        &self.inner
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
        let request = match input {
            Either::Left(uri) => {
                let uri = http::Uri::try_from(
                    uri.to_std_string()
                        .map_err(|_| js_error!(URIError: "URI cannot have unpaired surrogates"))?,
                )
                .map_err(|_| js_error!(URIError: "Invalid URI"))?;
                http::request::Request::builder()
                    .uri(uri)
                    .body(Vec::<u8>::new())
                    .map_err(|_| js_error!(Error: "Cannot construct request"))?
            }
            Either::Right(r) => r.into_inner(),
        };

        if let Some(options) = options {
            let inner = options.into_request_builder(Some(request))?;
            Ok(Self { inner })
        } else {
            Ok(Self { inner: request })
        }
    }
}

impl From<HttpRequest<Vec<u8>>> for JsRequest {
    fn from(inner: HttpRequest<Vec<u8>>) -> Self {
        Self { inner }
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
}
