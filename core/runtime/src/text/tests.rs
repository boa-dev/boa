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
fn decoder_utf16le_replaces_unpaired_surrogates() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::harness(),
            TestAction::run(indoc! {r#"
                const invalid16 = [
                    { invalid: [0x61, 0x62, 0xd8_00, 0x77, 0x78], replaced: [0x61, 0x62, 0xff_fd, 0x77, 0x78] },
                    { invalid: [0xd8_00], replaced: [0xff_fd] },
                    { invalid: [0xd8_00, 0xd8_00], replaced: [0xff_fd, 0xff_fd] },
                    { invalid: [0x61, 0x62, 0xdf_ff, 0x77, 0x78], replaced: [0x61, 0x62, 0xff_fd, 0x77, 0x78] },
                    { invalid: [0xdf_ff, 0xd8_00], replaced: [0xff_fd, 0xff_fd] },
                ];

                const d = new TextDecoder("utf-16le");
                for (const { invalid, replaced } of invalid16) {
                    const input = new Uint8Array(invalid.length * 2);
                    for (let i = 0; i < invalid.length; i++) {
                        input[2 * i] = invalid[i] & 0xff;
                        input[2 * i + 1] = invalid[i] >> 8;
                    }
                    assertEq(d.decode(input), String.fromCharCode(...replaced));
                }
            "#}),
        ],
        context,
    );
}

#[test]
fn decoder_utf16be_replaces_unpaired_surrogates() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::harness(),
            TestAction::run(indoc! {r#"
                const invalid16 = [
                    { invalid: [0x61, 0x62, 0xd8_00, 0x77, 0x78], replaced: [0x61, 0x62, 0xff_fd, 0x77, 0x78] },
                    { invalid: [0xd8_00], replaced: [0xff_fd] },
                    { invalid: [0xd8_00, 0xd8_00], replaced: [0xff_fd, 0xff_fd] },
                    { invalid: [0x61, 0x62, 0xdf_ff, 0x77, 0x78], replaced: [0x61, 0x62, 0xff_fd, 0x77, 0x78] },
                    { invalid: [0xdf_ff, 0xd8_00], replaced: [0xff_fd, 0xff_fd] },
                ];

                const d = new TextDecoder("utf-16be");
                for (const { invalid, replaced } of invalid16) {
                    const input = new Uint8Array(invalid.length * 2);
                    for (let i = 0; i < invalid.length; i++) {
                        input[2 * i] = invalid[i] >> 8;
                        input[2 * i + 1] = invalid[i] & 0xff;
                    }
                    assertEq(d.decode(input), String.fromCharCode(...replaced));
                }
            "#}),
        ],
        context,
    );
}

#[test]
fn decoder_utf16le_does_not_overproduce_on_truncation() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::harness(),
            TestAction::run(indoc! {r#"
                const d = new TextDecoder("utf-16le");
                assertEq(d.decode(Uint8Array.of(0, 0, 0)), "\0\uFFFD");
                assertEq(d.decode(Uint8Array.of(42, 0, 0)), "*\uFFFD");
                assertEq(d.decode(Uint8Array.of(0, 0xd8, 0)), "\uFFFD");
                assertEq(d.decode(Uint8Array.of(0, 0xd8, 0xd8)), "\uFFFD");
            "#}),
        ],
        context,
    );
}

#[test]
fn decoder_utf16be_does_not_overproduce_on_truncation() {
    let context = &mut Context::default();
    text::register(None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::harness(),
            TestAction::run(indoc! {r#"
                const d = new TextDecoder("utf-16be");
                assertEq(d.decode(Uint8Array.of(0, 0, 0)), "\0\uFFFD");
                assertEq(d.decode(Uint8Array.of(0, 42, 0)), "*\uFFFD");
                assertEq(d.decode(Uint8Array.of(0xd8, 0, 0)), "\uFFFD");
                assertEq(d.decode(Uint8Array.of(0xd8, 0, 0xd8)), "\uFFFD");
            "#}),
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
