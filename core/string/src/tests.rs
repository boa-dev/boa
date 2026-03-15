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

    let xy = JsString::concat(&x, &JsString::from(Y));
    assert_eq!(&xy, &ascii_to_utf16(b"hello, "));
    assert_eq!(xy.refcount(), Some(1));

    let xyz = JsString::concat(&xy, &z);
    assert_eq!(&xyz, &ascii_to_utf16(b"hello, world"));
    assert_eq!(xyz.refcount(), Some(1));

    let xyzw = JsString::concat(&xyz, &JsString::from(W));
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
fn rope_basic() {
    let s_large = JsString::from("a".repeat(1025)); // 1025 chars
    let s3 = JsString::from("!");
    let rope = JsString::concat_array_strings(&[s_large.clone(), s3.clone()]);

    assert_eq!(rope.kind(), JsStringKind::Rope);
    assert_eq!(rope.len(), 1026);
    assert_eq!(rope.code_unit_at(1025), Some(u16::from(b'!')));
}

#[test]
fn rope_balanced_tree() {
    let strings: Vec<JsString> = (0..8)
        .map(|i| JsString::from("a".repeat(1025) + &format!("{i:03}")))
        .collect();

    let rope = JsString::concat_array_strings(&strings);
    assert_eq!(rope.kind(), JsStringKind::Rope);
    assert_eq!(rope.len(), 8 * 1028);

    // With 8 strings, balanced tree should have depth 3.
    assert_eq!(rope.depth(), 3);
}

#[test]
fn rope_rebalancing() {
    let mut s = JsString::from("a".repeat(1025));
    // Highly unbalanced incremental concatenation.
    for _ in 0..100 {
        s = JsString::concat_array_strings(&[s, JsString::from("b")]);
    }

    // Without rebalancing, depth would be 100.
    // With Fibonacci rebalancing and hysteresis, depth should be kept small (e.g. < 45).
    assert_eq!(s.kind(), JsStringKind::Rope);
    assert!(
        s.depth() < 45,
        "Depth should be balanced (was {})",
        s.depth()
    );

    // Verify it still works.
    assert_eq!(s.code_unit_at(0), Some(u16::from(b'a')));
    assert_eq!(s.code_unit_at(1025), Some(u16::from(b'b')));
    assert_eq!(s.code_unit_at(1025 + 99), Some(u16::from(b'b')));
}

#[test]
fn pathological_batch_rebalancing() {
    // Create a very deep (unbalanced) rope manually (if possible) or by bypassing create if needed.
    // Actually, we can just create strings that are just at the threshold of rebalancing.
    // But since concat_strings_balanced now collects leaves, it's inherently safe.
    let strings: Vec<JsString> = (0..50).map(|_| JsString::from("a".repeat(200))).collect();

    // Batch concat should produce a balanced tree.
    let rope = JsString::concat_array_strings(&strings);
    assert_eq!(rope.kind(), JsStringKind::Rope);
    // log2(50) is ~6.
    assert!(
        rope.depth() <= 7,
        "Batch concat should be perfectly balanced (was {})",
        rope.depth()
    );
}

#[test]
fn test_rope_fibonacci_rebalancing() {
    let mut s1 = JsString::from("a".repeat(20)); // Base length to bypass 512 flat threshold quickly
    let mut s2 = JsString::from("b".repeat(20));

    // Skew right
    for _ in 0..200 {
        s1 = JsString::concat(&s1, &JsString::from("c"));
    }
    // Skew left
    for _ in 0..200 {
        s2 = JsString::concat(&JsString::from("d"), &s2);
    }

    assert_eq!(s1.len(), 20 + 200);
    assert_eq!(s2.len(), 20 + 200);

    // Despite 10,000 skewed concatenations, the Fibonacci heuristic
    // ensures the rope depth does not exceed moderate bounds (e.g. < 45).
    assert!(
        s1.depth() < 45,
        "Right-skewed rope should have logarithmic depth via Fibonacci rebalancing, got: {}",
        s1.depth()
    );
    assert!(
        s2.depth() < 45,
        "Left-skewed rope should have logarithmic depth via Fibonacci rebalancing, got: {}",
        s2.depth()
    );

    // Verify traversal is still accurate across the rebalanced structure
    assert_eq!(s1.code_unit_at(0), Some(u16::from(b'a')));
    assert_eq!(s1.code_unit_at(219), Some(u16::from(b'c')));

    assert_eq!(s2.code_unit_at(0), Some(u16::from(b'd')));
    assert_eq!(s2.code_unit_at(219), Some(u16::from(b'b')));
}

