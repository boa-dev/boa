use super::TestFetcher;
use crate::fetch::request::JsRequest;
use crate::fetch::response::JsResponse;
use crate::test::{TestAction, run_test_actions};
use boa_engine::{JsObject, js_str, js_string};
use either::Either;
use http::{Response, Uri};
use indoc::indoc;

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
fn request_constructor_forbidden_method_throws() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(indoc! {r#"
            for (const method of ["CONNECT", "TRACE", "TRACK", "connect"]) {
                try {
                    new Request("http://unit.test", { method });
                    throw Error("expected the call above to throw");
                } catch (e) {
                    if (!(e instanceof TypeError)) {
                        throw e;
                    }
                }
            }
        "#}),
    ]);
}

#[test]
fn request_constructor_get_with_body_throws() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(indoc! {r#"
            try {
                new Request("http://unit.test", { method: "GET", body: "x" });
                throw Error("expected the call above to throw");
            } catch (e) {
                if (!(e instanceof TypeError)) {
                    throw e;
                }
            }
        "#}),
    ]);
}

#[test]
fn request_constructor_head_with_body_throws() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(indoc! {r#"
            try {
                new Request("http://unit.test", { method: "HEAD", body: "x" });
                throw Error("expected the call above to throw");
            } catch (e) {
                if (!(e instanceof TypeError)) {
                    throw e;
                }
            }
        "#}),
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
fn request_clone_method_preserves_body() {
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
                globalThis.cloned = original.clone();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let cloned = ctx.global_object().get(js_str!("cloned"), ctx).unwrap();
            let cloned_obj = cloned.as_object().unwrap();
            let cloned_req = cloned_obj.downcast_ref::<JsRequest>().unwrap();
            assert_eq!(cloned_req.inner().body().as_slice(), b"payload");
        }),
    ]);
}

#[test]
fn request_clone_method_is_independent() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                const original = new Request("http://unit.test", {
                    method: "POST",
                    body: "original-body",
                });
                globalThis.original = original;
                globalThis.cloned = original.clone();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let original = ctx.global_object().get(js_str!("original"), ctx).unwrap();
            let original_obj = original.as_object().unwrap();
            let original_req = original_obj.downcast_ref::<JsRequest>().unwrap();

            let cloned = ctx.global_object().get(js_str!("cloned"), ctx).unwrap();
            let cloned_obj = cloned.as_object().unwrap();
            let cloned_req = cloned_obj.downcast_ref::<JsRequest>().unwrap();

            assert_eq!(original_req.inner().body().as_slice(), b"original-body");
            assert_eq!(cloned_req.inner().body().as_slice(), b"original-body");

            // Verify they are distinct objects (different pointers).
            assert!(!std::ptr::eq(
                original_req.inner().body().as_ptr(),
                cloned_req.inner().body().as_ptr()
            ));
        }),
    ]);
}

#[test]
fn request_stores_signal() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                globalThis.ctrl = new AbortController();
                globalThis.request = new Request("http://unit.test", {
                    signal: ctrl.signal,
                });
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let request = ctx.global_object().get(js_str!("request"), ctx).unwrap();
            let request_obj = request.as_object().unwrap();
            let request = request_obj.downcast_ref::<JsRequest>().unwrap();

            let signal = ctx.global_object().get(js_str!("ctrl"), ctx).unwrap();
            let signal = signal
                .as_object()
                .unwrap()
                .get(js_str!("signal"), ctx)
                .unwrap()
                .as_object()
                .unwrap();

            let stored_signal = request.signal().expect("request should keep its signal");
            assert!(JsObject::equals(&stored_signal, &signal));
        }),
    ]);
}

#[test]
fn request_clone_preserves_signal_without_override() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                globalThis.ctrl = new AbortController();
                const original = new Request("http://unit.test", {
                    signal: ctrl.signal,
                });
                globalThis.cloned = new Request(original, {
                    headers: { "x-test": "1" },
                });
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let cloned = ctx.global_object().get(js_str!("cloned"), ctx).unwrap();
            let cloned_obj = cloned.as_object().unwrap();
            let cloned = cloned_obj.downcast_ref::<JsRequest>().unwrap();

            let signal = ctx.global_object().get(js_str!("ctrl"), ctx).unwrap();
            let signal = signal
                .as_object()
                .unwrap()
                .get(js_str!("signal"), ctx)
                .unwrap()
                .as_object()
                .unwrap();

            let stored_signal = cloned
                .signal()
                .expect("cloned request should keep its signal");
            assert!(JsObject::equals(&stored_signal, &signal));
        }),
    ]);
}

#[test]
fn request_clone_signal_override() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                globalThis.ctrl1 = new AbortController();
                globalThis.ctrl2 = new AbortController();
                const original = new Request("http://unit.test", {
                    signal: ctrl1.signal,
                });
                globalThis.cloned = new Request(original, {
                    signal: ctrl2.signal,
                });
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let cloned = ctx.global_object().get(js_str!("cloned"), ctx).unwrap();
            let cloned_obj = cloned.as_object().unwrap();
            let cloned = cloned_obj.downcast_ref::<JsRequest>().unwrap();

            let signal = ctx.global_object().get(js_str!("ctrl2"), ctx).unwrap();
            let signal = signal
                .as_object()
                .unwrap()
                .get(js_str!("signal"), ctx)
                .unwrap()
                .as_object()
                .unwrap();

            let stored_signal = cloned
                .signal()
                .expect("overridden request should keep the new signal");
            assert!(JsObject::equals(&stored_signal, &signal));
        }),
    ]);
}
