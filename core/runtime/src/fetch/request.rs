//! The `Request` JavaScript class and adjacent types, implemented as [`JsRequest`].
//!
//! See the [Request interface documentation][mdn] for more information.
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request
use super::HttpRequest;
use boa_engine::value::{Convert, TryFromJs};
use boa_engine::{
    Finalize, JsData, JsObject, JsResult, JsString, JsValue, Trace, boa_class, js_error,
};
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

/// The [mode][mdn] for a `Request`.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/mode
#[derive(Debug, Clone, Trace, Finalize)]
pub enum RequestMode {
    /// Navigation request.
    Navigate,
    /// Same-origin request.
    SameOrigin,
    /// No CORS check.
    NoCors,
    /// CORS-enabled request.
    Cors,
}

impl TryFromJs for RequestMode {
    fn try_from_js(value: &JsValue, context: &mut boa_engine::Context) -> JsResult<Self> {
        let s = value.to_string(context)?;
        match s.to_std_string_escaped().as_str() {
            "navigate" => Ok(Self::Navigate),
            "same-origin" => Ok(Self::SameOrigin),
            "no-cors" => Ok(Self::NoCors),
            "cors" => Ok(Self::Cors),
            other => Err(js_error!(
                TypeError: "Request constructor: mode '{}' is not a supported value",
                other
            )),
        }
    }
}

/// The [credentials][mdn] mode for a `Request`.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/credentials
#[derive(Debug, Clone, Trace, Finalize)]
pub enum RequestCredentials {
    /// Never send or receive cookies.
    Omit,
    /// Send credentials only if the URL is on the same origin.
    SameOrigin,
    /// Always send credentials.
    Include,
}

impl TryFromJs for RequestCredentials {
    fn try_from_js(value: &JsValue, context: &mut boa_engine::Context) -> JsResult<Self> {
        let s = value.to_string(context)?;
        match s.to_std_string_escaped().as_str() {
            "omit" => Ok(Self::Omit),
            "same-origin" => Ok(Self::SameOrigin),
            "include" => Ok(Self::Include),
            other => Err(js_error!(
                TypeError: "Request constructor: credentials '{}' is not a supported value",
                other
            )),
        }
    }
}

/// The [cache][mdn] mode for a `Request`.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/cache
#[derive(Debug, Clone, Trace, Finalize)]
pub enum RequestCache {
    /// The browser looks for a matching request in its HTTP cache.
    Default,
    /// The browser fetches the resource from the remote server without first
    /// looking in the cache, and will not update the cache with the response.
    NoStore,
    /// The browser fetches the resource from the remote server without first
    /// looking in the cache, but then will update the cache with the response.
    Reload,
    /// The browser looks for a matching request in its HTTP cache; if found,
    /// the browser revalidates the response.
    NoCache,
    /// The browser looks for a matching request in its HTTP cache; if found,
    /// returns the cached response even if stale.
    ForceCache,
    /// The browser looks for a matching request in its HTTP cache; if found,
    /// returns it. Otherwise returns a `504 Gateway Timeout`.
    OnlyIfCached,
}

impl TryFromJs for RequestCache {
    fn try_from_js(value: &JsValue, context: &mut boa_engine::Context) -> JsResult<Self> {
        let s = value.to_string(context)?;
        match s.to_std_string_escaped().as_str() {
            "default" => Ok(Self::Default),
            "no-store" => Ok(Self::NoStore),
            "reload" => Ok(Self::Reload),
            "no-cache" => Ok(Self::NoCache),
            "force-cache" => Ok(Self::ForceCache),
            "only-if-cached" => Ok(Self::OnlyIfCached),
            other => Err(js_error!(
                TypeError: "Request constructor: cache '{}' is not a supported value",
                other
            )),
        }
    }
}

/// The [redirect][mdn] mode for a `Request`.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/redirect
#[derive(Debug, Clone, Trace, Finalize)]
pub enum RequestRedirect {
    /// Automatically follow redirects.
    Follow,
    /// Abort with an error if a redirect occurs.
    Error,
    /// Return a filtered response whose type is `opaqueredirect`.
    Manual,
}

