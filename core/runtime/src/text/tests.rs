use crate::test::{TestAction, run_test_actions_with};
use crate::text;
use boa_engine::object::builtins::JsUint8Array;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsString, js_str, js_string};
use indoc::indoc;
use test_case::test_case;

#[test]
fn encoder_js() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const encoder = new TextEncoder();
                encoded = encoder.encode("Hello, World!");
            "#}),
            TestAction::inspect_context(|context| {
                let encoded = context
                    .global_object()
                    .get(js_str!("encoded"), context)
                    .unwrap();
                let array =
                    JsUint8Array::from_object(encoded.as_object().unwrap().clone()).unwrap();
                let buffer = array.iter(context).collect::<Vec<_>>();

                assert_eq!(buffer, b"Hello, World!");
            }),
        ],
        context,
    );
}

#[test]
fn encoder_js_unpaired() {
    use crate::test::{TestAction, run_test_actions_with};
    use indoc::indoc;

    let context = &mut Context::default();
    text::register(None, context).unwrap();

    let unpaired_surrogates: [u16; 3] = [0xDC58, 0xD83C, 0x0015];
    let text = JsString::from(&unpaired_surrogates);
    context
        .register_global_property(js_str!("text"), text, Attribute::default())
        .unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const encoder = new TextEncoder();
                encoded = encoder.encode(text);
            "#}),
            TestAction::inspect_context(|context| {
                let encoded = context
                    .global_object()
                    .get(js_str!("encoded"), context)
                    .unwrap();
                let array =
                    JsUint8Array::from_object(encoded.as_object().unwrap().clone()).unwrap();
                let buffer = array.iter(context).collect::<Vec<_>>();

                assert_eq!(buffer, "\u{FFFD}\u{FFFD}\u{15}".as_bytes());
            }),
        ],
        context,
    );
}

#[test]
fn decoder_js() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const d = new TextDecoder();
                decoded = d.decode(
                    Uint8Array.from([ 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33 ])
                );
            "#}),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("Hello, World!")));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_js_invalid() {
    use crate::test::{TestAction, run_test_actions_with};
    use indoc::indoc;

    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const d = new TextDecoder();
                decoded = d.decode(
                    Uint8Array.from([ 72, 101, 108, 108, 111, 160, 87, 111, 114, 108, 100, 161 ])
                );
            "#}),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                assert_eq!(
                    decoded.as_string(),
                    Some(js_string!("Hello\u{FFFD}World\u{FFFD}"))
                );
            }),
        ],
        context,
    );
}

#[test]
fn decoder_js_respects_typed_array_subarray() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const d = new TextDecoder();
                decoded = d.decode(Uint8Array.of(0x42, 0x42).subarray(1));
            "#}),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("B")));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_js_respects_typed_array_subarray_end() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const d = new TextDecoder();
                decoded = d.decode(Uint8Array.of(0x41, 0x42, 0x43).subarray(1, 2));
            "#}),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("B")));
            }),
        ],
        context,
    );
}

#[test_case("utf-8")]
#[test_case("utf-16")]
#[test_case("utf-16le")]
#[test_case("utf-16be")]
fn roundtrip(encoding: &'static str) {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(format!(
                r#"
                const encoder = new TextEncoder({encoding:?});
                const decoder = new TextDecoder({encoding:?});
                const text = "Hello, World!";
                const encoded = encoder.encode(text);
                decoded = decoder.decode(encoded);
            "#
            )),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("Hello, World!")));
            }),
        ],
        context,
    );
}
