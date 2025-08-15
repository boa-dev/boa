//! Test types and methods to help with testing the Fetch API.

use crate::fetch::request::JsRequest;
use crate::fetch::response::JsResponse;
use boa_engine::{Context, Finalize, JsData, JsResult, JsString, Trace, js_error};
use http::{Request, Response, Uri};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(test)]
mod e2e;
#[cfg(test)]
mod request;
#[cfg(test)]
mod response;

/// A [`crate::fetch::Fetcher`] implementation for tests. Maps a URL to a response,
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

impl crate::fetch::Fetcher for TestFetcher {
    async fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        _context: &RefCell<&mut Context>,
    ) -> JsResult<JsResponse> {
        let request = request.into_inner();
        self.requests_received.borrow_mut().push(request.clone());
        let url = request.uri();
        self.request_mapper
            .get(url)
            .cloned()
            .map(|response| JsResponse::basic(JsString::from(url.to_string()), response))
            .ok_or_else(|| js_error!("No response found for URL"))
    }
}
