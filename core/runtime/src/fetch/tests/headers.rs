use super::TestFetcher;
use crate::test::{TestAction, run_test_actions};

fn register(ctx: &mut boa_engine::Context) {
    crate::fetch::register(TestFetcher::default(), None, ctx).expect("failed to register fetch");
}

#[test]
fn headers_are_iterable() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const headers = new Headers([["x", "y"]]);
                const entries = [...headers];
                assertEq(entries.length, 1);
                assertEq(entries[0][0], "x");
                assertEq(entries[0][1], "y");

                const map = new Map(headers);
                assertEq(map.get("x"), "y");
            "#,
        ),
    ]);
}

#[test]
fn headers_get_combines_duplicate_values_with_comma_space() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const headers = new Headers([
                    ["x-test", "1"],
                    ["x-test", "2"],
                ]);

                assertEq(headers.get("x-test"), "1, 2");
            "#,
        ),
    ]);
}

#[test]
fn headers_normalize_values() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const expectations = {
                    name1: [" space ", "space"],
                    name2: ["\ttab\t", "tab"],
                    name3: [" spaceAndTab\t", "spaceAndTab"],
                    name4: ["\r\n newLine", "newLine"],
                    name5: ["newLine\r\n ", "newLine"],
                    name6: ["\r\n\tnewLine", "newLine"],
                };

                const fromObject = new Headers(
                    Object.fromEntries(
                        Object.entries(expectations).map(([name, [value]]) => [name, value]),
                    ),
                );

                for (const [name, [, expected]] of Object.entries(expectations)) {
                    assertEq(fromObject.get(name), expected, `constructor should normalize ${name}`);
                }

                const appended = new Headers();
                for (const [name, [value, expected]] of Object.entries(expectations)) {
                    appended.append(name, value);
                    assertEq(appended.get(name), expected, `append should normalize ${name}`);
                }

                const setHeaders = new Headers();
                for (const [name, [value, expected]] of Object.entries(expectations)) {
                    setHeaders.set(name, value);
                    assertEq(setHeaders.get(name), expected, `set should normalize ${name}`);
                }
            "#,
        ),
    ]);
}

#[test]
fn headers_values_are_byte_strings() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                globalThis.headers = new Headers({"x-test": "1µs\u0080\u00ff"});
                headers.append("x-append", "µ");
                headers.set("set-cookie", "key=µ");
                const expected = "1µs\u0080\u00ff";
                assertEq(headers.get("x-test"), expected);
                assertEq(headers.get("x-append"), "µ");
                assertEq(headers.getSetCookie()[0], "key=µ");
                assertEq([...headers].find(([name]) => name === "x-test")[1], "1µs\u0080\u00ff");
                assertEq([...headers.values()].includes("1µs\u0080\u00ff"), true);
                let values = [];
                headers.forEach(value => values.push(value));
                assertEq(values.includes("1µs\u0080\u00ff"), true);
            "#,
        ),
        TestAction::inspect_context(|ctx| {
            use crate::fetch::headers::JsHeaders;
            use boa_engine::{js_str, value::TryFromJs};
            let value = ctx.global_object().get(js_str!("headers"), ctx).unwrap();
            let headers = JsHeaders::try_from_js(&value, ctx).unwrap();
            assert_eq!(
                headers.as_header_map().borrow()["x-test"].as_bytes(),
                b"1\xb5s\x80\xff"
            );
        }),
    ]);
}

#[test]
fn headers_invalid_inputs_throw_type_error_objects() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const cases = [
                    () => new Headers([["a\n", "b"]]),
                    () => new Headers([["x-test", "a\u0000b"]]),
                    () => new Headers([["x-test", "\u0100"]]),
                    () => new Headers({"x-test": "\u20ac"}),
                    () => {
                        const h = new Headers();
                        h.append("a\n", "b");
                    },
                    () => {
                        const h = new Headers();
                        h.set("x-test", "a\u0000b");
                    },
                    () => {
                        const h = new Headers();
                        h.append("x-test", "\ud83d\ude00");
                    },
                    () => {
                        const h = new Headers();
                        h.set("x-test", "\u0100");
                    },
                ];

                for (const make of cases) {
                    try {
                        make();
                        throw Error("expected the call above to throw");
                    } catch (e) {
                        assert(e instanceof TypeError, "should throw TypeError instance");
                        assert(typeof e.message === "string" && e.message.length > 0, "error message should be non-empty");
                    }

                }
            "#,
        ),
    ]);
}

#[test]
fn headers_iterator_throws_on_invalid_this() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                try {
                    const iterator = Headers.prototype[Symbol.iterator].call({});
                    throw Error("expected the call above to throw");
                } catch (e) {
                    assert(e instanceof TypeError);
                    assertEq(e.message, "`Headers.prototype[Symbol.iterator]` requires a `Headers` object");
                }

                try {
                    const iterator = Headers.prototype[Symbol.iterator].call(1);
                    throw Error("expected the call above to throw");
                } catch (e) {
                    assert(e instanceof TypeError);
                    assertEq(e.message, "`Headers.prototype[Symbol.iterator]` requires a `Headers` object");
                }
            "#,
        ),
    ]);
}
