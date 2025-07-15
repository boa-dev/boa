use boa_engine::{js_error, Context, Finalize, JsData, JsError, JsResult, Trace};
use boa_gc::{Gc, GcRefCell};
use boa_runtime::fetch::Fetcher;
use http::{HeaderName, HeaderValue};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Trace, Finalize, JsData)]
struct WptFetcherInner {
    wpt_root: PathBuf,
    current_file: Option<PathBuf>,

    stash: BTreeMap<String, Vec<u8>>,
}

impl WptFetcherInner {
    pub fn new(wpt_root: impl Into<PathBuf>) -> Self {
        Self {
            wpt_root: wpt_root.into(),
            current_file: None,
            stash: BTreeMap::new(),
        }
    }

    fn fetch_inspect_header(
        &self,
        request: http::Request<Option<Vec<u8>>>,
    ) -> JsResult<http::Response<Option<Vec<u8>>>> {
        let mut response = http::Response::builder();

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
                response.headers_mut().unwrap().insert(
                    HeaderName::from_str(resp_header_name.as_str()).unwrap(),
                    HeaderValue::from_str(resp_header_value).unwrap(),
                );
            }
        }

        response.body(None).map_err(JsError::from_rust)
    }

    pub fn parse_headers(
        &self,
        request: http::Request<Option<Vec<u8>>>,
    ) -> JsResult<http::Response<Option<Vec<u8>>>> {
        let mut response = http::Response::builder();

        let url = Url::parse(&request.uri().to_string()).unwrap();
        let pairs = url.query_pairs();

        let pairs = pairs.collect::<BTreeMap<Cow<str>, Cow<str>>>();
        if let Some(header) = pairs.get("my-custom-header") {
            let value = HeaderValue::from_str(header)
                .map_err(|_| js_error!(TypeError: "failed to parse header value"))?;

            response
                .headers_mut()
                .unwrap()
                .insert(HeaderName::from_str("My-Custom-Header").unwrap(), value);
        }
        response.body(None).map_err(JsError::from_rust)
    }
}

#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct WptFetcher {
    inner: Gc<GcRefCell<WptFetcherInner>>,
}

impl WptFetcher {
    pub fn new(wpt_root: impl Into<PathBuf>) -> Self {
        Self {
            inner: Gc::new(GcRefCell::new(WptFetcherInner::new(wpt_root))),
        }
    }

    pub fn set_current_file(&mut self, file: impl Into<PathBuf>) {
        self.inner.borrow_mut().current_file = Some(file.into());
    }
}

impl Fetcher for WptFetcher {
    fn resolve_uri(&self, uri: String, _context: &mut Context) -> JsResult<String> {
        let Some(current_file) = &self.inner.borrow().current_file else {
            return Err(js_error!("No current file was set by the test framework."));
        };
        let wpt_root = &self.inner.borrow().wpt_root;

        let base = Url::from_file_path(current_file).expect("Invalid wpt root path");
        let result = base
            .join(&uri)
            .map(|mut u| {
                u.set_host(Some("wpt-test"))
                    .expect("Unable to set URL host.");

                let full = PathBuf::from(u.path());
                let path = full
                    .strip_prefix(wpt_root)
                    .expect("File should always be under root.")
                    .to_str()
                    .unwrap();
                u.set_path(path);

                u.to_string()
            })
            .map_err(JsError::from_rust);

        result
    }

    fn fetch_blocking(
        &self,
        request: http::request::Request<Option<Vec<u8>>>,
        _context: &mut Context,
    ) -> JsResult<http::response::Response<Option<Vec<u8>>>> {
        eprintln!("fetch --- {}", request.uri().path());
        match request.uri().path() {
            "/fetch/api/resources/inspect-headers.py" => {
                self.inner.borrow().fetch_inspect_header(request)
            }
            "/xhr/resources/parse-headers.py" => self.inner.borrow().parse_headers(request),
            _ => Ok(http::response::Response::builder()
                .status(401)
                .body(None)
                .unwrap()),
        }
    }
}
