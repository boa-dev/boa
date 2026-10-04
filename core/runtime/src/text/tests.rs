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
fn decoder_js_without_input() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const d = new TextDecoder();
                decoded = d.decode();
                decodedUndefined = d.decode(undefined);
            "#}),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                let decoded_undefined = context
                    .global_object()
                    .get(js_str!("decodedUndefined"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("")));
                assert_eq!(decoded_undefined.as_string(), Some(js_string!("")));
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
fn roundtrip_utf8() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const encoder = new TextEncoder();
                const decoder = new TextDecoder();
                const text = "Hello, World!";
                const encoded = encoder.encode(text);
                decoded = decoder.decode(encoded);
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

#[test_case("utf-8")]
#[test_case("utf-16")]
#[test_case("utf-16le")]
#[test_case("utf-16be")]
fn encoder_ignores_non_utf_encoding_arguments(encoding: &'static str) {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(format!(
                r#"
                const encoder = new TextEncoder({encoding:?});
                actualEncoding = encoder.encoding;
                encoded = encoder.encode("Hi");
            "#
            )),
            TestAction::inspect_context(|context| {
                let actual_encoding = context
                    .global_object()
                    .get(js_str!("actualEncoding"), context)
                    .unwrap();
                assert_eq!(actual_encoding.as_string(), Some(js_string!("utf-8")));

                let encoded = context
                    .global_object()
                    .get(js_str!("encoded"), context)
                    .unwrap();
                let array =
                    JsUint8Array::from_object(encoded.as_object().unwrap().clone()).unwrap();
                let buffer = array.iter(context).collect::<Vec<_>>();
                assert_eq!(buffer, b"Hi");
            }),
        ],
        context,
    );
}

// Default behavior: BOM is stripped
#[test_case("utf-8", &[0xEF, 0xBB, 0xBF, 72, 105])]
#[test_case("utf-16le", &[0xFF, 0xFE, 72, 0, 105, 0])]
#[test_case("utf-16be", &[0xFE, 0xFF, 0, 72, 0, 105])]
fn decoder_bom_default_stripped(encoding: &'static str, bytes: &'static [u8]) {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    let input = JsUint8Array::from_iter(bytes.iter().copied(), context).unwrap();
    context
        .register_global_property(js_str!("input"), input, Attribute::default())
        .unwrap();

    run_test_actions_with(
        [
            TestAction::run(format!(
                r#"
                const d = new TextDecoder({encoding:?});
                decoded = d.decode(input);
            "#
            )),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("Hi")));
            }),
        ],
        context,
    );
}

// ignoreBOM: true — BOM is kept in the output
#[test_case("utf-8", &[0xEF, 0xBB, 0xBF, 72, 105])]
#[test_case("utf-16le", &[0xFF, 0xFE, 72, 0, 105, 0])]
#[test_case("utf-16be", &[0xFE, 0xFF, 0, 72, 0, 105])]
fn decoder_bom_ignore_bom_true(encoding: &'static str, bytes: &'static [u8]) {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    let input = JsUint8Array::from_iter(bytes.iter().copied(), context).unwrap();
    context
        .register_global_property(js_str!("input"), input, Attribute::default())
        .unwrap();

    run_test_actions_with(
        [
            TestAction::run(format!(
                r#"
                const d = new TextDecoder({encoding:?}, {{ ignoreBOM: true }});
                decoded = d.decode(input);
            "#
            )),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("\u{FEFF}Hi")));
            }),
        ],
        context,
    );
}

// ignoreBOM: false — same as default, BOM is stripped
#[test_case("utf-8", &[0xEF, 0xBB, 0xBF, 72, 105])]
#[test_case("utf-16le", &[0xFF, 0xFE, 72, 0, 105, 0])]
#[test_case("utf-16be", &[0xFE, 0xFF, 0, 72, 0, 105])]
fn decoder_bom_ignore_bom_false(encoding: &'static str, bytes: &'static [u8]) {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    let input = JsUint8Array::from_iter(bytes.iter().copied(), context).unwrap();
    context
        .register_global_property(js_str!("input"), input, Attribute::default())
        .unwrap();

    run_test_actions_with(
        [
            TestAction::run(format!(
                r#"
                const d = new TextDecoder({encoding:?}, {{ ignoreBOM: false }});
                decoded = d.decode(input);
            "#
            )),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("Hi")));
            }),
        ],
        context,
    );
}

