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
    Context, Finalize, JsData, JsError, JsObject, JsResult, JsString, JsValue, NativeObject, Trace,
    js_error, js_string,
};
use boa_interop::IntoJsFunctionCopied;
use either::Either;
use http::{Request as HttpRequest, Request};
use std::cell::RefCell;
use std::rc::Rc;

pub mod headers;
pub mod request;
pub mod response;

pub mod fetchers;

/// A trait for backend implementation of an HTTP fetcher.
// TODO: consider implementing an async version of this.
pub trait Fetcher: NativeObject {
    /// Resolve a string to a URL. This is used when a string (e.g., the first argument to
    /// `fetch()`) is passed, and we need resolution. Some cases require resolution of
    /// a relative path, for example (to the "page" base URL).
    /// By default, this will return the `URI` as is.
    ///
    /// # Errors
    /// This function should return an error if the URL cannot be handled by the [`Fetcher`].
    fn resolve_uri(&self, uri: String, _context: &mut Context) -> JsResult<String> {
        Ok(uri)
    }

    /// Perform the Fetch execution, taking a [`request::JsRequest`] and returning a
    /// [`response::JsResponse`].
    ///
    /// # Errors
    /// Any errors returned by the HTTP implementation must conform to [`JsError`].
    async fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        context: &RefCell<&mut Context>,
    ) -> JsResult<JsResponse>;
}

/// A reference counted pointer to a `Fetcher` implementation. This is so we can
/// add this to the context, but we need to be able to keep an `Rc<>` structure
/// to make API calls.
#[derive(Debug, Trace, Finalize, JsData)]
struct FetcherRc<T: Fetcher>(#[unsafe_ignore_trace] pub Rc<T>);

impl<T: Fetcher> Clone for FetcherRc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Get a Fetcher instance from the context.
fn get_fetcher<T: Fetcher>(context: &mut Context) -> JsResult<Rc<T>> {
    // Try fetching from the context first, then the current realm. Else fail.
    let Some(fetcher) = context.get_data::<FetcherRc<T>>().cloned().or_else(|| {
        context
            .realm()
            .host_defined()
            .get::<FetcherRc<T>>()
            .cloned()
    }) else {
        return Err(
            js_error!(Error: "Implementation of fetch requires a fetcher registered in the context"),
        );
    };

    Ok(fetcher.0.clone())
}

/// The `fetch` function internals.
async fn fetch_inner<T: Fetcher>(
    resource: Either<JsString, JsObject>,
    options: Option<RequestInit>,
    context: &RefCell<&mut Context>,
) -> JsResult<JsValue> {
    let fetcher = get_fetcher::<T>(&mut context.borrow_mut())?;

    // The resource parsing is complicated, so we parse it in Rust here (instead of relying on
    // `TryFromJs` and friends).
    let request: Request<Option<Vec<u8>>> = match resource {
        Either::Left(url) => {
            let url = url.to_std_string().map_err(JsError::from_rust)?;
            let url = fetcher
                .resolve_uri(url, &mut context.borrow_mut())
                .map_err(JsError::from_rust)?;

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
    let response = fetcher.fetch(JsRequest::from(request), context).await?;
    let result = Class::from_data(response, &mut context.borrow_mut())?;
    Ok(result.into())
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
) -> JsPromise {
    let context = RefCell::new(context);
    let future = async move { fetch_inner::<T>(resource, options, &context).await };
    JsPromise::from_future(future, &mut context.borrow_mut())
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
        realm.host_defined_mut().insert(FetcherRc(Rc::new(fetcher)));

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

        context.insert_data(FetcherRc(Rc::new(fetcher)));
        context.register_global_property(js_string!("fetch"), fetch_fn, Attribute::all())?;
    }

    Ok(())
}
