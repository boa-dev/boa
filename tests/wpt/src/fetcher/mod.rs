use boa_engine::{Context, Finalize, JsData, JsError, JsResult, Trace};
use boa_runtime::fetch::Fetcher;
use http::{HeaderName, HeaderValue, Response};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Trace, Finalize, JsData)]
pub struct WptFetcher {
    wpt_root: PathBuf,

    stash: BTreeMap<String, Vec<u8>>,
}

impl WptFetcher {
    pub fn new(wpt_root: impl Into<PathBuf>) -> Self {
        Self {
            wpt_root: wpt_root.into(),
            stash: BTreeMap::new(),
        }
    }

    fn fetch_inspect_header(
        &self,
        request: http::request::Request<Option<Vec<u8>>>,
    ) -> JsResult<http::response::Response<Option<Vec<u8>>>> {
        let mut response = http::response::Response::builder();

        let url = Url::parse(&request.uri().to_string()).unwrap();
        let pairs = url.query_pairs();

        let pairs = pairs.collect::<BTreeMap<Cow<str>, Cow<str>>>();
        if let Some(headers) = pairs.get("headers") {
            for header in headers.split('|') {
                let resp_header_name = format!("x-request-{header}");
                let resp_header_value = request
                    .headers()
                    .get(header)
                    .map(|v| v.to_str().unwrap())
                    .unwrap_or("");
                eprintln!("header  {} => {}", resp_header_name, resp_header_value);
                response.headers_mut().unwrap().insert(
                    HeaderName::from_str(resp_header_name.as_str()).unwrap(),
                    HeaderValue::from_str(resp_header_value).unwrap(),
                );
            }
        }

        response.body(None).map_err(JsError::from_rust)
    }
}

impl Fetcher for WptFetcher {
    fn fetch_blocking(
        &self,
        request: http::request::Request<Option<Vec<u8>>>,
        _context: &mut Context,
    ) -> JsResult<http::response::Response<Option<Vec<u8>>>> {
        eprintln!("url: {}", request.uri());
        eprintln!("req {request:?}");
        match request.uri().path() {
            "/fetch/resources/inspect-headers.py" => self.fetch_inspect_header(request),
            _ => Ok(Response::builder().status(401).body(None).unwrap()),
        }
    }
}
