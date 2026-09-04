#![allow(clippy::redundant_clone)]

use std::hash::{BuildHasher, BuildHasherDefault, Hash};

use crate::{
    CodePoint, CommonJsStringBuilder, JsStr, JsString, JsStringKind, Latin1JsStringBuilder,
    StaticJsStrings, StaticString, Utf16JsStringBuilder,
};

use rustc_hash::FxHasher;

fn hash_value<T: Hash>(value: &T) -> u64 {
    BuildHasherDefault::<FxHasher>::default().hash_one(value)
}

const fn ascii_to_utf16<const LEN: usize>(ascii: &[u8; LEN]) -> [u16; LEN] {
    let mut array = [0; LEN];
    let mut i = 0;
    while i < LEN {
        array[i] = ascii[i] as u16;
        i += 1;
    }
    array
}

#[test]
fn empty() {
    let s = StaticJsStrings::EMPTY_STRING;
    assert_eq!(&s, &[]);
}

#[test]
fn refcount() {
    let x = JsString::from("Hello world");
    assert_eq!(x.refcount(), Some(1));

    {
        let y = x.clone();
        assert_eq!(x.refcount(), Some(2));
        assert_eq!(y.refcount(), Some(2));

        {
            let z = y.clone();
            assert_eq!(x.refcount(), Some(3));
            assert_eq!(y.refcount(), Some(3));
            assert_eq!(z.refcount(), Some(3));
        }

        assert_eq!(x.refcount(), Some(2));
        assert_eq!(y.refcount(), Some(2));
    }

    assert_eq!(x.refcount(), Some(1));
}

#[test]
fn static_refcount() {
    let x = StaticJsStrings::EMPTY_STRING;
    assert_eq!(x.refcount(), None);

    {
        let y = x.clone();
        assert_eq!(x.refcount(), None);
        assert_eq!(y.refcount(), None);
    };

    assert_eq!(x.refcount(), None);
}

#[test]
fn ptr_eq() {
    let x = JsString::from("Hello");
    let y = x.clone();

    assert!(!x.is_static());

    assert_eq!(x.ptr.addr(), y.ptr.addr());

    let z = JsString::from("Hello");
    assert_ne!(x.ptr.addr(), z.ptr.addr());
    assert_ne!(y.ptr.addr(), z.ptr.addr());
}

#[test]
fn static_ptr_eq() {
    let x = StaticJsStrings::EMPTY_STRING;
    let y = x.clone();

    assert!(x.is_static());

    assert_eq!(x.ptr.addr(), y.ptr.addr());

    let z = StaticJsStrings::EMPTY_STRING;
    assert_eq!(x.ptr.addr(), z.ptr.addr());
    assert_eq!(y.ptr.addr(), z.ptr.addr());
}

#[test]
fn as_str() {
    const HELLO: &[u16] = &ascii_to_utf16(b"Hello");
    let x = JsString::from(HELLO);

    assert_eq!(&x, HELLO);
}

#[test]
fn hash() {
    const HELLOWORLD: JsStr<'_> = JsStr::latin1("Hello World!".as_bytes());
    let x = JsString::from(HELLOWORLD);

    assert_eq!(x.as_str(), HELLOWORLD);

    assert!(HELLOWORLD.is_latin1());
    assert!(x.as_str().is_latin1());

    let s_hash = hash_value(&HELLOWORLD);
    let x_hash = hash_value(&x);

    assert_eq!(s_hash, x_hash);
}

#[test]
fn concat() {
    const Y: &[u16] = &ascii_to_utf16(b", ");
    const W: &[u16] = &ascii_to_utf16(b"!");

    let x = JsString::from("hello");
    let z = JsString::from("world");

    let xy = JsString::concat(x.as_str(), JsString::from(Y).as_str());
    assert_eq!(&xy, &ascii_to_utf16(b"hello, "));
    assert_eq!(xy.refcount(), Some(1));

    let xyz = JsString::concat(xy.as_str(), z.as_str());
    assert_eq!(&xyz, &ascii_to_utf16(b"hello, world"));
    assert_eq!(xyz.refcount(), Some(1));

    let xyzw = JsString::concat(xyz.as_str(), JsString::from(W).as_str());
    assert_eq!(&xyzw, &ascii_to_utf16(b"hello, world!"));
    assert_eq!(xyzw.refcount(), Some(1));
}

