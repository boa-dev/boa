//! Module containing various implementations of the [`super::Fetcher`] trait.
use crate::fetch::Fetcher;
use boa_engine::{js_error, Context, Finalize, JsData, JsError, JsResult, Trace};

/// Implementation of `Fetcher` which will always return an error.
#[derive(Clone, Debug, Trace, Finalize, JsData)]
pub struct ErrorFetcher;

impl Fetcher for ErrorFetcher {
    fn fetch_blocking(
        &self,
        _request: http::Request<Option<Vec<u8>>>,
        _context: &mut Context,
    ) -> JsResult<http::Response<Option<Vec<u8>>>> {
        Err(js_error!(ReferenceError: "Invalid Fetcher used in fetch API."))
    }
}

/// Implementation of `Fetcher` that uses `reqwest` as the backend.
#[cfg(feature = "reqwest")]
#[derive(Default, Debug, Clone, Trace, Finalize, JsData)]
pub struct ReqwestFetcher {
    #[unsafe_ignore_trace]
    client: reqwest::blocking::Client,
}

#[cfg(feature = "reqwest")]
impl Fetcher for ReqwestFetcher {
    fn fetch_blocking(
        &self,
        request: http::Request<Option<Vec<u8>>>,
        _context: &mut Context,
    ) -> JsResult<http::Response<Option<Vec<u8>>>> {
        let req = self
            .client
            .request(request.method().clone(), request.uri().to_string())
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
    }
}
