use crate::fetch::request::JsRequest;
use crate::fetch::response::JsResponse;
use crate::test::{run_test_actions, TestAction};
use boa_engine::{
    js_error, js_str, Context, Finalize, JsData, JsError, JsResult, JsString, Trace,
};
use http::Response;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use url::Url;

/// This is a special end-to-end fetcher that processes requests.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
struct E2eFetcher;

impl E2eFetcher {
    fn headers(request: &JsRequest, _context: &mut Context) -> JsResult<JsResponse> {
        let url = Url::parse(&request.uri().to_string()).map_err(JsError::from_rust)?;
        let request_query: BTreeMap<String, String> = url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();

        let Some(header) = request_query.get("header") else {
            return Err(js_error!("Invalid query."));
        };

        let mut response = Response::new(b"".to_vec());
        response.headers_mut().append(
            "x-headers",
            request
                .inner()
                .headers()
                .get(header)
                .cloned()
                .unwrap_or("--not found--".parse().unwrap()),
        );

        Ok(JsResponse::basic(
            JsString::from(url.to_string().as_str()),
            response,
        ))
    }
}

impl crate::fetch::Fetcher for E2eFetcher {
    async fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        context: &RefCell<&mut Context>,
    ) -> JsResult<JsResponse> {
        match request.uri().path() {
            "/headers" => Self::headers(&request, &mut context.borrow_mut()),
            _ => Err(js_error!("Invalid request.")),
        }
    }
}

fn register(ctx: &mut Context) {
    let fetcher = E2eFetcher;
    crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
}

fn await_response(ctx: &mut Context) {
    let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
    response.as_promise().unwrap().await_blocking(ctx).unwrap();
}

#[test]
fn custom_header() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const response = await fetch("http://unit.test/headers?header=test", {
                        headers: {
                            "test": "123",
                        },
                    });
                    assertEq(response.headers.get("x-headers"), "123");

                    const text = await response.text();
                    assertEq(text, "");
                })();
            "#,
        ),
        TestAction::inspect_context(await_response),
    ]);
}
