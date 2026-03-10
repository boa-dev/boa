use crate::test::{TestAction, run_test_actions_with};
use boa_engine::Context;
use indoc::indoc;

#[test]
fn btoa_basic() {
    let context = &mut Context::default();
    crate::base64::register(None, context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            if (btoa("hello") !== "aGVsbG8=") {
                throw new Error("btoa('hello') failed: " + btoa("hello"));
            }
            if (btoa("") !== "") {
                throw new Error("btoa('') failed");
            }
            if (btoa("a") !== "YQ==") {
                throw new Error("btoa('a') failed: " + btoa("a"));
            }
            if (btoa("ab") !== "YWI=") {
                throw new Error("btoa('ab') failed: " + btoa("ab"));
            }
            if (btoa("abc") !== "YWJj") {
                throw new Error("btoa('abc') failed: " + btoa("abc"));
            }
        "#})],
        context,
    );
}

#[test]
fn atob_basic() {
    let context = &mut Context::default();
    crate::base64::register(None, context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            if (atob("aGVsbG8=") !== "hello") {
                throw new Error("atob('aGVsbG8=') failed: " + atob("aGVsbG8="));
            }
            if (atob("") !== "") {
                throw new Error("atob('') failed");
            }
            if (atob("YQ==") !== "a") {
                throw new Error("atob('YQ==') failed");
            }
            if (atob("YWI=") !== "ab") {
                throw new Error("atob('YWI=') failed");
            }
            if (atob("YWJj") !== "abc") {
                throw new Error("atob('YWJj') failed");
            }
        "#})],
        context,
    );
}

#[test]
fn roundtrip() {
    let context = &mut Context::default();
    crate::base64::register(None, context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            var inputs = ["", "a", "ab", "abc", "hello", "Hello, World!", "\x80\xFF", "caf\xE9"];
            for (var i = 0; i < inputs.length; i++) {
                if (atob(btoa(inputs[i])) !== inputs[i]) {
                    throw new Error("roundtrip failed for input at index " + i);
                }
            }
        "#})],
        context,
    );
}

#[test]
fn btoa_throws_on_non_latin1() {
    let context = &mut Context::default();
    crate::base64::register(None, context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            var threw = false;
            try {
                btoa("\u{2713}");
            } catch (e) {
                threw = true;
                var msg = String(e);
                if (msg.indexOf("InvalidCharacterError") === -1) {
                    throw new Error("Expected InvalidCharacterError, got: " + msg);
                }
            }
            if (!threw) {
                throw new Error("btoa should throw on non-Latin1 characters");
            }
        "#})],
        context,
    );
}

#[test]
fn atob_throws_on_invalid_input() {
    let context = &mut Context::default();
    crate::base64::register(None, context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            function expectThrow(input, label) {
                var threw = false;
                try {
                    atob(input);
                } catch (e) {
                    threw = true;
                }
                if (!threw) {
                    throw new Error("atob should throw on " + label + ": " + input);
                }
            }

            expectThrow("a", "length % 4 == 1");
            expectThrow("!!!!", "invalid characters");
            expectThrow("YQ=a", "misplaced padding");
        "#})],
        context,
    );
}

#[test]
fn atob_ignores_whitespace() {
    let context = &mut Context::default();
    crate::base64::register(None, context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            if (atob(" aGVs bG8= ") !== "hello") {
                throw new Error("atob should ignore spaces");
            }
            if (atob("\taGVs\tbG8=\t") !== "hello") {
                throw new Error("atob should ignore tabs");
            }
            if (atob("\naGVs\nbG8=\n") !== "hello") {
                throw new Error("atob should ignore newlines");
            }
            if (atob("\x0CaGVs\x0CbG8=\x0C") !== "hello") {
                throw new Error("atob should ignore form feeds");
            }
            if (atob("\raGVs\rbG8=\r") !== "hello") {
                throw new Error("atob should ignore carriage returns");
            }
        "#})],
        context,
    );
}