impl TryFromJs for RequestRedirect {
    fn try_from_js(value: &JsValue, context: &mut boa_engine::Context) -> JsResult<Self> {
        let s = value.to_string(context)?;
        match s.to_std_string_escaped().as_str() {
            "follow" => Ok(Self::Follow),
            "error" => Ok(Self::Error),
            "manual" => Ok(Self::Manual),
            other => Err(js_error!(
                TypeError: "Request constructor: redirect '{}' is not a supported value",
                other
            )),
        }
    }
}

/// The [referrer policy][mdn] for a `Request`.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/referrerPolicy
#[derive(Debug, Clone, Trace, Finalize)]
pub enum ReferrerPolicy {
    /// No policy (empty string).
    Empty,
    /// No referrer information sent.
    NoReferrer,
    /// Send the referrer for same-protocol destinations (HTTPS→HTTPS).
    NoReferrerWhenDowngrade,
    /// Only send referrer for same-origin requests.
    SameOrigin,
    /// Only send the origin as the referrer.
    Origin,
    /// Send origin for cross-origin; full URL for same-origin.
    StrictOrigin,
    /// Send origin for cross-origin; full URL for same-origin.
    OriginWhenCrossOrigin,
    /// Same as origin-when-cross-origin but only for same-protocol.
    StrictOriginWhenCrossOrigin,
    /// Always send the full URL as the referrer.
    UnsafeUrl,
}

impl TryFromJs for ReferrerPolicy {
    fn try_from_js(value: &JsValue, context: &mut boa_engine::Context) -> JsResult<Self> {
        let s = value.to_string(context)?;
        match s.to_std_string_escaped().as_str() {
            "" => Ok(Self::Empty),
            "no-referrer" => Ok(Self::NoReferrer),
            "no-referrer-when-downgrade" => Ok(Self::NoReferrerWhenDowngrade),
            "same-origin" => Ok(Self::SameOrigin),
            "origin" => Ok(Self::Origin),
            "strict-origin" => Ok(Self::StrictOrigin),
            "origin-when-cross-origin" => Ok(Self::OriginWhenCrossOrigin),
            "strict-origin-when-cross-origin" => Ok(Self::StrictOriginWhenCrossOrigin),
            "unsafe-url" => Ok(Self::UnsafeUrl),
            other => Err(js_error!(
                TypeError: "Request constructor: referrerPolicy '{}' is not a supported value",
                other
            )),
        }
    }
}

type VecOrMap<K, V> = Either<Vec<(K, V)>, BTreeMap<K, V>>;

/// A [RequestInit][mdn] object. This is a JavaScript object (not a
/// class) that can be used as options for creating a [`JsRequest`].
///
/// [mdn]:https://developer.mozilla.org/en-US/docs/Web/API/RequestInit
// NOTE: This does not yet contain *all* fields from the spec, but it
// now supports several of the most common ones.
#[derive(Debug, Clone, TryFromJs, Trace, Finalize)]
pub struct RequestInit {
    body: Option<JsValue>,
    headers: Option<VecOrMap<JsString, Convert<JsString>>>,
    method: Option<Convert<JsString>>,

    // Additional RequestInit fields from the Fetch spec. These are
    // parsed and validated via their enum TryFromJs implementations,
    // but most are not yet wired through to the underlying HTTP client.
    mode: Option<RequestMode>,
    credentials: Option<RequestCredentials>,
    cache: Option<RequestCache>,
    redirect: Option<RequestRedirect>,
    referrer: Option<Convert<JsString>>,
    referrer_policy: Option<ReferrerPolicy>,
    integrity: Option<Convert<JsString>>,
    keepalive: Option<bool>,
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

        // The enum fields (mode, credentials, cache, redirect,
        // referrer_policy) are already validated by their TryFromJs
        // implementations. They are accepted here but treated as no-ops
        // until the runtime wires them to the underlying HTTP client.
        drop(self.mode.take());
        drop(self.credentials.take());
        drop(self.cache.take());
        drop(self.redirect.take());
        drop(self.referrer_policy.take());

        // `referrer`, `integrity` and `keepalive` are accepted for now but
        // treated as no-ops until the runtime makes use of them.
        drop(self.referrer.take());
        drop(self.integrity.take());
        let _ = self.keepalive.take();

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
