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
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_basic() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const params = new URLSearchParams("foo=1&bar=2&foo=3");
                assert_eq(params.get("foo"), "1", "get first foo");
                assert_eq(params.get("bar"), "2", "get bar");
                assert_eq(params.get("missing"), null, "get missing");
                assert_eq(params.size, 3, "size");
                assert_eq(params.toString(), "foo=1&bar=2&foo=3", "toString");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_constructor_leading_question_mark() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const params = new URLSearchParams("?a=1&b=2");
                assert_eq(params.get("a"), "1", "leading ? stripped");
                assert_eq(params.get("b"), "2", "second param");
                assert_eq(params.size, 2, "size");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_append_delete() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const params = new URLSearchParams();
                params.append("a", "1");
                params.append("b", "2");
                params.append("a", "3");
                assert_eq(params.size, 3, "size after appends");
                assert_eq(params.toString(), "a=1&b=2&a=3", "toString after appends");

                params.delete("a");
                assert_eq(params.size, 1, "size after delete");
                assert_eq(params.get("a"), null, "a deleted");
                assert_eq(params.get("b"), "2", "b still present");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_has() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const params = new URLSearchParams("a=1&b=2&a=3");
                assert(params.has("a"), "has a");
                assert(params.has("b"), "has b");
                assert(!params.has("c"), "no c");
                assert(params.has("a", "1"), "has a=1");
                assert(!params.has("a", "999"), "no a=999");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_set() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const params = new URLSearchParams("a=1&b=2&a=3");
                params.set("a", "999");
                assert_eq(params.get("a"), "999", "set replaces first");
                assert_eq(params.size, 2, "duplicates removed");
                assert_eq(params.toString(), "a=999&b=2", "toString after set");

                params.set("c", "new");
                assert_eq(params.get("c"), "new", "set adds new");
                assert_eq(params.size, 3, "size after set new");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_sort() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const params = new URLSearchParams("c=3&a=1&b=2");
                params.sort();
                assert_eq(params.toString(), "a=1&b=2&c=3", "sorted");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_get_all() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const params = new URLSearchParams("a=1&b=2&a=3");
                const all = params.getAll("a");
                assert_eq(all.length, 2, "getAll length");
                assert_eq(all[0], "1", "getAll[0]");
                assert_eq(all[1], "3", "getAll[1]");

                const empty = params.getAll("missing");
                assert_eq(empty.length, 0, "getAll missing");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_for_each() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const params = new URLSearchParams("a=1&b=2");
                const collected = [];
                params.forEach(function(value, name) {
                    collected.push(name + "=" + value);
                });
                assert_eq(collected.length, 2, "forEach count");
                assert_eq(collected[0], "a=1", "forEach[0]");
                assert_eq(collected[1], "b=2", "forEach[1]");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_from_url() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const url = new URL("https://example.com?foo=1&bar=2");
                const params = url.searchParams;
                assert_eq(params.get("foo"), "1", "foo from URL");
                assert_eq(params.get("bar"), "2", "bar from URL");
                assert_eq(params.size, 2, "size from URL");
            "##,
        ),
    ]);
}

#[test]
fn url_search_params_encoding() {
    run_test_actions([
        TestAction::run(TEST_HARNESS),
        TestAction::run(
            r##"
                const params = new URLSearchParams();
                params.append("key with spaces", "value&special=chars");
                assert_eq(
                    params.toString(),
                    "key+with+spaces=value%26special%3Dchars",
                    "URL-encoded"
                );
            "##,
        ),
    ]);
}