#[test]
fn trim_start_non_ascii_to_ascii() {
    let s = "\u{2029}abc";
    let x = JsString::from(s);

    let y = x.trim_start();

    assert_eq!(&y, s.trim_start());
}

#[test]
fn conversion_to_known_static_js_string() {
    const JS_STR_U8: &JsStr<'_> = &JsStr::latin1("length".as_bytes());
    const JS_STR_U16: &JsStr<'_> = &JsStr::utf16(&ascii_to_utf16(b"length"));

    assert!(JS_STR_U8.is_latin1());
    assert!(!JS_STR_U16.is_latin1());

    assert_eq!(JS_STR_U8, JS_STR_U8);
    assert_eq!(JS_STR_U16, JS_STR_U16);

    assert_eq!(JS_STR_U8, JS_STR_U16);
    assert_eq!(JS_STR_U16, JS_STR_U8);

    assert_eq!(hash_value(JS_STR_U8), hash_value(JS_STR_U16));

    let string = StaticJsStrings::get_string(JS_STR_U8);

    assert!(string.is_some());
    assert!(string.unwrap().as_str().is_latin1());

    let string = StaticJsStrings::get_string(JS_STR_U16);

    assert!(string.is_some());
    assert!(string.unwrap().as_str().is_latin1());
}

#[test]
fn to_std_string_escaped() {
    assert_eq!(
        JsString::from("Hello, \u{1D49E} world!").to_std_string_escaped(),
        "Hello, \u{1D49E} world!"
    );

    assert_eq!(
        JsString::from("Hello, world!").to_std_string_escaped(),
        "Hello, world!"
    );

    // 15 should not be escaped.
    let unpaired_surrogates: [u16; 3] = [0xDC58, 0xD83C, 0x0015];
    assert_eq!(
        JsString::from(&unpaired_surrogates).to_std_string_escaped(),
        "\\uDC58\\uD83C\u{15}"
    );
}

#[test]
fn from_static_js_string() {
    static STATIC_HELLO_WORLD: StaticString =
        StaticString::new(JsStr::latin1("hello world".as_bytes()));
    static STATIC_EMOJIS: StaticString = StaticString::new(JsStr::utf16(&[
        0xD83C, 0xDFB9, 0xD83C, 0xDFB6, 0xD83C, 0xDFB5,
    ])); // 🎹🎶🎵

    let latin1 = JsString::from_static(&STATIC_HELLO_WORLD);
    let utf16 = JsString::from_static(&STATIC_EMOJIS);

    // content compare
    assert_eq!(latin1, "hello world");
    assert_eq!(utf16, "🎹🎶🎵");

    // refcount check
    let clone = latin1.clone();

    assert_eq!(clone, latin1);

    let clone = utf16.clone();

    assert_eq!(clone, utf16);

    assert!(latin1.refcount().is_none());
    assert!(utf16.refcount().is_none());

    // `is_latin1` check
    assert!(latin1.as_str().is_latin1());
    assert!(!utf16.as_str().is_latin1());
}

#[test]
fn compare_static_and_dynamic_js_string() {
    static STATIC_HELLO_WORLD: StaticString =
        StaticString::new(JsStr::latin1("hello world".as_bytes()));
    static STATIC_EMOJIS: StaticString = StaticString::new(JsStr::utf16(&[
        0xD83C, 0xDFB9, 0xD83C, 0xDFB6, 0xD83C, 0xDFB5,
    ])); // 🎹🎶🎵

    let static_latin1 = JsString::from_static(&STATIC_HELLO_WORLD);
    let static_utf16 = JsString::from_static(&STATIC_EMOJIS);

    let dynamic_latin1 = JsString::from(JsStr::latin1("hello world".as_bytes()));
    let dynamic_utf16 = JsString::from(&[0xD83C, 0xDFB9, 0xD83C, 0xDFB6, 0xD83C, 0xDFB5]);

    // content compare
    assert_eq!(static_latin1, dynamic_latin1);
    assert_eq!(static_utf16, dynamic_utf16);

    // length check
    assert_eq!(static_latin1.len(), dynamic_latin1.len());
    assert_eq!(static_utf16.len(), dynamic_utf16.len());

    // `is_static` check
    assert!(static_latin1.is_static());
    assert!(static_utf16.is_static());
    assert!(!dynamic_latin1.is_static());
    assert!(!dynamic_utf16.is_static());
}

