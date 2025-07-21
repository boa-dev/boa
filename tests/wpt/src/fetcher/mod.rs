use boa_engine::{js_error, Context, Finalize, JsData, JsError, JsResult, Trace};
use boa_runtime::fetch::request::JsRequest;
use boa_runtime::fetch::response::JsResponse;
use boa_runtime::fetch::BlockingReqwestFetcher;
use boa_runtime::fetch::Fetcher;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use url::Url;

#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct WptFetcher {
    wpt_server: String,
    wpt_root: PathBuf,
    current_file: Rc<RefCell<Option<PathBuf>>>,

    #[unsafe_ignore_trace]
    inner: Rc<BlockingReqwestFetcher>,
}

impl WptFetcher {
    pub fn new(wpt_root: impl Into<PathBuf>, wpt_server: String) -> Self {
        Self {
            wpt_server,
            wpt_root: wpt_root.into(),
            current_file: Default::default(),
            inner: Rc::new(BlockingReqwestFetcher::default()),
        }
    }

    pub fn set_current_file(&mut self, file: impl Into<PathBuf>) {
        self.current_file.borrow_mut().replace(file.into());
    }
}

impl Fetcher for WptFetcher {
    fn resolve_uri(&self, uri: String, _context: &mut Context) -> JsResult<String> {
        // If it's already a valid URL, return it.
        if let Ok(u) = Url::parse(&uri) {
            return Ok(u.to_string());
        }

        // If it's a relative URL, we need to perform some resolution first...
        let cf = self.current_file.borrow();
        let Some(current_file) = cf.as_ref() else {
            return Err(js_error!("No current file was set by the test framework."));
        };
        let wpt_root = &self.wpt_root;

        let base = Url::from_file_path(current_file).expect("Invalid wpt root path");
        let url = base.join(&uri).expect("Invalid URL (join)");

        let full = PathBuf::from(url.path());
        let path = full
            .strip_prefix(wpt_root)
            .expect("File should always be under root.")
            .to_str()
            .unwrap();

        let wpt_server = &self.wpt_server;
        let query = url.query().map_or("".to_string(), |q| format!("?{q}"));
        let fragment = url.fragment().map_or("".to_string(), |q| format!("#{q}"));

        Url::parse(&format!("http://{wpt_server}/{path}{query}{fragment}"))
            .map_err(JsError::from_rust)
            .map(|url| url.to_string())
    }

    async fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        context: &RefCell<&mut Context>,
    ) -> JsResult<JsResponse> {
        eprintln!("request: {request:?}");
        let response = self.inner.clone().fetch(request, context).await;
        eprintln!("response: {response:?}");
        response
    }
}
