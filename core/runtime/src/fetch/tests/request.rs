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
        TestAction::inspect_context(|ctx| {
            let request = JsRequest::create_from_js(
                Either::Left(js_string!("http://example.com")),
                None,
                ctx,
            )
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
#[test]
fn request_body_typedarray() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                const buf = new Uint8Array([104, 101, 108, 108, 111]); // "hello"
                globalThis.req1 = new Request("http://unit.test", {
                    method: "POST",
                    body: buf,
                });
                const dv = new DataView(buf.buffer);
                globalThis.req2 = new Request("http://unit.test", {
                    method: "POST",
                    body: dv,
                });
                // Uint8Array subarray exercising offset/length slicing ("ell")
                const sub = buf.subarray(1, 4);
                globalThis.req3 = new Request("http://unit.test", {
                    method: "POST",
                    body: sub,
                });
                // DataView with non-zero byteOffset and explicit byteLength ("ell")
                const dvSlice = new DataView(buf.buffer, 1, 3);
                globalThis.req4 = new Request("http://unit.test", {
                    method: "POST",
                    body: dvSlice,
                });
                // Plain ArrayBuffer body ("hello")
                const ab = buf.buffer;
                globalThis.req5 = new Request("http://unit.test", {
                    method: "POST",
                    body: ab,
                });
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let request1 = ctx.global_object().get(js_str!("req1"), ctx).unwrap();
            let request1_obj = request1.as_object().unwrap();
            let request1 = request1_obj.downcast_ref::<JsRequest>().unwrap();
            assert_eq!(request1.inner().body().as_slice(), b"hello");

            let request2 = ctx.global_object().get(js_str!("req2"), ctx).unwrap();
            let request2_obj = request2.as_object().unwrap();
            let request2 = request2_obj.downcast_ref::<JsRequest>().unwrap();
            assert_eq!(request2.inner().body().as_slice(), b"hello");

            let request3 = ctx.global_object().get(js_str!("req3"), ctx).unwrap();
            let request3_obj = request3.as_object().unwrap();
            let request3 = request3_obj.downcast_ref::<JsRequest>().unwrap();
            assert_eq!(request3.inner().body().as_slice(), b"ell");

            let request4 = ctx.global_object().get(js_str!("req4"), ctx).unwrap();
            let request4_obj = request4.as_object().unwrap();
            let request4 = request4_obj.downcast_ref::<JsRequest>().unwrap();
            assert_eq!(request4.inner().body().as_slice(), b"ell");

            let request5 = ctx.global_object().get(js_str!("req5"), ctx).unwrap();
            let request5_obj = request5.as_object().unwrap();
            let request5 = request5_obj.downcast_ref::<JsRequest>().unwrap();
            assert_eq!(request5.inner().body().as_slice(), b"hello");
        }),
    ]);
}
