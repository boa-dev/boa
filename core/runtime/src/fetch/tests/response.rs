use super::TestFetcher;
use crate::test::{TestAction, run_test_actions};
use boa_engine::{Context, JsValue, js_str};
use http::{Response, Uri};

fn register(responses: &[(&'static str, Response<Vec<u8>>)], ctx: &mut Context) {
    let mut fetcher = TestFetcher::default();

    for (url, resp) in responses {
        fetcher.add_response(Uri::from_static(url), resp.clone());
    }
    crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
}

#[test]
fn response_error() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| register(&[], ctx)),
        TestAction::run(
            r#"
                const response = Response.error();

                assertEq(response.status, 0);
                assertEq(response.statusText, "");
                assertEq(response.headers.get("custom-header"), null);
                assertEq(response.type, "error");
                assertEq(response.url, "");
            "#,
        ),
        // Assertions made in JavaScript.
    ]);
}

#[test]
fn response_constructor_null_body_status_throws() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| register(&[], ctx)),
        TestAction::run(
            r#"
                for (const status of [204, 205, 304]) {
                    try {
                        new Response("x", { status });
                        throw Error("expected the call above to throw");
                    } catch (e) {
                        if (!(e instanceof TypeError)) {
                            throw e;
                        }
                    }
                }
            "#,
        ),
    ]);
}

#[test]
fn response_text() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            register(
                &[("http://unit.test", Response::new(b"Hello World".to_vec()))],
                ctx,
            );
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const request = new Request("http://unit.test");
                    const response = await fetch(request);
                    const text = await response.text();
                    assertEq(text, "Hello World");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn response_json() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            register(
                &[(
                    "http://unit.test",
                    Response::new(b"{ \"hello world\": 123 }".to_vec()),
                )],
                ctx,
            );
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const request = new Request("http://unit.test");
                    const response = await fetch(request);
                    const json = await response.json();
                    assertEq(json["hello world"], 123);
                    return json;
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            let response = response.as_promise().unwrap().await_blocking(ctx).unwrap();
            assert_eq!(response.display_obj(false), "{\n    hello world: 123\n}");
        }),
    ]);
}

#[test]
fn response_bytes() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            register(
                &[("http://unit.test", Response::new(b"Hello World".to_vec()))],
                ctx,
            );
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const request = new Request("http://unit.test");
                    const response = await fetch(request);
                    const bytes = await response.bytes();
                    const text = new TextDecoder().decode(bytes);
                    assertEq(text, "Hello World");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn response_getter() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            let mut response = Response::new(b"Hello World".to_vec());
            response
                .headers_mut()
                .append("custom-header", "custom-value".parse().unwrap());
            register(&[("http://unit.test", response)], ctx);
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const request = new Request("http://unit.test");
                    const response = await fetch(request);

                    assertEq(response.status, 200);
                    assertEq(response.statusText, "OK");
                    assertEq(response.headers.get("custom-header"), "custom-value");
                    assertEq(response.type, "basic");
                    assertEq(response.url, "http://unit.test/");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();

            // Assertions made in JavaScript.
        }),
    ]);
}

#[test]
fn fetch_request_uses_request_signal() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            register(
                &[("http://unit.test", Response::new(b"Hello World".to_vec()))],
                ctx,
            );
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const ctrl = new AbortController();
                    const request = new Request("http://unit.test", { signal: ctrl.signal });
                    ctrl.abort("stop");

                    try {
                        await fetch(request);
                        return "fulfilled";
                    } catch (e) {
                        return e;
                    }
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            let response = response.as_promise().unwrap().await_blocking(ctx).unwrap();
            assert_eq!(
                response.as_string().map(|s| s.to_std_string_escaped()),
                Some("stop".into())
            );
        }),
    ]);
}

#[test]
fn fetch_options_signal_overrides_request_signal() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            register(
                &[("http://unit.test", Response::new(b"Hello World".to_vec()))],
                ctx,
            );
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const requestCtrl = new AbortController();
                    const fetchCtrl = new AbortController();
                    const request = new Request("http://unit.test", {
                        signal: requestCtrl.signal,
                    });
                    fetchCtrl.abort("fetch stop");

                    try {
                        await fetch(request, { signal: fetchCtrl.signal });
                        return "fulfilled";
                    } catch (e) {
                        return e;
                    }
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            let response = response.as_promise().unwrap().await_blocking(ctx).unwrap();
            assert_eq!(
                response.as_string().map(|s| s.to_std_string_escaped()),
                Some("fetch stop".into())
            );
        }),
    ]);
}

