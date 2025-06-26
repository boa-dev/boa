#![cfg(test)]

use crate::fetch::request::JsRequest;
use crate::test::{run_test_actions, TestAction};
use crate::{register_fetch, Fetcher};
use boa_engine::{js_error, js_str, js_string, Context, Finalize, JsData, JsResult, Trace};
use either::Either;
use http::{Request, Response, Uri};
use std::cell::RefCell;
use std::collections::HashMap;

/// A [`Fetcher`] implementation for tests. Maps a URL to a response,
/// and record requests received for later use.
///
/// The actual safety of this implementation is not guaranteed, as it
/// is only intended for testing purposes.
#[derive(Default, Debug, Trace, Finalize, JsData)]
pub struct TestFetcher {
    #[unsafe_ignore_trace]
    requests_received: RefCell<Vec<Request<Vec<u8>>>>,
    #[unsafe_ignore_trace]
    request_mapper: HashMap<Uri, Response<Vec<u8>>>,
}

impl TestFetcher {
    /// Add a response mapping for a URL.
    pub fn add_response(&mut self, url: Uri, response: Response<Vec<u8>>) {
        self.request_mapper.insert(url, response);
    }
}

impl Fetcher for TestFetcher {
    fn fetch_blocking(
        &self,
        request: &Request<Vec<u8>>,
        context: &mut Context,
    ) -> JsResult<Response<Vec<u8>>> {
        self.requests_received.borrow_mut().push(request.clone());
        let url = request.uri();
        self.request_mapper
            .get(url)
            .cloned()
            .ok_or_else(|| js_error!("No response found for URL"))
    }
}

#[test]
fn request_constructor() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let mut fetcher = TestFetcher::default();
            fetcher.add_response(
                Uri::from_static("http://example.com"),
                Response::new("Hello World".as_bytes().to_vec()),
            );
            register_fetch(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                const request = new Request("http://example.com");
                globalThis.response = fetch(request);
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            let response = response.as_promise().unwrap().await_blocking(ctx).unwrap();

            eprintln!("Response: {response:?}");
        }),
        TestAction::inspect_context(|_ctx| {
            let request =
                JsRequest::create_from_js(Either::Left(js_string!("http://example.com")), None)
                    .unwrap();
            assert_eq!(request.uri().to_string(), "http://example.com/");
        }),
    ]);
}
