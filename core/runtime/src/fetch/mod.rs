//! Boa's implementation of JavaScript's `fetch` function.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [WHATWG `fetch` specification][spec]
//!
//! [spec]: https://fetch.spec.whatwg.org/
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/fetch

use crate::fetch::headers::JsHeaders;
use crate::fetch::request::{JsRequest, RequestInit};
use crate::fetch::response::JsResponse;
use boa_engine::class::Class;
use boa_engine::object::FunctionObjectBuilder;
use boa_engine::object::builtins::JsArray;
use boa_engine::property::PropertyDescriptor;
use boa_engine::realm::Realm;
use boa_engine::{
    Context, Finalize, JsData, JsError, JsObject, JsResult, JsString, JsSymbol, JsValue,
    NativeObject, Trace, boa_module, js_error, js_string, native_function::NativeFunction,
};
use either::Either;
use http::{HeaderName, HeaderValue, Request as HttpRequest, Request};
use std::cell::RefCell;
use std::rc::Rc;

pub mod headers;
pub mod request;
pub mod response;
pub mod tests;

mod fetchers;

#[doc(inline)]
pub use fetchers::*;

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
    fn resolve_uri(&self, uri: String, _context: &Context) -> JsResult<String> {
        Ok(uri)
    }

    /// Perform the Fetch execution, taking a [`request::JsRequest`] and returning a
    /// [`response::JsResponse`].
    ///
    /// # Errors
    /// Any errors returned by the HTTP implementation must conform to [`JsError`].
    #[expect(async_fn_in_trait, reason = "all our APIs are single-threaded")]
    async fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        context: &RefCell<&Context>,
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
fn get_fetcher<T: Fetcher>(context: &Context) -> JsResult<Rc<T>> {
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
    context: &RefCell<&Context>,
) -> JsResult<JsValue> {
    let fetcher = get_fetcher::<T>(&context.borrow())?;

    // The resource parsing is complicated, so we parse it in Rust here (instead of relying on
    // `TryFromJs` and friends).
    let request: Request<Vec<u8>> = match resource {
        Either::Left(url) => {
            let url = url.to_std_string().map_err(JsError::from_rust)?;
            let url = fetcher
                .resolve_uri(url, &context.borrow())
                .map_err(JsError::from_rust)?;

            let r = HttpRequest::get(url).body(Vec::new());
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

    let mut request = if let Some(options) = options {
        options.into_request_builder(Some(request))?
    } else {
        request
    };

    // Add the `Accept-Language` which should be automatically included, unless specified.
    if !request.headers().contains_key(
        "accept-language"
            .parse::<HeaderName>()
            .map_err(JsError::from_rust)?,
    ) {
        let lang = HeaderValue::from_static("en-US");
        request.headers_mut().append("Accept-Language", lang);
    }

    let response = fetcher.fetch(JsRequest::from(request), context).await?;
    let result = Class::from_data(response, &context.borrow())?;
    Ok(result.into())
}

/// JavaScript module containing the fetch types and functions.
#[boa_module]
pub mod js_module {
    use crate::fetch::request::RequestInit;
    use crate::fetch::{Fetcher, fetch_inner};
    use boa_engine::object::builtins::JsPromise;
    use boa_engine::{Context, JsObject, JsString};
    use either::Either;

    type JsHeaders = super::JsHeaders;
    type JsRequest = super::JsRequest;
    type JsResponse = super::JsResponse;

    /// The `fetch` function.
    ///
    /// # Errors
    /// If the fetcher is not registered in the context, an error is returned.
    /// This function will also return any error that the fetcher returns, or
    /// any conversion to/from JavaScript types.
    pub fn fetch<T: Fetcher>(
        resource: Either<JsString, JsObject>,
        options: Option<RequestInit>,
        context: &Context,
    ) -> JsPromise {
        JsPromise::from_async_fn(
            async move |context| fetch_inner::<T>(resource, options, context).await,
            context,
        )
    }
}

#[doc(inline)]
pub use js_module::fetch;

fn headers_iterator(this: &JsValue, _: &[JsValue], context: &Context) -> JsResult<JsValue> {
    let this_object = this.as_object();
    let headers = this_object
        .as_ref()
        .and_then(JsObject::downcast_ref::<JsHeaders>)
        .ok_or_else(|| {
            js_error!(TypeError: "`Headers.prototype[Symbol.iterator]` requires a `Headers` object")
        })?;

    let entries = headers.entries(context);
    let entries_array = JsArray::from_object(entries.to_object(context)?)?;
    entries_array.values(context)
}

/// Register the `fetch` function in the realm, as well as ALL supporting classes.
/// Pass `None` as the realm to register globally.
///
/// # Errors
/// If any of the classes fail to register, an error is returned.
pub fn register<F: Fetcher>(fetcher: F, realm: Option<Realm>, context: &Context) -> JsResult<()> {
    if let Some(ref realm) = realm {
        realm.host_defined_mut().insert(FetcherRc(Rc::new(fetcher)));
    } else {
        context.insert_data(FetcherRc(Rc::new(fetcher)));
    }
    js_module::boa_register::<F>(realm.clone(), context)?;

    // TODO(#4688): Replace this manual `[Symbol.iterator]` wiring once `#[boa(class)]`
    // supports symbol-named methods.
    let headers_proto = match realm {
        Some(realm) => realm.get_class::<JsHeaders>(),
        None => context.get_global_class::<JsHeaders>(),
    }
    .ok_or_else(|| js_error!(Error: "Headers class should be registered"))?
    .prototype();

    let iterator = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(headers_iterator),
    )
    .name(js_string!("[Symbol.iterator]"))
    .length(0)
    .constructor(false)
    .build();

    headers_proto.define_property_or_throw(
        JsSymbol::iterator(),
        PropertyDescriptor::builder()
            .value(iterator)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    Ok(())
}