#[test]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::undocumented_unsafe_blocks)]
fn js_string_builder() {
    let s = "2024年5月21日";
    let utf16 = s.encode_utf16().collect::<Vec<_>>();
    let s_utf16 = utf16.as_slice();
    let ascii = "Lorem ipsum dolor sit amet";
    let s_ascii = ascii.as_bytes();
    let latin1_as_utf8_literal = "Déjà vu";
    let s_latin1_literal: &[u8] = &[
        b'D', 0xE9, /* é */
        b'j', 0xE0, /* à */
        b' ', b'v', b'u',
    ];

    // latin1 builder -- test

    // push ascii
    let mut builder = Latin1JsStringBuilder::new();
    for &code in s_ascii {
        builder.push(code);
    }
    let s_builder = builder.build().unwrap_or_default();
    assert_eq!(s_builder, ascii);

    // push latin1
    let mut builder = Latin1JsStringBuilder::new();
    for &code in s_latin1_literal {
        builder.push(code);
    }
    let s_builder = unsafe { builder.build_as_latin1() };
    assert_eq!(
        s_builder.to_std_string().unwrap_or_default(),
        latin1_as_utf8_literal
    );

    // from_iter ascii
    let s_builder = s_ascii
        .iter()
        .copied()
        .collect::<Latin1JsStringBuilder>()
        .build()
        .unwrap_or_default();
    assert_eq!(s_builder.to_std_string().unwrap_or_default(), ascii);

    // from_iter latin1
    let s_builder = unsafe {
        s_latin1_literal
            .iter()
            .copied()
            .collect::<Latin1JsStringBuilder>()
            .build_as_latin1()
    };
    assert_eq!(
        s_builder.to_std_string().unwrap_or_default(),
        latin1_as_utf8_literal
    );

    // extend_from_slice ascii
    let mut builder = Latin1JsStringBuilder::new();
    builder.extend_from_slice(s_ascii);
    let s_builder = builder.build().unwrap_or_default();
    assert_eq!(s_builder.to_std_string().unwrap_or_default(), ascii);

    // extend_from_slice latin1
    let mut builder = Latin1JsStringBuilder::new();
    builder.extend_from_slice(s_latin1_literal);
    let s_builder = unsafe { builder.build_as_latin1() };
    assert_eq!(
        s_builder.to_std_string().unwrap_or_default(),
        latin1_as_utf8_literal
    );

    // build from utf16 encoded string
    let s_builder = s
        .as_bytes()
        .iter()
        .copied()
        .collect::<Latin1JsStringBuilder>()
        .build();
    assert_eq!(None, s_builder);

    let s_builder = s_utf16
        .iter()
        .copied()
        .map(|v| v as u8)
        .collect::<Latin1JsStringBuilder>()
        .build();
    assert_eq!(None, s_builder);

    // utf16 builder -- test

    // push
    let mut builder = Utf16JsStringBuilder::new();
    for &code in s_utf16 {
        builder.push(code);
    }
    let s_builder = builder.build();
    assert_eq!(s_builder.to_std_string().unwrap_or_default(), s);

    // from_iter
    let s_builder = s_utf16
        .iter()
        .copied()
        .collect::<Utf16JsStringBuilder>()
        .build();
    assert_eq!(s_builder.to_std_string().unwrap_or_default(), s);

    // extend_from_slice
    let mut builder = Utf16JsStringBuilder::new();
    builder.extend_from_slice(s_utf16);
    let s_builder = builder.build();
    assert_eq!(s_builder.to_std_string().unwrap_or_default(), s);
}