#[test]
fn response_redirect_default_status() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| register(&[], ctx)),
        TestAction::run(
            r#"
                const response = Response.redirect("http://example.com/");
                assertEq(response.status, 302);
                assertEq(response.headers.get("location"), "http://example.com/");
            "#,
        ),
    ]);
}

#[test]
fn response_redirect_type_default() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| register(&[], ctx)),
        TestAction::run(
            r#"
                assertEq(Response.redirect("http://unit.test").type, "default");
                assertEq(Response.redirect("http://unit.test", 301).type, "default");
            "#,
        ),
    ]);
}

#[test]
fn response_redirect_custom_status_and_coercion() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| register(&[], ctx)),
        TestAction::run(
            r#"
                const response = Response.redirect("http://example.com/", 301);
                assertEq(response.status, 301);

                // Tests Web IDL coercion of the URL parameter
                const response2 = Response.redirect(12345);
                assertEq(response2.headers.get("location"), "12345");
            "#,
        ),
    ]);
}

#[test]
fn response_redirect_invalid_status() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| register(&[], ctx)),
        TestAction::run(
            r#"
                let threw = false;
                try {
                    Response.redirect("http://example.com/", 200);
                } catch (e) {
                    threw = true;
                    if (!(e instanceof RangeError)) {
                        throw new Error("Expected RangeError, got " + e.name);
                    }
                }
                if (!threw) {
                    throw new Error("Expected RangeError, but no error was thrown");
                }
            "#,
        ),
    ]);
}

