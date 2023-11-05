use crate::{Interner, Sym, COMMON_STRINGS_UTF8};
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
        assert_eq!(s8, &*resolved);
        let new_sym = interner.get_or_intern(s8);
        assert_eq!(sym, new_sym);

        let sym = interner.get_or_intern(s16);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(s16, &*resolved);
        let new_sym = interner.get_or_intern(s16);
        assert_eq!(sym, new_sym);
    }
}

#[test]
fn check_static_resolve() {
    let mut interner = Interner::default();

    for string in COMMON_STRINGS_UTF8
        .into_iter()
        .copied()
        .chain(["my test str", "hello world", ";"].into_iter())
    {
        let sym = interner.get_or_intern(string);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(string, &*resolved);

        let new_sym = interner.get_or_intern(string);
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

    let sym3 = interner.get_or_intern(&unp[..]);
    let sym4 = interner.get_or_intern("ghi");
    let sym5 = interner.get_or_intern(&unp2[..]);

    let sym6 = interner.get_or_intern("jkl");

    assert_eq!(&*interner.resolve_expect(sym), "abc");
    assert_eq!(&*interner.resolve_expect(sym2), "def");
    assert_eq!(&*interner.resolve_expect(sym3), &unp[..]);
    assert_eq!(&*interner.resolve_expect(sym4), "ghi");
    assert_eq!(&*interner.resolve_expect(sym5), &unp2[..]);
    assert_eq!(&*interner.resolve_expect(sym6), "jkl");
}

#[test]
fn check_empty_interner() {
    let interner = Interner::default();

    let sym = sym_from_usize(123); // Choose an arbitrary symbol

    assert!(interner.resolve(sym).is_none());
}

#[test]
fn check_capacity() {
    let interner = Interner::with_capacity(100);

    let sym = sym_from_usize(123); // Choose an arbitrary symbol

    assert!(interner.resolve(sym).is_none());
}
