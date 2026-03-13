use crate::test::{TestAction, run_test_actions};

const TEST_HARNESS: &str = r#"
function assert(condition, message) {
    if (!condition) {
        if (!message) {
            message = "Assertion failed";
        }
        throw new Error(message);
    }
}

function assert_eq(a, b, message) {
    if (a !== b) {
        throw new Error(`${message} (${JSON.stringify(a)} !== ${JSON.stringify(b)})`);
    }
}
"#;

#[test]
fn url_basic() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                url = new URL("https://example.com:8080/path/to/resource?query#fragment");
                assert(url instanceof URL);
                assert_eq(url.href, "https://example.com:8080/path/to/resource?query#fragment");
                assert_eq(url.protocol, "https:");
                assert_eq(url.host, "example.com:8080");
                assert_eq(url.hostname, "example.com");
                assert_eq(url.port, "8080");
                assert_eq(url.pathname, "/path/to/resource");
                assert_eq(url.search, "?query");
                assert_eq(url.hash, "#fragment");
            "##,
        ),
    ]);
}

#[test]
fn url_base() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                url = new URL("https://example.com:8080/path/to/resource?query#fragment", "http://example.org/");
                assert_eq(url.href, "https://example.com:8080/path/to/resource?query#fragment");
                assert_eq(url.protocol, "https:");
                assert_eq(url.host, "example.com:8080");
                assert_eq(url.hostname, "example.com");
                assert_eq(url.port, "8080");
                assert_eq(url.pathname, "/path/to/resource");
                assert_eq(url.search, "?query");
                assert_eq(url.hash, "#fragment");
            "##,
        ),
        TestAction::run(
            r##"
                url = new URL("/path/to/resource?query#fragment", "http://example.org/");
                assert_eq(url.href, "http://example.org/path/to/resource?query#fragment");
                assert_eq(url.protocol, "http:");
                assert_eq(url.host, "example.org");
                assert_eq(url.hostname, "example.org");
                assert_eq(url.port, "");
                assert_eq(url.pathname, "/path/to/resource");
                assert_eq(url.search, "?query");
                assert_eq(url.hash, "#fragment");
            "##,
        ),
        TestAction::run(
            r##"
                url = new URL("http://example.org/new/path?new-query#new-fragment", "about:blank");
                assert_eq(url.href, "http://example.org/new/path?new-query#new-fragment");
            "##,
        ),
    ]);
}

#[test]
fn url_setters() {
    // These were double checked against Firefox.
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                url = new URL("https://example.com:8080/path/to/resource?query#fragment");
                url.protocol = "http:";
                url.host = "example.org:80"; // Since protocol is http, port is removed.
                url.pathname = "/new/path";
                url.search = "?new-query";
                url.hash = "#new-fragment";
                assert_eq(url.href, "http://example.org/new/path?new-query#new-fragment");
                assert_eq(url.protocol, "http:");
                assert_eq(url.host, "example.org");
                assert_eq(url.hostname, "example.org");
                assert_eq(url.port, "");
                assert_eq(url.pathname, "/new/path");
                assert_eq(url.search, "?new-query");
                assert_eq(url.hash, "#new-fragment");
            "##,
        ),
    ]);
}

