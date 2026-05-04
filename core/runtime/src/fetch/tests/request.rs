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

#[test]
fn request_body_methods() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                globalThis.promise = (async () => {
                    const textRequest = new Request("http://unit.test", {
                        method: "POST",
                        body: "",
                    });
                    assertEq(textRequest.bodyUsed, false);
                    assertEq(await textRequest.text(), "");
                    assertEq(textRequest.bodyUsed, true);

                    const bytesRequest = new Request("http://unit.test", {
                        method: "POST",
                        body: "hello",
                    });
                    const bytes = await bytesRequest.bytes();
                    assertEq(new TextDecoder().decode(bytes), "hello");
                    assertEq(bytesRequest.bodyUsed, true);

                    const jsonRequest = new Request("http://unit.test", {
                        method: "POST",
                        body: '{ "value": 1 }',
                    });
                    const json = await jsonRequest.json();
                    assertEq(json.value, 1);
                    assertEq(jsonRequest.bodyUsed, true);
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let promise = ctx.global_object().get(js_str!("promise"), ctx).unwrap();
            promise.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn request_without_body_is_not_disturbed_by_reads() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                globalThis.promise = (async () => {
                    const request = new Request("http://unit.test");
                    assertEq(await request.text(), "");
                    assertEq(await request.text(), "");
                    assertEq(request.bodyUsed, false);
                    const cloned = request.clone();
                    assertEq(cloned instanceof Request, true);
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let promise = ctx.global_object().get(js_str!("promise"), ctx).unwrap();
            promise.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn request_used_body_cannot_be_reused() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                globalThis.promise = (async () => {
                    const request = new Request("http://unit.test", {
                        method: "POST",
                        body: "payload",
                    });

                    assertEq(await request.text(), "payload");

                    for (const action of [
                        () => request.clone(),
                        () => new Request(request),
                    ]) {
                        try {
                            action();
                            throw Error("expected the call above to throw");
                        } catch (e) {
                            if (!(e instanceof TypeError)) {
                                throw e;
                            }
                        }
                    }

                    const overridden = new Request(request, {
                        method: "POST",
                        body: "override",
                    });
                    assertEq(await overridden.text(), "override");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let promise = ctx.global_object().get(js_str!("promise"), ctx).unwrap();
            promise.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn request_constructor_consumes_source_body() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                globalThis.promise = (async () => {
                    const withBody = new Request("http://unit.test", {
                        method: "POST",
                        body: "payload",
                    });
                    const copied = new Request(withBody);
                    assertEq(withBody.bodyUsed, true);
                    assertEq(await copied.text(), "payload");

                    const withEmptyBody = new Request("http://unit.test", {
                        method: "POST",
                        body: "",
                    });
                    const copiedEmpty = new Request(withEmptyBody);
                    assertEq(withEmptyBody.bodyUsed, true);
                    assertEq(await copiedEmpty.text(), "");

                    const withoutBody = new Request("http://unit.test");
                    const copiedWithoutBody = new Request(withoutBody);
                    assertEq(withoutBody.bodyUsed, false);
                    assertEq(await copiedWithoutBody.text(), "");

                    const overridden = new Request("http://unit.test", {
                        method: "POST",
                        body: "payload",
                    });
                    const overrideCopy = new Request(overridden, {
                        method: "POST",
                        body: "override",
                    });
                    assertEq(overridden.bodyUsed, false);
                    assertEq(await overrideCopy.text(), "override");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let promise = ctx.global_object().get(js_str!("promise"), ctx).unwrap();
            promise.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn request_constructor_does_not_consume_source_when_it_throws() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                const request = new Request("http://unit.test", {
                    method: "POST",
                    body: "payload",
                });

                try {
                    new Request(request, { method: "CONNECT" });
                    throw Error("expected the call above to throw");
                } catch (e) {
                    if (!(e instanceof TypeError)) {
                        throw e;
                    }
                }

                assertEq(request.bodyUsed, false);
            "#,
        ),
    ]);
}

#[test]
fn fetch_marks_request_body_used() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            let mut fetcher = TestFetcher::default();
            fetcher.add_response(
                Uri::from_static("http://unit.test"),
                Response::new("ok".as_bytes().to_vec()),
            );
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(
            r#"
                globalThis.promise = (async () => {
                    const request = new Request("http://unit.test", {
                        method: "POST",
                        body: "payload",
                    });

                    const response = await fetch(request);
                    assertEq(await response.text(), "ok");
                    assertEq(request.bodyUsed, true);

                    try {
                        await fetch(request);
                        throw Error("expected the call above to throw");
                    } catch (e) {
                        if (!(e instanceof TypeError)) {
                            throw e;
                        }
                    }

                    const overrideResponse = await fetch(request, {
                        method: "POST",
                        body: "override",
                    });
                    assertEq(await overrideResponse.text(), "ok");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let promise = ctx.global_object().get(js_str!("promise"), ctx).unwrap();
            promise.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}
