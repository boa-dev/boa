use crate::test::{run_test_actions, TestAction};

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
