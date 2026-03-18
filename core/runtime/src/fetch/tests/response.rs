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
            let p = ctx.global_object().get(js_str!("p"), ctx).unwrap();
            p.as_promise().unwrap().await_blocking(ctx).unwrap();
            let response = ctx.global_object().get(js_str!("response"), ctx).unwrap();
            response.as_promise().unwrap().await_blocking(ctx).unwrap();
        }),
    ]);
}