#[test]
fn url_static_methods() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                assert(URL.canParse("http://example.org/new/path?new-query#new-fragment"));
                assert(!URL.canParse("http//:example.org/new/path?new-query#new-fragment"));
                assert(!URL.canParse("http://example.org/new/path?new-query#new-fragment", "http:"));
                assert(URL.canParse("/new/path?new-query#new-fragment", "http://example.org/"));
                assert(URL.canParse("http://example.org/new/path?new-query#new-fragment", "about:blank"));
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_constructor_and_methods() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const params = new URLSearchParams([["b", "2"], ["a", "1"], ["a", "3"]]);
                assert_eq(params.size, 3);
                assert_eq(params.get("a"), "1");
                assert_eq(params.get("missing"), null);
                assert_eq(params.getAll("a").join(","), "1,3");
                assert(params.has("b"));
                assert(!params.has("b", "4"));

                params.append("c", "5");
                params.delete("a", "1");
                params.set("b", "4");
                params.sort();

                assert_eq(params.toString(), "a=3&b=4&c=5");
                assert_eq([...params].map(([k, v]) => `${k}=${v}`).join("&"), "a=3&b=4&c=5");
            "##,
        ),
        TestAction::run(
            r##"
                const fromObject = new URLSearchParams({ a: "1", b: "2" });
                assert_eq(fromObject.toString(), "a=1&b=2");

                const fromString = new URLSearchParams("?x=1&x=2");
                assert_eq(fromString.getAll("x").join(","), "1,2");
            "##,
        ),
        TestAction::run(
            r##"
                const fromIterableObject = new URLSearchParams({
                    *[Symbol.iterator]() {
                        yield ["i", "1"];
                        yield new Set(["j", "2"]);
                    },
                    ignored: "value",
                });
                assert_eq(fromIterableObject.toString(), "i=1&j=2");
            "##,
        ),
        TestAction::run(
            r##"
                const record = Object.create({ inherited: "x" });
                Object.defineProperty(record, "hidden", { value: "2", enumerable: false });
                record.a = "1";
                assert_eq(new URLSearchParams(record).toString(), "a=1");
            "##,
        ),
        TestAction::run(
            r##"
                const originalFrom = Array.from;
                Array.from = () => {
                    throw new Error("patched");
                };

                try {
                    const params = new URLSearchParams([["a", "1"], new Set(["b", "2"])]);
                    assert_eq(params.toString(), "a=1&b=2");
                } finally {
                    Array.from = originalFrom;
                }
            "##,
        ),
        TestAction::run(
            r##"
                const customParams = new URLSearchParams("x=1");
                customParams[Symbol.iterator] = function* () {
                    yield ["a", "b"];
                };

                assert_eq(new URLSearchParams(customParams).toString(), "a=b");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_is_live_and_cached() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const url = new URL("https://example.com/?a=1&b=2");
                const params1 = url.searchParams;
                const params2 = url.searchParams;

                assert(params1 === params2, "searchParams must be the same object");
                assert_eq(params1.get("a"), "1");

                params1.append("c", "3");
                assert_eq(url.search, "?a=1&b=2&c=3");

                url.search = "?x=9";
                assert_eq(params1.get("x"), "9");
                assert_eq(params1.get("a"), null);

                params1.delete("x");
                assert_eq(url.href, "https://example.com/");
            "##,
        ),
        TestAction::run(
            r##"
                const liveUrl = new URL("http://a.b/c?a=1&b=2&c=3");
                const liveSeen = [];

                for (const entry of liveUrl.searchParams) {
                    if (entry[0] === "a") {
                        liveUrl.search = "x=1&y=2&z=3";
                    }
                    liveSeen.push(entry.join("="));
                }

                assert_eq(liveSeen.join("&"), "a=1&y=2&z=3");
            "##,
        ),
        TestAction::run(
            r##"
                const forEachUrl = new URL("http://localhost/query?param0=0&param1=1&param2=2");
                const forEachSeen = [];

                forEachUrl.searchParams.forEach((value, key) => {
                    forEachSeen.push(`${key}=${value}`);
                    if (key === "param0") {
                        forEachUrl.searchParams.delete("param1");
                    }
                });

                assert_eq(forEachSeen.join("&"), "param0=0&param2=2");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_optional_value_argument() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const params = new URLSearchParams("a=b&a=undefined&b=c");

                assert(params.has("a", undefined));
                assert(!params.has("a", "missing"));

                params.delete("a", undefined);
                assert_eq(params.toString(), "a=b&b=c");
            "##,
        ),
        TestAction::run(
            r##"
                const deleteAllParams = new URLSearchParams("a=b&a=undefined&a=d");
                deleteAllParams.delete("a");
                assert_eq(deleteAllParams.toString(), "");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_constructor_errors() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                var threw = false;
                try {
                    new URLSearchParams(new String("ab"));
                } catch (e) {
                    threw = e instanceof TypeError;
                }
                assert(threw, "string wrapper objects must use the iterable branch");
            "##,
        ),
        TestAction::run(
            r##"
                var threw = false;
                try {
                    new URLSearchParams([[1, 2, 3]]);
                } catch (e) {
                    threw = e instanceof TypeError;
                }
                assert(threw, "pairs with length != 2 must throw");
            "##,
        ),
        TestAction::run(
            r##"
                var threw = false;
                try {
                    new URLSearchParams({ a: "1", [Symbol.iterator]: 1 });
                } catch (e) {
                    threw = e instanceof TypeError;
                }
                assert(threw, "non-callable @@iterator must throw");
            "##,
        ),
        TestAction::run(
            r##"
                const key = Symbol("k");
                var threw = false;
                try {
                    new URLSearchParams({ [key]: "1", a: "2" });
                } catch (e) {
                    threw = e instanceof TypeError;
                }
                assert(threw, "enumerable symbol keys must throw during record conversion");
            "##,
        ),
    ]);
}