#[test]
fn rope_dag_sharing_stress() {
    // Create a DAG via s = s + s.
    // Length grows as 2^n.
    let mut s = JsString::from("abc");
    for _ in 0..30 {
        s = JsString::concat(&s, &s);
    }

    // Depth should be reasonable (22).
    // Note: iterations where length <= 1024 produce SequenceStrings (depth 0),
    // and s="abc" (3) * 2^8 = 768. 3 * 2^9 = 1536.
    // So depth only starts increasing at iter 8. 30 - 8 = 22.
    assert_eq!(s.depth(), 22);
    assert_eq!(s.len(), (1 << 30) * 3);

    // Flattening would take ~3GB, which might be too much for some CI environments.
    // But we can check code_unit_at which is O(depth).
    assert_eq!(s.code_unit_at(0), Some(u16::from(b'a')));
    assert_eq!(s.code_unit_at(s.len() - 1), Some(u16::from(b'c')));
}

#[test]
fn rope_dag_rebalance_explosion_prevented() {
    // To trigger rebalance on a DAG, we need len < Fib(depth+2).
    // So we need a very deep tree with very little content, but with sharing.
    // Pathological case: s = "a", then s = s + empty.
    let mut s = JsString::from("a");
    let empty = JsString::from("");
    for _ in 0..35 {
        s = JsString::concat(&s, &empty);
    }
    // Now s has depth 35 and len 1.
    // Fib(37) is 24 million. 1 < 24 million, so it rebalances.
    // If we share the s:
    let shared = JsString::concat(&s, &s);
    assert_eq!(shared.len(), 2);
}

#[test]
fn test_reentrancy_oncecell() {
    let mut s = JsString::from("a");
    // Create a deeply shared DAG
    for i in 0..10 {
        s = JsString::concat(&s.clone(), &s.clone());
        println!("Depth step {}: len = {}", i, s.len());
    }

    println!("Triggering lazy flattening on DAG...");
    // Our iterative flattening naturally avoids OnceCell reentrancy panics.
    // In a DAG where `left == right`, a recursive implementation would call `as_str()`
    // on the same node twice, triggering `get_or_init` while already inside it.
    // However, our iterative `rope_as_str` avoids calling `as_str()` on sub-ropes,
    // instead manually traversing them using `flattened.get()`. This ensures
    // we never trigger a recursive initialization of the same OnceCell.
    let flat = s.as_str();
    println!("Flattened successfully! Len: {}", flat.len());
}

#[test]
fn deep_rope_stress() {
    let mut s = JsString::from("a");

    for _ in 0..10000 {
        s = JsString::concat(&s.clone(), &"b".into());
    }

    let _ = s.as_str();
}

#[test]
fn shared_rope_stress() {
    let base = JsString::from("x");

    let mut ropes = Vec::new();

    for _ in 0..1000 {
        ropes.push(JsString::concat(&base.clone(), &base.clone()));
    }

    for r in ropes {
        let _ = r.as_str();
    }
}

#[test]
fn rope_flatten_cache() {
    let a = JsString::from("a".repeat(2000));
    let b = JsString::from("b".repeat(2000));

    let rope = JsString::concat(&a, &b);

    let first = rope.as_str();
    let second = rope.as_str();

    assert_eq!(first, second);
}

#[test]
fn flatten_shared_subtree() {
    let base = JsString::from("x".repeat(2000));

    let a = JsString::concat(&base, &base);
    let b = JsString::concat(&base, &base);

    let root = JsString::concat(&a, &b);

    assert_eq!(root.as_str().len(), 8000);
}
