//! Module containing various implementations of the [`Fetcher`] trait.

use crate::fetch::request::JsRequest;
use crate::fetch::response::JsResponse;
use crate::fetch::Fetcher;
use boa_engine::{js_error, Context, Finalize, JsData, JsError, JsResult, JsString, Trace};
use std::cell::RefCell;
use std::rc::Rc;

/// Implementation of `Fetcher` which will always reject any fetch.
#[derive(Clone, Debug, Trace, Finalize, JsData)]
pub struct ErrorFetcher;

impl Fetcher for ErrorFetcher {
    async fn fetch(
        self: Rc<Self>,
        _request: JsRequest,
        _context: &RefCell<&mut Context>,
    ) -> JsResult<JsResponse> {
        Err(js_error!(ReferenceError: "ErrorFetcher used in fetch API."))
    }
}

/// Implementation of `Fetcher` that uses the blocking `reqwest` library as the backend.
#[cfg(feature = "reqwest")]
#[derive(Default, Debug, Clone, Trace, Finalize, JsData)]
pub struct BlockingReqwestFetcher {
    #[unsafe_ignore_trace]
    client: reqwest::blocking::Client,
}

#[cfg(feature = "reqwest")]
impl Fetcher for BlockingReqwestFetcher {
    async fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        _context: &RefCell<&mut Context>,
    ) -> JsResult<JsResponse> {
        let request = request.into_inner();
        let url = request.uri().to_string();
        let req = self
            .client
            .request(request.method().clone(), &url)
            .headers(request.headers().clone());

        let req = if let Some(body) = request.body().clone() {
            req.body(body).build()
        } else {
            req.build()
        }
        .map_err(JsError::from_rust)?;

        let resp = self.client.execute(req).map_err(JsError::from_rust)?;

        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = resp.bytes().map_err(JsError::from_rust)?;
        let mut builder = http::Response::builder().status(status.as_u16());

        for k in headers.keys() {
            for v in headers.get_all(k) {
                builder = builder.header(k.as_str(), v);
            }
        }

        builder
            .body(bytes.is_empty().then(|| bytes.to_vec()))
            .map_err(JsError::from_rust)
            .map(|request| JsResponse::basic(JsString::from(url), request))
    }
}
