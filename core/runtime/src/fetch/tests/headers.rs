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
fn headers_invalid_inputs_throw_type_error_objects() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const cases = [
                    () => new Headers([["a\n", "b"]]),
                    () => new Headers([["x-test", "a\u0000b"]]),
                    () => {
                        const h = new Headers();
                        h.append("a\n", "b");
                    },
                    () => {
                        const h = new Headers();
                        h.set("x-test", "a\u0000b");
                    },
                ];

                for (const make of cases) {
                    let threw = false;
                    try {
                        make();
                        throw Error("expected the call above to throw");
                    } catch (e) {
                        threw = true;
                        assert(e instanceof TypeError, "should throw TypeError instance");
                        assert(typeof e.message === "string" && e.message.length > 0, "error message should be non-empty");
                    }

                    assert(threw, "expected Headers validation to throw");
                }
            "#,
        ),
    ]);
}