#[test_case("UTF-8", "utf-8"; "uppercase utf8")]
#[test_case(" utf-8 ", "utf-8"; "spaced utf8")]
#[test_case("\nutf-16\t", "utf-16le"; "spaced utf16")]
#[test_case("UTF-16BE", "utf-16be"; "uppercase utf16be")]
#[test_case("utf8", "utf-8"; "utf8 alias")]
#[test_case("Unicode-1-1-UTF-8", "utf-8"; "unicode alias")]
#[test_case("csUnicode", "utf-16le"; "csunicode alias")]
#[test_case(" unicodefeff ", "utf-16le"; "unicodefeff alias")]
#[test_case("UnicodeFFFE", "utf-16be"; "unicodefffe alias")]
fn decoder_normalizes_supported_labels(label: &'static str, expected: &'static str) {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(format!(
                r#"
                const d = new TextDecoder({label:?});
                encoding = d.encoding;
            "#
            )),
            TestAction::inspect_context(move |context| {
                let encoding = context
                    .global_object()
                    .get(js_str!("encoding"), context)
                    .unwrap();
                assert_eq!(encoding.as_string(), Some(JsString::from(expected)));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_rejects_unsupported_label_after_normalization() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
                try {
                    new TextDecoder(" utf-32 ");
                    throw new Error("expected RangeError");
                } catch (e) {
                    if (!(e instanceof RangeError)) {
                        throw e;
                    }
                }
            "#})],
        context,
    );
}

#[test]
fn decoder_ignore_bom_getter() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const d1 = new TextDecoder();
                const d2 = new TextDecoder("utf-8", { ignoreBOM: true });
                const d3 = new TextDecoder("utf-8", { ignoreBOM: false });
                ignoreBOM1 = d1.ignoreBOM;
                ignoreBOM2 = d2.ignoreBOM;
                ignoreBOM3 = d3.ignoreBOM;
            "#}),
            TestAction::inspect_context(|context| {
                let v1 = context
                    .global_object()
                    .get(js_str!("ignoreBOM1"), context)
                    .unwrap();
                let v2 = context
                    .global_object()
                    .get(js_str!("ignoreBOM2"), context)
                    .unwrap();
                let v3 = context
                    .global_object()
                    .get(js_str!("ignoreBOM3"), context)
                    .unwrap();
                assert_eq!(v1.as_boolean(), Some(false));
                assert_eq!(v2.as_boolean(), Some(true));
                assert_eq!(v3.as_boolean(), Some(false));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_handle_data_view() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                var decoded = new TextDecoder().decode(
                    new DataView(new TextEncoder().encode("hello").buffer)
                );
            "#}),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("hello")));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_handle_typed_array_offset_and_length() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                var decoded = new TextDecoder().decode(Uint8Array.of(0x41, 0x43, 0x45, 0x47).subarray(1, 3));
            "#}),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("CE")));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_handle_data_view_offset_and_length() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const buffer = Uint8Array.of(0x41, 0x43, 0x45, 0x47).buffer;
                const view = new DataView(buffer, 1, 2);
                var decoded = new TextDecoder().decode(view);
            "#}),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decoded"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("CE")));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_fatal_getter() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const d1 = new TextDecoder();
                const d2 = new TextDecoder("utf-8", { fatal: true });
                const d3 = new TextDecoder("utf-8", { fatal: false });
                fatal1 = d1.fatal;
                fatal2 = d2.fatal;
                fatal3 = d3.fatal;
                hasFatal = "fatal" in d1;
            "#}),
            TestAction::inspect_context(|context| {
                let v1 = context
                    .global_object()
                    .get(js_str!("fatal1"), context)
                    .unwrap();
                let v2 = context
                    .global_object()
                    .get(js_str!("fatal2"), context)
                    .unwrap();
                let v3 = context
                    .global_object()
                    .get(js_str!("fatal3"), context)
                    .unwrap();
                let has = context
                    .global_object()
                    .get(js_str!("hasFatal"), context)
                    .unwrap();
                assert_eq!(v1.as_boolean(), Some(false));
                assert_eq!(v2.as_boolean(), Some(true));
                assert_eq!(v3.as_boolean(), Some(false));
                assert_eq!(has.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_fatal_utf8_invalid_throws() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const decoder = new TextDecoder("utf-8", { fatal: true });
                try {
                    decoder.decode(Uint8Array.of(0xf0, 0x80, 0x80));
                    thrown = false;
                } catch (e) {
                    thrown = e instanceof TypeError;
                }
            "#}),
            TestAction::inspect_context(|context| {
                let thrown = context
                    .global_object()
                    .get(js_str!("thrown"), context)
                    .unwrap();
                assert_eq!(thrown.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_fatal_utf16le_invalid_throws() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const decoder = new TextDecoder("utf-16le", { fatal: true });
                try {
                    decoder.decode(new Uint8Array([0x00]));
                    thrown = false;
                } catch (e) {
                    thrown = e instanceof TypeError;
                }
            "#}),
            TestAction::inspect_context(|context| {
                let thrown = context
                    .global_object()
                    .get(js_str!("thrown"), context)
                    .unwrap();
                assert_eq!(thrown.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_fatal_utf16be_invalid_throws() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const decoder = new TextDecoder("utf-16be", { fatal: true });
                try {
                    decoder.decode(new Uint8Array([0x00]));
                    thrown = false;
                } catch (e) {
                    thrown = e instanceof TypeError;
                }
            "#}),
            TestAction::inspect_context(|context| {
                let thrown = context
                    .global_object()
                    .get(js_str!("thrown"), context)
                    .unwrap();
                assert_eq!(thrown.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_utf16_unaligned_buffer() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const buffer = new ArrayBuffer(8);
                const u8 = new Uint8Array(buffer, 1, 4);
                // "Hi" in UTF-16LE at unaligned offset: [72, 0, 105, 0]
                u8[0] = 72;
                u8[1] = 0;
                u8[2] = 105;
                u8[3] = 0;
                decodedLe = new TextDecoder("utf-16le").decode(u8);
            "#}),
            TestAction::inspect_context(|context| {
                let decoded = context
                    .global_object()
                    .get(js_str!("decodedLe"), context)
                    .unwrap();
                assert_eq!(decoded.as_string(), Some(js_string!("Hi")));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_utf16le_surrogates() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const decoder = new TextDecoder("utf-16le");
                res = decoder.decode(new Uint8Array([0x00, 0xd8]));
            "#}),
            TestAction::inspect_context(|context| {
                let res = context
                    .global_object()
                    .get(js_str!("res"), context)
                    .unwrap();
                assert_eq!(res.as_string(), Some(js_string!("\u{FFFD}")));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_utf16be_surrogates() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const decoder = new TextDecoder("utf-16be");
                res = decoder.decode(new Uint8Array([0xd8, 0x00]));
            "#}),
            TestAction::inspect_context(|context| {
                let res = context
                    .global_object()
                    .get(js_str!("res"), context)
                    .unwrap();
                assert_eq!(res.as_string(), Some(js_string!("\u{FFFD}")));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_fatal_utf16le_surrogates_throws() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const decoder = new TextDecoder("utf-16le", { fatal: true });
                try {
                    decoder.decode(new Uint8Array([0x00, 0xd8]));
                    thrown = false;
                } catch (e) {
                    thrown = e instanceof TypeError;
                }
            "#}),
            TestAction::inspect_context(|context| {
                let thrown = context
                    .global_object()
                    .get(js_str!("thrown"), context)
                    .unwrap();
                assert_eq!(thrown.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn decoder_fatal_utf16be_surrogates_throws() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                const decoder = new TextDecoder("utf-16be", { fatal: true });
                try {
                    decoder.decode(new Uint8Array([0xd8, 0x00]));
                    thrown = false;
                } catch (e) {
                    thrown = e instanceof TypeError;
                }
            "#}),
            TestAction::inspect_context(|context| {
                let thrown = context
                    .global_object()
                    .get(js_str!("thrown"), context)
                    .unwrap();
                assert_eq!(thrown.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}
