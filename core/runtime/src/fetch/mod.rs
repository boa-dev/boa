//! Boa's implementation of JavaScript's `fetch` function.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [WHATWG `fetch` specification][spec]
//!
//! [spec]: https://fetch.spec.whatwg.org/
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/fetch

pub mod tests;

use crate::fetch::headers::JsHeaders;
use crate::fetch::request::{JsRequest, RequestInit};
use crate::fetch::response::JsResponse;
use boa_engine::class::{Class, ClassBuilder};
use boa_engine::object::builtins::JsPromise;
use boa_engine::property::Attribute;
use boa_engine::realm::Realm;
use boa_engine::{
    js_error, js_string, Context, JsError, JsObject, JsResult, JsString, NativeObject,
};
use boa_gc::Gc;
use boa_interop::IntoJsFunctionCopied;
use either::Either;
use http::{Request as HttpRequest, Request, Response as HttpResponse};

pub mod headers;
pub mod request;
pub mod response;

pub mod fetchers;

/// A trait for backend implementation of an HTTP fetcher.
// TODO: consider implementing an async version of this.
pub trait Fetcher: NativeObject + Sized {
    /// Resolve a URI to a URL. URIs can be any strings, but to do an `HttpRequest`
    /// we need a proper URL.
    /// By default, this will return the `URI` as is.
    fn resolve_uri(&self, uri: &http::Uri) -> String {
        uri.to_string()
    }

    /// Fetch an HTTP document, returning an HTTP response.
    ///
    /// # Errors
    /// Any errors returned by the HTTP implementation must conform to
    /// [`JsError`].
    fn fetch_blocking(
        &self,
        request: HttpRequest<Option<Vec<u8>>>,
        context: &mut Context,
    ) -> JsResult<HttpResponse<Option<Vec<u8>>>>;
}

/// The `fetch` function.
///
/// A [`Gc`]<[`Fetcher`]> implementation MUST be inserted in the [`Context`] (or
/// [`Realm`] if you're using multiple contexts) before calling this function.
///
/// # Errors
/// If the fetcher is not registered in the context, an error is returned.
/// This function will also return any error that the fetcher returns, or
/// any conversion to/from JavaScript types.
pub fn fetch<T: Fetcher>(
    resource: Either<JsString, JsObject>,
    options: Option<RequestInit>,
    context: &mut Context,
) -> JsResult<JsPromise> {
    // Try fetching from the context first, then the current realm. Else fail.
    let Some(fetcher) = context
        .get_data::<Gc<T>>()
        .cloned()
        .or_else(|| context.realm().host_defined().get::<Gc<T>>().cloned())
    else {
        return Err(
            js_error!(Error: "Implementation of fetch requires a fetcher registered in the context"),
        );
    };

    // The resource parsing is complicated, so we parse it in Rust here (instead of relying on
    // `TryFromJs` and friends).
    let request: Request<Option<Vec<u8>>> = match resource {
        Either::Left(url) => {
            let url = url.to_std_string().map_err(JsError::from_rust)?;

            let r = HttpRequest::get(url).body(Some(Vec::new()));
            r.map_err(JsError::from_rust)?
        }
        Either::Right(request) => {
            // This can be a [`JsRequest`] object.
            let Ok(request) = request.downcast::<JsRequest>() else {
                return Err(js_error!(TypeError: "Resource must be a URL or Request object"));
            };
            let Ok(request_ref) = request.try_borrow() else {
                return Err(js_error!(TypeError: "Request object is already in use"));
            };

            request_ref.data().inner().clone()
        }
    };

    let request = if let Some(options) = options {
        options.into_request_builder(Some(request))?
    } else {
        request
    };
    let url = JsString::from(request.uri().to_string());

    let response = fetcher.fetch_blocking(request, context)?;

    let result = Class::from_data(JsResponse::new(url, response), context)?;
    Ok(JsPromise::resolve(result, context))
}

/// Register the `fetch` function in the realm, as well as ALL supporting classes.
/// Pass `None` as the realm to register globally.
///
/// # Errors
/// If any of the classes fail to register, an error is returned.
pub fn register<F: Fetcher>(
    fetcher: F,
    realm: Option<Realm>,
    context: &mut Context,
) -> JsResult<()> {
    if let Some(realm) = realm {
        realm.host_defined_mut().insert(Gc::new(fetcher));

        let mut class_builder = ClassBuilder::new::<JsHeaders>(context);
        JsHeaders::init(&mut class_builder)?;
        let class = class_builder.build();
        realm.register_class::<JsHeaders>(class);

        let mut class_builder = ClassBuilder::new::<JsRequest>(context);
        JsRequest::init(&mut class_builder)?;
        let class = class_builder.build();
        realm.register_class::<JsRequest>(class);

        let mut class_builder = ClassBuilder::new::<JsResponse>(context);
        JsResponse::init(&mut class_builder)?;
        let class = class_builder.build();
        realm.register_class::<JsResponse>(class);
    } else {
        context.register_global_class::<JsHeaders>()?;
        context.register_global_class::<JsRequest>()?;
        context.register_global_class::<JsResponse>()?;

        let fetch_fn = fetch::<F>
            .into_js_function_copied(context)
            .to_js_function(&realm.unwrap_or_else(|| context.realm().clone()));

        context.insert_data(Gc::new(fetcher));
        context.register_global_property(js_string!("fetch"), fetch_fn, Attribute::all())?;
    }

    Ok(())
}
