#![allow(clippy::redundant_clone)]

use std::hash::{BuildHasher, BuildHasherDefault, Hash};

use crate::{
    CommonJsStringBuilder, JsStr, JsString, Latin1JsStringBuilder, StaticJsString, StaticJsStrings,
    ToStringEscaped, Utf16JsStringBuilder,
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

    assert!(!x.ptr.is_tagged());

    assert_eq!(x.ptr.addr(), y.ptr.addr());

    let z = JsString::from("Hello");
    assert_ne!(x.ptr.addr(), z.ptr.addr());
    assert_ne!(y.ptr.addr(), z.ptr.addr());
}

#[test]
fn static_ptr_eq() {
    let x = StaticJsStrings::EMPTY_STRING;
    let y = x.clone();

    assert!(x.ptr.is_tagged());

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

    let y = JsString::from(x.trim_start());

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
fn to_string_escaped() {
    assert_eq!(
        JsString::from("Hello, \u{1D49E} world!").to_string_escaped(),
        "Hello, \u{1D49E} world!"
    );

    assert_eq!(
        JsString::from("Hello, world!").to_string_escaped(),
        "Hello, world!"
    );

    // 15 should not be escaped.
    let unpaired_surrogates: [u16; 3] = [0xDC58, 0xD83C, 0x0015];
    assert_eq!(
        JsString::from(&unpaired_surrogates).to_string_escaped(),
        "\\uDC58\\uD83C\u{15}"
    );
}

#[test]
fn from_static_js_string() {
    static STATIC_HELLO_WORLD: StaticJsString =
        StaticJsString::new(JsStr::latin1("hello world".as_bytes()));
    static STATIC_EMOJIS: StaticJsString = StaticJsString::new(JsStr::utf16(&[
        0xD83C, 0xDFB9, 0xD83C, 0xDFB6, 0xD83C, 0xDFB5,
    ])); // 🎹🎶🎵
    let latin1 = JsString::from_static_js_string(&STATIC_HELLO_WORLD);
    let utf16 = JsString::from_static_js_string(&STATIC_EMOJIS);

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
    static STATIC_HELLO_WORLD: StaticJsString =
        StaticJsString::new(JsStr::latin1("hello world".as_bytes()));
    static STATIC_EMOJIS: StaticJsString = StaticJsString::new(JsStr::utf16(&[
        0xD83C, 0xDFB9, 0xD83C, 0xDFB6, 0xD83C, 0xDFB5,
    ])); // 🎹🎶🎵
    let static_latin1 = JsString::from_static_js_string(&STATIC_HELLO_WORLD);
    let static_utf16 = JsString::from_static_js_string(&STATIC_EMOJIS);

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
    assert!(cloned_from.capacity() == 0);
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
    assert!(cloned_from.capacity() == 0);
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
