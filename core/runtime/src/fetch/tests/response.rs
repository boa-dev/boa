use super::TestFetcher;
use crate::test::{TestAction, run_test_actions};
use boa_engine::{Context, js_str};
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
            assert_eq!(
                format!("{}", response.display_obj(false)),
                "{\n    hello world: 123\n}"
            );
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
fn response_redirect() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            register(&[], ctx);
        }),
        TestAction::run(
            r#"
                const response = Response.redirect("http://example.com/other1", 301);
                assertEq(response.status, 301);
                assertEq(response.headers.get("Location"), "http://example.com/other1");

                const responseDefault = Response.redirect("http://example.com/other2");
                assertEq(responseDefault.status, 302);
                assertEq(responseDefault.headers.get("Location"), "http://example.com/other2");

                let threw = false;
                try {
                    Response.redirect("http://example.com/", 200);
                } catch(e) {
                    threw = e instanceof RangeError;
                }
                assertEq(threw, true);
            "#,
        ),
    ]);
}

#[test]
fn response_static_json() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(|ctx| {
            register(&[], ctx);
        }),
        TestAction::run(
            r#"
                globalThis.response_tests = (async () => {
                    const data = { message: "Hello" };
                    const res = Response.json(data);
                    assertEq(res.status, 200);
                    assertEq(res.headers.get("Content-Type"), "application/json");

                    const body = await res.json();
                    assertEq(body.message, "Hello");

                    const res2 = Response.json([1, 2, 3], { status: 201, headers: { "X-Test": "test" } });
                    assertEq(res2.status, 201);
                    assertEq(res2.headers.get("Content-Type"), "application/json");
                    assertEq(res2.headers.get("X-Test"), "test");
                    
                    const res3 = Response.json(null, { headers: { "Content-Type": "application/custom+json" } });
                    assertEq(res3.headers.get("Content-Type"), "application/custom+json");
                })();
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            let response = ctx
                .global_object()
                .get(js_str!("response_tests"), ctx)
                .unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}
