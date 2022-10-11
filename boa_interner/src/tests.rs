use crate::{Interner, Sym, COMMON_STRINGS_UTF16, COMMON_STRINGS_UTF8};
use boa_macros::utf16;

#[track_caller]
fn sym_from_usize(index: usize) -> Sym {
    Sym::new(index).expect("Invalid NonZeroUsize")
}

#[test]
fn check_static_strings() {
    let mut interner = Interner::default();

    for (i, &str) in COMMON_STRINGS_UTF8.into_iter().enumerate() {
        assert_eq!(interner.get_or_intern(str), sym_from_usize(i + 1));
    }
}

#[test]
fn check_new_string() {
    let mut interner = Interner::default();

    assert!(interner.get_or_intern("my test string").get() > COMMON_STRINGS_UTF8.len());
}

#[test]
fn check_resolve() {
    let mut interner = Interner::default();

    let utf_8_strings = ["test string", "arguments", "hello"];
    let utf_8_strings = utf_8_strings.into_iter();
    let utf_16_strings = [utf16!("TEST STRING"), utf16!("ARGUMENTS"), utf16!("HELLO")];
    let utf_16_strings = utf_16_strings.into_iter();

    for (s8, s16) in utf_8_strings.zip(utf_16_strings) {
        let sym = interner.get_or_intern(s8);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(Some(s8), resolved.utf8());
        let new_sym = interner.get_or_intern(s8);
        assert_eq!(sym, new_sym);

        let sym = interner.get_or_intern(s16);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(s16, resolved.utf16());
        let new_sym = interner.get_or_intern(s16);
        assert_eq!(sym, new_sym);
    }
}

#[test]
fn check_static_resolve() {
    let mut interner = Interner::default();

    for (utf8, utf16) in COMMON_STRINGS_UTF8
        .into_iter()
        .copied()
        .zip(COMMON_STRINGS_UTF16.iter().copied())
        .chain(
            [
                ("my test str", utf16!("my test str")),
                ("hello world", utf16!("hello world")),
                (";", utf16!(";")),
            ]
            .into_iter(),
        )
    {
        let sym = interner.get_or_intern_static(utf8, utf16);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(Some(utf8), resolved.utf8());
        assert_eq!(utf16, resolved.utf16());

        let new_sym = interner.get_or_intern(utf8);

        assert_eq!(sym, new_sym);
    }
}

#[test]
fn check_unpaired_surrogates() {
    let mut interner = Interner::default();

    let unp = &[
        0xDC15u16, 0xDC19, 'h' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 'o' as u16,
    ];
    let unp2 = &[
        0xDC01u16, 'w' as u16, 'o' as u16, 'r' as u16, 0xDCF4, 'l' as u16, 'd' as u16,
    ];

    let sym = interner.get_or_intern("abc");
    let sym2 = interner.get_or_intern("def");

    let sym3 = interner.get_or_intern(unp);
    let sym4 = interner.get_or_intern(utf16!("ghi"));
    let sym5 = interner.get_or_intern(unp2);

    let sym6 = interner.get_or_intern("jkl");

    assert_eq!(interner.resolve_expect(sym).utf8(), Some("abc"));
    assert_eq!(interner.resolve_expect(sym).utf16(), utf16!("abc"));

    assert_eq!(interner.resolve_expect(sym2).utf8(), Some("def"));
    assert_eq!(interner.resolve_expect(sym2).utf16(), utf16!("def"));

    assert!(interner.resolve_expect(sym3).utf8().is_none());
    assert_eq!(interner.resolve_expect(sym3).utf16(), unp);

    assert_eq!(interner.resolve_expect(sym4).utf8(), Some("ghi"));
    assert_eq!(interner.resolve_expect(sym4).utf16(), utf16!("ghi"));

    assert!(interner.resolve_expect(sym5).utf8().is_none());
    assert_eq!(interner.resolve_expect(sym5).utf16(), unp2);

    assert_eq!(interner.resolve_expect(sym6).utf8(), Some("jkl"));
    assert_eq!(interner.resolve_expect(sym6).utf16(), utf16!("jkl"));
}
