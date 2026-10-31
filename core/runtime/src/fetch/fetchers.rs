//! Module containing various implementations of the [`Fetcher`] trait.

use crate::fetch::Fetcher;
use crate::fetch::request::JsRequest;
use crate::fetch::response::JsResponse;
use boa_engine::{Context, Finalize, JsData, JsObject, JsResult, Trace, js_error};
use std::cell::RefCell;
use std::future;
use std::rc::Rc;

/// Implementation of `Fetcher` which will always reject any fetch.
#[derive(Clone, Debug, Trace, Finalize, JsData)]
pub struct ErrorFetcher;

impl Fetcher for ErrorFetcher {
    fn fetch(
        self: Rc<Self>,
        _request: JsRequest,
        _signal: Option<JsObject>,
        _context: &RefCell<&mut Context>,
    ) -> impl Future<Output = JsResult<JsResponse>> {
        future::ready(Err(
            js_error!(ReferenceError: "ErrorFetcher used in fetch API."),
        ))
    }
}

/// Implementation of `Fetcher` that uses the blocking `reqwest` library as the backend.
#[cfg(feature = "reqwest-blocking")]
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct BlockingReqwestFetcher {
    #[unsafe_ignore_trace]
    client: reqwest::blocking::Client,
    #[unsafe_ignore_trace]
    url_list: std::sync::Arc<std::sync::Mutex<Vec<url::Url>>>,
}

#[cfg(feature = "reqwest-blocking")]
impl Default for BlockingReqwestFetcher {
    fn default() -> Self {
        use std::sync::{Arc, Mutex};

        let url_list = Arc::new(Mutex::new(Vec::new()));
        let url_list_clone = Arc::clone(&url_list);

        // Record each redirect target URL so `Response.url` and `Response.redirected`
        // reflect the full WHATWG Fetch response URL list (https://github.com/boa-dev/boa/issues/5310).
        let custom_policy = reqwest::redirect::Policy::custom(move |attempt| {
            url_list_clone
                .lock()
                .expect("should not be poisoned")
                .push(attempt.url().clone());
            reqwest::redirect::Policy::default().redirect(attempt)
        });

        let client = reqwest::blocking::Client::builder()
            .redirect(custom_policy)
            .build()
            .unwrap_or_default();

        Self { client, url_list }
    }
}

#[cfg(feature = "reqwest-blocking")]
impl Fetcher for BlockingReqwestFetcher {
    fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        signal: Option<JsObject>,
        _context: &RefCell<&mut Context>,
    ) -> impl Future<Output = JsResult<JsResponse>> {
        use boa_engine::{JsError, JsString};

        if let Some(ref sig) = signal
            && let Some(sig_ref) = sig.downcast_ref::<crate::abort::JsAbortSignal>()
            && sig_ref.is_aborted()
        {
            return future::ready(Err(JsError::from_opaque(
                boa_engine::js_string!("AbortError").into(),
            )));
        }

        let request = request.into_inner();
        let url = request.uri().to_string();
        let mut req = self
            .client
            .request(request.method().clone(), &url)
            .headers(request.headers().clone());
        if let Some(body) = request.body().clone() {
            req = req.body(body);
        }

        let req = match req.build().map_err(JsError::from_rust) {
            Ok(req) => req,
            Err(err) => return future::ready(Err(err)),
        };

        self.url_list
            .lock()
            .expect("should not be poisoned")
            .clear();

        let result = self.client.execute(req);
        let redirected_urls =
            std::mem::take(&mut *self.url_list.lock().expect("should not be poisoned"));

        let resp = match result.map_err(JsError::from_rust) {
            Ok(resp) => resp,
            Err(err) => return future::ready(Err(err)),
        };

        if let Some(ref sig) = signal
            && let Some(sig_ref) = sig.downcast_ref::<crate::abort::JsAbortSignal>()
            && sig_ref.is_aborted()
        {
            return future::ready(Err(JsError::from_opaque(
                boa_engine::js_string!("AbortError").into(),
            )));
        }

        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = match resp.bytes().map_err(JsError::from_rust) {
            Ok(bytes) => bytes,
            Err(err) => return future::ready(Err(err)),
        };
        let mut builder = http::Response::builder().status(status.as_u16());

        for k in headers.keys() {
            for v in headers.get_all(k) {
                builder = builder.header(k.as_str(), v);
            }
        }

        let mut url_list = Vec::with_capacity(1 + redirected_urls.len());
        url_list.push(JsString::from(url));
        url_list.extend(
            redirected_urls
                .into_iter()
                .map(|u| JsString::from(u.as_str())),
        );

        future::ready(
            builder
                .body(bytes.to_vec())
                .map_err(JsError::from_rust)
                .map(|request| JsResponse::basic_with_url_list(url_list, request)),
        )
    }
}
