use super::TestFetcher;
use crate::fetch::request::{JsRequest, RequestInit};
use crate::fetch::response::JsResponse;
use crate::test::{TestAction, run_test_actions};
use boa_engine::{js_str, js_string};
use either::Either;
use http::{Response, Uri};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn request_constructor() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let mut fetcher = TestFetcher::default();
            fetcher.add_response(
                Uri::from_static("http://unit.test"),
                Response::new("Hello World".as_bytes().to_vec()),
            );
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                const request = new Request("http://unit.test");
                globalThis.response = fetch(request);
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            let response = response.as_promise().unwrap().await_blocking(ctx).unwrap();

            assert_eq!(
                response
                    .as_object()
                    .as_ref()
                    .and_then(|o| o.downcast_ref::<JsResponse>())
                    .unwrap()
                    .body()
                    .as_ref()
                    .as_slice(),
                "Hello World".as_bytes()
            );
        }),
        TestAction::inspect_context(|_ctx| {
            let request =
                JsRequest::create_from_js(Either::Left(js_string!("http://example.com")), None)
                    .unwrap();
            assert_eq!(request.uri().to_string(), "http://example.com/");
        }),
    ]);
}

#[test]
fn request_clone_preserves_body_with_options() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let mut fetcher = TestFetcher::default();
            fetcher.add_response(
                Uri::from_static("http://unit.test"),
                Response::new("response".as_bytes().to_vec()),
            );
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                const original = new Request("http://unit.test", {
                    method: "POST",
                    body: "payload",
                });
                const withHeaders = new Request(original, {
                    headers: { "x-test": "1" },
                });
                globalThis.response = fetch(withHeaders);
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            let response = response.as_promise().unwrap().await_blocking(ctx).unwrap();

            // Verify the response came back (meaning the request was successful).
            assert!(
                response
                    .as_object()
                    .as_ref()
                    .and_then(|o| o.downcast_ref::<JsResponse>())
                    .is_some(),
            );
        }),
    ]);
}

#[test]
fn request_clone_preserves_body_rust_api() {
    // Create a request with a body via the Rust API.
    let original = JsRequest::create_from_js(
        Either::Left(js_string!("http://example.com")),
        Some(RequestInit::new(
            Some("test body".to_string()),
            None,
            Some("POST".to_string()),
        )),
    )
    .unwrap();

    assert_eq!(original.inner().body().as_slice(), b"test body");
    assert_eq!(original.inner().method(), "POST");

    // Clone with only headers changed (no body in options).
    let cloned = JsRequest::create_from_js(
        Either::Right(original),
        Some(RequestInit::new(None, None, None)),
    )
    .unwrap();

    // Body must be preserved from the original request.
    assert_eq!(cloned.inner().body().as_slice(), b"test body");
    assert_eq!(cloned.inner().method(), "POST");
}

#[test]
fn request_clone_allows_body_override() {
    // Create a request with a body.
    let original = JsRequest::create_from_js(
        Either::Left(js_string!("http://example.com")),
        Some(RequestInit::new(
            Some("original body".to_string()),
            None,
            Some("POST".to_string()),
        )),
    )
    .unwrap();

    // Clone with a new body explicitly provided.
    let cloned = JsRequest::create_from_js(
        Either::Right(original),
        Some(RequestInit::new(Some("new body".to_string()), None, None)),
    )
    .unwrap();

    // Body must be the new one, not the original.
    assert_eq!(cloned.inner().body().as_slice(), b"new body");
}

/// A fetcher that echoes the request body back as the response body.
/// This lets JS-level tests verify the exact body content end-to-end.
#[derive(Debug, Default, boa_engine::Trace, boa_engine::Finalize, boa_engine::JsData)]
struct EchoBodyFetcher;

impl crate::fetch::Fetcher for EchoBodyFetcher {
    async fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        _context: &RefCell<&mut boa_engine::Context>,
    ) -> boa_engine::JsResult<JsResponse> {
        let inner = request.into_inner();
        let body = inner.body().clone();
        let url = inner.uri().to_string();
        let response = Response::new(body);
        Ok(JsResponse::basic(boa_engine::JsString::from(url), response))
    }
}

/// Exact reproduction case from issue #4686.
/// POST body must survive cloning when only headers are added.
#[test]
fn issue_4686_body_preserved_when_cloning_with_headers() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            crate::fetch::register(EchoBodyFetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const original = new Request("http://unit.test", {
                        method: "POST",
                        body: "payload",
                    });
                    const withHeaders = new Request(original, {
                        headers: { "x-test": "1" },
                    });
                    const resp = await fetch(withHeaders);
                    const text = await resp.text();
                    assertEq(text, "payload");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

/// Cloning a request with a method change should still preserve the body.
#[test]
fn request_clone_with_method_change_preserves_body() {
    let original = JsRequest::create_from_js(
        Either::Left(js_string!("http://example.com")),
        Some(RequestInit::new(
            Some("keep me".to_string()),
            None,
            Some("POST".to_string()),
        )),
    )
    .unwrap();

    let cloned = JsRequest::create_from_js(
        Either::Right(original),
        Some(RequestInit::new(None, None, Some("PUT".to_string()))),
    )
    .unwrap();

    assert_eq!(cloned.inner().method(), "PUT");
    assert_eq!(cloned.inner().body().as_slice(), b"keep me");
}

/// Cloning a request without any options should preserve everything.
#[test]
fn request_clone_without_options_preserves_body() {
    let original = JsRequest::create_from_js(
        Either::Left(js_string!("http://example.com")),
        Some(RequestInit::new(
            Some("original".to_string()),
            None,
            Some("POST".to_string()),
        )),
    )
    .unwrap();

    let cloned = JsRequest::create_from_js(Either::Right(original), None).unwrap();

    assert_eq!(cloned.inner().body().as_slice(), b"original");
    assert_eq!(cloned.inner().method(), "POST");
}

/// Cloning a GET request (no body) with options should still have an empty body.
#[test]
fn request_clone_no_body_stays_empty() {
    let original =
        JsRequest::create_from_js(Either::Left(js_string!("http://example.com")), None).unwrap();

    assert!(original.inner().body().is_empty());

    let cloned = JsRequest::create_from_js(
        Either::Right(original),
        Some(RequestInit::new(None, None, None)),
    )
    .unwrap();

    assert!(cloned.inner().body().is_empty());
    assert_eq!(cloned.inner().method(), "GET");
}

/// End-to-end: body override through JS constructor should send the new body.
#[test]
fn issue_4686_body_override_e2e() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            crate::fetch::register(EchoBodyFetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const original = new Request("http://unit.test", {
                        method: "POST",
                        body: "old",
                    });
                    const replaced = new Request(original, {
                        body: "new",
                    });
                    const resp = await fetch(replaced);
                    const text = await resp.text();
                    assertEq(text, "new");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}