#[test]
fn clone_builder() {
    // latin1 builder -- test
    let origin = Latin1JsStringBuilder::from(&b"0123456789"[..]);
    let empty_origin = Latin1JsStringBuilder::new();

    // clone == origin
    let cloned = origin.clone();
    assert_eq!(origin, cloned);

    // clone_from == origin
    let mut cloned_from = Latin1JsStringBuilder::new();
    cloned_from.clone_from(&origin);
    assert_eq!(origin, cloned_from);

    // clone == origin(empty)
    let cloned = empty_origin.clone();
    assert_eq!(empty_origin, cloned);

    // clone_from == origin(empty)

    cloned_from.clone_from(&empty_origin);
    assert!(cloned_from.capacity() > 0); // Should not be reallocated so the capacity is preserved.
    assert_eq!(empty_origin, cloned_from);

    // clone_from(empty) == origin(empty)
    let mut cloned_from = Latin1JsStringBuilder::new();
    cloned_from.clone_from(&empty_origin);
    assert_eq!(cloned_from.capacity(), 0);
    assert_eq!(empty_origin, cloned_from);

    // utf16 builder -- test
    let s = "2024年5月21日";

    let origin = Utf16JsStringBuilder::from(s.encode_utf16().collect::<Vec<_>>().as_slice());
    let empty_origin = Utf16JsStringBuilder::new();
    // clone == origin
    let cloned = origin.clone();
    assert_eq!(origin, cloned);

    // clone_from == origin(empty)
    let mut cloned_from = Utf16JsStringBuilder::new();
    cloned_from.clone_from(&origin);

    assert_eq!(origin, cloned_from);
    // clone == origin(empty)
    let cloned = empty_origin.clone();
    assert_eq!(empty_origin, cloned);

    // clone_from == origin(empty)

    cloned_from.clone_from(&empty_origin);
    assert!(cloned_from.capacity() > 0); // should not be reallocated so the capacity is preserved.
    assert_eq!(empty_origin, cloned_from);

    // clone_from(empty) == origin(empty)
    let mut cloned_from = Utf16JsStringBuilder::new();
    cloned_from.clone_from(&empty_origin);
    assert_eq!(cloned_from.capacity(), 0);
    assert_eq!(empty_origin, cloned_from);
}

#[test]
fn common_js_string_builder() {
    let utf16 = "2024年5月21日".encode_utf16().collect::<Vec<_>>();
    let s_utf16 = utf16.as_slice();
    let s = "Lorem ipsum dolor sit amet";
    let js_str_utf16 = JsStr::utf16(s_utf16);
    let js_str_ascii = JsStr::latin1(s.as_bytes());
    let latin1_bytes = [
        b'D', 0xE9, /* é */
        b'j', 0xE0, /* à */
        b' ', b'v', b'u',
    ];
    let ch = '🎹';
    let mut builder = CommonJsStringBuilder::with_capacity(10);
    builder += ch;
    builder += s;
    builder += js_str_utf16;
    builder += js_str_ascii;
    builder += ch;
    assert_eq!(builder.len(), 5);
    let js_string = builder.build_from_utf16();
    assert_eq!(
        js_string,
        "🎹Lorem ipsum dolor sit amet2024年5月21日Lorem ipsum dolor sit amet🎹"
    );
    let mut builder = CommonJsStringBuilder::new();
    for b in latin1_bytes {
        builder += b;
    }
    builder += s_utf16;
    builder += ch;
    let js_string = builder.build();
    assert_eq!(
        js_string.to_std_string().unwrap_or_default(),
        "Déjà vu2024年5月21日🎹"
    );
}

#[test]
fn code_points_optimization() {
    // Test Latin1 optimization with extended Latin1 characters
    let latin1_str = JsStr::latin1(b"Caf\xe9 na\xefve"); // "Café naïve" in Latin1 encoding
    let latin1_points: Vec<CodePoint> = latin1_str.code_points().collect();
    let expected_latin1: Vec<CodePoint> = "Café naïve".chars().map(CodePoint::Unicode).collect();
    assert_eq!(latin1_points, expected_latin1);

    // Test UTF-16 behavior unchanged (including non-ASCII)
    let utf16_str = JsStr::utf16(&[
        0x0043, 0x0061, 0x0066, 0x00E9, // "Café"
        0x0020, // space
        0x006E, 0x0061, 0x00EF, 0x0076, 0x0065, // "naïve"
    ]);
    let utf16_points: Vec<CodePoint> = utf16_str.code_points().collect();
    assert_eq!(latin1_points, utf16_points); // Same result for same content
}

