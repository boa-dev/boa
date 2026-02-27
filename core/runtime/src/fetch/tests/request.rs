use super::TestFetcher;
use crate::fetch::request::JsRequest;
use crate::fetch::response::JsResponse;
use crate::test::{TestAction, run_test_actions};
use boa_engine::{js_str, js_string};
use either::Either;
use http::{Response, Uri};

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
fn request_clone_preserves_body_without_override() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                const original = new Request("http://unit.test", {
                    method: "POST",
                    body: "payload",
                });
                globalThis.cloned = new Request(original, {
                    headers: { "x-test": "1" },
                });
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let request = ctx.global_object().get(js_str!("cloned"), ctx).unwrap();
            let request_obj = request.as_object().unwrap();
            let request = request_obj.downcast_ref::<JsRequest>().unwrap();
            assert_eq!(request.inner().body().as_slice(), b"payload");
        }),
    ]);
}

#[test]
fn request_clone_empty_body_preserved() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                const original = new Request("http://unit.test", {
                    method: "POST",
                    body: "",
                });
                globalThis.cloned = new Request(original, {
                    headers: { "x-test": "1" },
                });
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let request = ctx.global_object().get(js_str!("cloned"), ctx).unwrap();
            let request_obj = request.as_object().unwrap();
            let request = request_obj.downcast_ref::<JsRequest>().unwrap();
            assert_eq!(request.inner().body().as_slice(), b"");
        }),
    ]);
}

#[test]
fn request_clone_body_override() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                const original = new Request("http://unit.test", {
                    method: "POST",
                    body: "payload",
                });
                globalThis.cloned = new Request(original, {
                    body: "override",
                });
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let request = ctx.global_object().get(js_str!("cloned"), ctx).unwrap();
            let request_obj = request.as_object().unwrap();
            let request = request_obj.downcast_ref::<JsRequest>().unwrap();
            assert_eq!(request.inner().body().as_slice(), b"override");
        }),
    ]);
}

#[test]
fn request_clone_no_body_preserved() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                const original = new Request("http://unit.test");
                globalThis.cloned = new Request(original, {
                    headers: { "x-test": "1" },
                });
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let request = ctx.global_object().get(js_str!("cloned"), ctx).unwrap();
            let request_obj = request.as_object().unwrap();
            let request = request_obj.downcast_ref::<JsRequest>().unwrap();
            assert_eq!(request.inner().body().as_slice(), b"");
        }),
    ]);
}