#[test]
fn response_json_static() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| register(&[], ctx)),
        TestAction::run(
            r#"
                globalThis.p = (async () => {
                    const response = Response.json({ hello: "world" });
                    assertEq(response.status, 200);
                    assertEq(response.headers.get("content-type"), "application/json");
                    const body = await response.json();
                    assertEq(body.hello, "world");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let p = ctx.global_object().get(js_str!("p"), ctx).unwrap();
            p.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn response_headers_get_combines_duplicate_values_with_comma_space() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            let mut response = Response::new(b"Hello World".to_vec());
            response
                .headers_mut()
                .append("x-test", "1".parse().unwrap());
            response
                .headers_mut()
                .append("x-test", "2".parse().unwrap());
            register(&[("http://unit.test", response)], ctx);
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const response = await fetch("http://unit.test");
                    assertEq(response.headers.get("x-test"), "1, 2");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn response_clone_read_both() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            register(
                &[("http://unit.test", Response::new(b"Hello World".to_vec()))],
                ctx,
            );
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const response = await fetch(new Request("http://unit.test"));
                    const cloned = response.clone();
                    const t1 = await response.text();
                    const t2 = await cloned.text();
                    assertEq(t1, "Hello World");
                    assertEq(t2, "Hello World");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn response_clone_header_independence() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            let mut resp = Response::new(b"body".to_vec());
            resp.headers_mut()
                .append("x-original", "value".parse().unwrap());
            register(&[("http://unit.test", resp)], ctx);
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const response = await fetch(new Request("http://unit.test"));
                    const cloned = response.clone();

                    cloned.headers.set("x-original", "mutated");
                    cloned.headers.set("x-new", "added");

                    assertEq(response.headers.get("x-original"), "value");
                    assertEq(response.headers.get("x-new"), null);
                    assertEq(cloned.headers.get("x-original"), "mutated");
                    assertEq(cloned.headers.get("x-new"), "added");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn response_clone_preserves_status() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            register(&[("http://unit.test", Response::new(b"ok".to_vec()))], ctx);
        }),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const response = await fetch(new Request("http://unit.test"));
                    const cloned = response.clone();

                    assertEq(cloned.status, response.status);
                    assertEq(cloned.statusText, response.statusText);
                    assertEq(cloned.type, response.type);
                    assertEq(cloned.url, response.url);
                    assertEq(cloned.ok, response.ok);
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn response_constructor_with_body_and_status() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| register(&[], ctx)),
        TestAction::run(
            r#"
                globalThis.response = (async () => {
                    const response = new Response('Hello World', { status: 404 });
                    assertEq(response.status, 404);
                    assertEq(response.type, "default");
                    const text = await response.text();
                    assertEq(text, "Hello World");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn response_body_buffer_sources() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| register(&[], ctx)),
        TestAction::run(
            r#"
                // ArrayBuffer body
                let ab = new Uint8Array([104,101,108,108,111]).buffer;
                let r0 = new Response(ab);
                assertEq(r0.headers.has("content-type"), false);

                // Uint8Array body (no content-type should be injected)
                let u8a = new Uint8Array([104,101,108,108,111]);
                let r1 = new Response(u8a);
                assertEq(r1.headers.has("content-type"), false);

                // Subarray/view slicing
                let sub = new Uint8Array([0,104,101,108,108,111,0]).subarray(1, 6);
                let r2 = new Response(sub);
                assertEq(r2.headers.has("content-type"), false);

                // DataView with non-zero offset
                let buffer = new Uint8Array([0,104,101,108,108,111,0]).buffer;
                let dv = new DataView(buffer, 1, 5);
                let r3 = new Response(dv);
                assertEq(r3.headers.has("content-type"), false);

                // String body fallback (should retain default text content-type)
                let r4 = new Response("hello");
                assertEq(r4.headers.get("content-type"), "text/plain;charset=UTF-8");

                globalThis.verify = async () => {
                    assertEq(await r0.text(), "hello");
                    assertEq(await r1.text(), "hello");
                    assertEq(await r2.text(), "hello");
                    assertEq(await r3.text(), "hello");
                    assertEq(await r4.text(), "hello");
                };
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let verify_fn = ctx.global_object().get(js_str!("verify"), ctx).unwrap();
            let promise = verify_fn
                .as_callable()
                .unwrap()
                .call(&JsValue::undefined(), &[], ctx)
                .unwrap();
            promise.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}

#[test]
fn response_url_list_and_redirected() {
    use crate::fetch::Fetcher;
    use crate::fetch::request::JsRequest;
    use crate::fetch::response::JsResponse;
    use boa_engine::{Finalize, JsData, JsObject, JsResult, JsString, Trace, js_error};
    use std::cell::RefCell;
    use std::future;
    use std::rc::Rc;

    #[derive(Clone, Debug, Trace, Finalize, JsData)]
    struct RedirectTestFetcher;

    impl Fetcher for RedirectTestFetcher {
        fn fetch(
            self: Rc<Self>,
            request: JsRequest,
            _signal: Option<JsObject>,
            _context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<JsResponse>> {
            let uri = request.uri().to_string();
            let resp = Response::new(b"ok".to_vec());
            let result = match request.uri().path() {
                "/direct" => Ok(JsResponse::basic(JsString::from(uri), resp)),
                "/direct-frag" => Ok(JsResponse::basic(
                    JsString::from("http://unit.test/direct#frag"),
                    resp,
                )),
                "/redirect" => Ok(JsResponse::basic_with_url_list(
                    vec![
                        JsString::from("http://unit.test/redirect"),
                        JsString::from("http://unit.test/target#frag"),
                    ],
                    resp,
                )),
                "/bounce" => Ok(JsResponse::basic_with_url_list(
                    vec![
                        JsString::from("http://unit.test/bounce"),
                        JsString::from("http://unit.test/hop"),
                        JsString::from("http://unit.test/bounce#frag"),
                    ],
                    resp,
                )),
                _ => Err(js_error!("unexpected path")),
            };
            future::ready(result)
        }
    }

    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            crate::fetch::register(RedirectTestFetcher, None, ctx)
                .expect("failed to register RedirectTestFetcher");
        }),
        TestAction::run(
            r#"
                // Synthetic responses have an empty URL list.
                const constructed = new Response("body");
                assertEq(constructed.url, "");
                assertEq(constructed.redirected, false);

                const errResp = Response.error();
                assertEq(errResp.url, "");
                assertEq(errResp.redirected, false);

                const redirStatic = Response.redirect("http://unit.test/other");
                assertEq(redirStatic.url, "");
                assertEq(redirStatic.redirected, false);

                const jsonStatic = Response.json({ a: 1 });
                assertEq(jsonStatic.url, "");
                assertEq(jsonStatic.redirected, false);

                globalThis.response = (async () => {
                    const direct = await fetch("http://unit.test/direct");
                    assertEq(direct.redirected, false);
                    assertEq(direct.url, "http://unit.test/direct");

                    const directFrag = await fetch("http://unit.test/direct-frag");
                    assertEq(directFrag.redirected, false);
                    assertEq(directFrag.url, "http://unit.test/direct");

                    const redirected = await fetch("http://unit.test/redirect");
                    assertEq(redirected.redirected, true);
                    assertEq(redirected.url, "http://unit.test/target");

                    const cloned = redirected.clone();
                    assertEq(cloned.redirected, true);
                    assertEq(cloned.url, "http://unit.test/target");

                    // Redirect A -> B -> A has URL list size 3 (> 1).
                    const bounced = await fetch("http://unit.test/bounce");
                    assertEq(bounced.redirected, true);
                    assertEq(bounced.url, "http://unit.test/bounce");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}