#[test]
fn slice() {
    let sliced = {
        let base_str = JsString::from("Hello World");
        assert_eq!(base_str.kind(), JsStringKind::Latin1Sequence);

        base_str.slice(1, 5)
    };
    assert_eq!(sliced, JsString::from("ello"));
    assert_eq!(sliced.kind(), JsStringKind::Slice);

    let sliced2 = sliced.slice(1, 3);
    drop(sliced);
    assert_eq!(sliced2, JsString::from("ll"));
    assert_eq!(sliced2.kind(), JsStringKind::Slice);

    let sliced3 = sliced2.slice(0, 2);
    drop(sliced2);
    assert_eq!(sliced3, JsString::from("ll"));
    assert_eq!(sliced3.kind(), JsStringKind::Slice);

    let sliced4 = sliced3.slice(0, 2);
    drop(sliced3);
    assert_eq!(sliced4, JsString::from("ll"));
    assert_eq!(sliced4.kind(), JsStringKind::Slice);

    let sliced4 = sliced4.slice(0, 2);
    assert_eq!(sliced4, JsString::from("ll"));
    assert_eq!(sliced4.kind(), JsStringKind::Slice);

    let sliced5 = sliced4.slice(1, 1);
    assert_eq!(sliced5, JsString::from(""));
    assert_eq!(sliced5.kind(), JsStringKind::Static);

    assert_eq!(sliced5.slice(4, 4), StaticJsStrings::EMPTY_STRING);
}

#[test]
fn split() {
    let base_str = JsString::from("Hello World");
    assert_eq!(base_str.kind(), JsStringKind::Latin1Sequence);

    let str1 = base_str.slice(0, 5);
    let str2 = base_str.slice(6, base_str.len());

    assert_eq!(str1, JsString::from("Hello"));
    assert_eq!(str2, JsString::from("World"));

    let str3 = str1.clone();
    drop(str1);
    assert_eq!(str3, JsString::from("Hello"));
    drop(base_str);
    assert_eq!(str3, JsString::from("Hello"));
}

#[test]
fn trim() {
    // Very basic test for trimming. The extensive testing is done by `boa_engine`.
    let base_str = JsString::from(" \u{000B} Hello World \t ");
    assert_eq!(base_str.trim(), JsString::from("Hello World"));
}

#[test]
fn starts_with_and_ends_with_basic() {
    let basic = JsString::from("abcdef");
    let start_needle = JsStr::latin1("abc".as_bytes());
    assert!(basic.starts_with(start_needle));
    assert!(!basic.ends_with(start_needle));

    let end_needle = JsStr::latin1("def".as_bytes());
    assert!(!basic.starts_with(end_needle));
    assert!(basic.ends_with(end_needle));
}

#[test]
fn repeat() {
    let empty = JsString::from("");
    assert_eq!(empty.repeat(0), JsString::from(""));
    assert_eq!(empty.repeat(10), JsString::from(""));

    let single = JsString::from("a");
    assert_eq!(single.repeat(0), JsString::from(""));
    assert_eq!(single.repeat(1), JsString::from("a"));
    assert_eq!(single.repeat(5), JsString::from("aaaaa"));

    let latin = JsString::from("abc");
    assert_eq!(latin.repeat(3), JsString::from("abcabcabc"));
    assert_eq!(
        latin.repeat(10),
        JsString::from("abcabcabcabcabcabcabcabcabcabc")
    );

    let utf16 = JsString::from("🔥🦀");
    assert_eq!(utf16.repeat(0), JsString::from(""));
    assert_eq!(utf16.repeat(1), JsString::from("🔥🦀"));
    assert_eq!(utf16.repeat(4), JsString::from("🔥🦀🔥🦀🔥🦀🔥🦀"));
}

#[test]
fn join() {
    let sep = JsStr::latin1(", ".as_bytes());
    let empty_list: &[JsStr<'_>] = &[];
    assert_eq!(JsString::join(sep, empty_list), JsString::from(""));

    let single_item = [JsStr::latin1("one".as_bytes())];
    assert_eq!(JsString::join(sep, &single_item), JsString::from("one"));

    let multiple_items = [
        JsStr::latin1("one".as_bytes()),
        JsStr::latin1("two".as_bytes()),
        JsStr::latin1("three".as_bytes()),
    ];
    assert_eq!(
        JsString::join(sep, &multiple_items),
        JsString::from("one, two, three")
    );

    let utf16_item = JsStr::utf16(&[0xD83D, 0xDD25]); // 🔥
    let mixed_items = [
        JsStr::latin1("fire".as_bytes()),
        utf16_item,
        JsStr::latin1("crab".as_bytes()),
    ];
    assert_eq!(
        JsString::join(sep, &mixed_items),
        JsString::from("fire, 🔥, crab")
    );

    // Empty separator: pure concatenation.
    let no_sep = JsStr::latin1(b"");
    let ab = [JsStr::latin1(b"a"), JsStr::latin1(b"b")];
    assert_eq!(JsString::join(no_sep, &ab), JsString::from("ab"));

    // UTF-16 separator forces UTF-16 promotion.
    let utf16_sep = JsStr::utf16(&[0xD83D, 0xDD25]);
    let latin_elems = [JsStr::latin1(b"a"), JsStr::latin1(b"b")];
    let promoted = JsString::join(utf16_sep, &latin_elems);
    assert_eq!(promoted, JsString::from("a🔥b"));
    assert!(!promoted.as_str().is_latin1());

    // All-empty with empty sep hits the `total_len == 0` early return.
    let empties = [JsStr::latin1(b""), JsStr::latin1(b"")];
    assert_eq!(
        JsString::join(no_sep, &empties),
        StaticJsStrings::EMPTY_STRING
    );

    // Empty `JsStr::utf16(&[])` parts are encoding-neutral: result stays Latin1.
    let empty_utf16 = JsStr::utf16(&[]);
    let neutral = [JsStr::latin1(b"a"), empty_utf16, JsStr::latin1(b"b")];
    let neutral_joined = JsString::join(no_sep, &neutral);
    assert_eq!(neutral_joined, JsString::from("ab"));
    assert!(neutral_joined.as_str().is_latin1());

    // Empty input returns the EMPTY static.
    assert_eq!(
        JsString::join(sep, empty_list),
        StaticJsStrings::EMPTY_STRING
    );
}

#[test]
fn index_of_variants() {
    let latin = JsString::from("abcabc");
    let l = latin.as_str();
    assert_eq!(l.index_of(JsStr::latin1(b"bc"), 0), Some(1));
    assert_eq!(l.index_of(JsStr::latin1(b"a"), 1), Some(3));
    assert_eq!(l.index_of(JsStr::latin1(b"abc"), 4), None);
    assert_eq!(l.index_of(JsStr::latin1(b"abcdefg"), 0), None);
    assert_eq!(l.index_of(JsStr::latin1(b""), 2), Some(2));
    assert_eq!(l.index_of(JsStr::latin1(b""), 99), None);

    let utf16 = JsString::from("a🔥b🔥c");
    let u = utf16.as_str();
    // Lone trail-surrogate unit search in UTF-16 haystack.
    assert_eq!(u.index_of(JsStr::utf16(&[0xDD25]), 0), Some(2));
    // Latin1 needle in UTF-16 haystack.
    assert_eq!(u.index_of(JsStr::latin1(b"b"), 0), Some(3));
    // UTF-16 needle (<= 0xFF) in Latin1 haystack.
    assert_eq!(l.index_of(JsStr::utf16(&[u16::from(b'b')]), 0), Some(1));
    // UTF-16 needle with units > 0xFF can never match Latin1 haystack.
    assert_eq!(l.index_of(JsStr::utf16(&[0x2603]), 0), None);
    // UTF-16 x UTF-16 multi-unit.
    assert_eq!(
        u.index_of(JsStr::utf16(&[0xD83D, 0xDD25, u16::from(b'b')]), 0),
        Some(1)
    );
}

#[test]
fn replace_once_cases() {
    use crate::{JsStr, JsString, StaticJsStrings};

    let j = JsString::from;
    // Latin1 x Latin1.
    assert_eq!(
        JsString::replace_once(
            j("hello").as_str(),
            JsStr::latin1(b"l"),
            JsStr::latin1(b"L")
        ),
        j("heLlo")
    );
    // Empty search inserts at 0.
    assert_eq!(
        JsString::replace_once(j("abc").as_str(), JsStr::latin1(b""), JsStr::latin1(b"X")),
        j("Xabc")
    );
    // Not found returns an equal value.
    assert_eq!(
        JsString::replace_once(j("abc").as_str(), JsStr::latin1(b"z"), JsStr::latin1(b"X")),
        j("abc")
    );
    // Empty replacement deletes.
    assert_eq!(
        JsString::replace_once(
            j("abcabc").as_str(),
            JsStr::latin1(b"b"),
            JsStr::latin1(b"")
        ),
        j("acabc")
    );
    // total_len == 0 returns the EMPTY static.
    assert_eq!(
        JsString::replace_once(j("a").as_str(), JsStr::latin1(b"a"), JsStr::latin1(b"")),
        StaticJsStrings::EMPTY_STRING
    );
    // UTF-16 haystack + Latin1 needle/replacement promotes to UTF-16.
    let fire = JsString::from("fire🔥fire");
    let r = JsString::replace_once(
        fire.as_str(),
        JsStr::latin1(b"fire"),
        JsStr::latin1(b"water"),
    );
    assert_eq!(r, JsString::from("water🔥fire"));
    assert!(!r.as_str().is_latin1());
    // Non-Latin1 needle never matches a Latin1 haystack.
    let needle = JsStr::utf16(&[0xD83D, 0xDD25]);
    assert_eq!(
        JsString::replace_once(j("abc").as_str(), needle, JsStr::latin1(b"X")),
        j("abc")
    );
    // `replace_once_at` reuses a known position (no second search).
    let s = j("abcabc");
    let pos = s.as_str().index_of(JsStr::latin1(b"bc"), 0).unwrap();
    assert_eq!(
        JsString::replace_once_at(s.as_str(), 2, JsStr::latin1(b"X"), pos),
        j("aXabc")
    );
}

#[test]
fn repeat_encoding_and_sharing() {
    // `count == 1` on `&self` shares instead of copying.
    let s = JsString::from("abc");
    assert_eq!(s.repeat(1), JsString::from("abc"));
    // BMP non-Latin1 stays UTF-16.
    let bmp = JsString::from("年");
    let r = bmp.repeat(3);
    assert_eq!(r, JsString::from("年年年"));
    assert!(!r.as_str().is_latin1());
    // Latin1 stays Latin1.
    let latin = JsString::from("ab").repeat(3);
    assert_eq!(latin, JsString::from("ababab"));
    assert!(latin.as_str().is_latin1());
    // Doubling-remainder boundary.
    assert_eq!(
        JsString::from("abc").repeat(10),
        JsString::from("abcabcabcabcabcabcabcabcabcabc")
    );
}

#[test]
fn trim_sharing_and_empty() {
    use crate::{JsString, StaticJsStrings};

    let s = JsString::from("hello");
    // Already-trimmed shares the allocation instead of slicing.
    assert_eq!(s.trim(), JsString::from("hello"));
    assert_eq!(s.trim_start(), JsString::from("hello"));
    assert_eq!(s.trim_end(), JsString::from("hello"));

    assert_eq!(JsString::from("   ").trim(), StaticJsStrings::EMPTY_STRING);
    assert_eq!(
        JsString::from("").trim_start(),
        StaticJsStrings::EMPTY_STRING
    );
    assert_eq!(JsString::from("  a").trim_start(), JsString::from("a"));
    assert_eq!(JsString::from("a  ").trim_end(), JsString::from("a"));
}
