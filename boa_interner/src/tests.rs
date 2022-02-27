use crate::{Interner, JStrRef, Sym, COMMON_STRINGS};
use const_utf16::encode as utf16;

#[track_caller]
fn sym_from_usize(index: usize) -> Sym {
    Sym::new(index).expect("Invalid NonZeroUsize")
}

#[test]
fn check_static_strings() {
    let mut interner = Interner::default();

    for (i, &str) in COMMON_STRINGS.into_iter().enumerate() {
        assert_eq!(interner.get_or_intern(str), sym_from_usize(i + 1));
    }
}

#[test]
fn check_new_string() {
    let mut interner = Interner::default();

    assert!(interner.get_or_intern("my test string").get() > COMMON_STRINGS.len());
}

#[test]
fn check_resolve() {
    let mut interner = Interner::default();

    let utf_8_strings = ["test string ", "arguments ", "hello "];
    let utf_8_strings = utf_8_strings.into_iter();
    let utf_16_strings = [
        &utf16!("TEST STRING")[..],
        utf16!("ARGUMENTS"),
        utf16!("HELLO"),
    ];
    let utf_16_strings = utf_16_strings.into_iter();

    for (s8, s16) in utf_8_strings.zip(utf_16_strings) {
        let sym = interner.get_or_intern(s8);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(JStrRef::from(s8), resolved);
        let new_sym = interner.get_or_intern(s8);
        assert_eq!(sym, new_sym);

        let sym = interner.get_or_intern(s16);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(JStrRef::from(s16), resolved);
        let new_sym = interner.get_or_intern(s16);
        assert_eq!(sym, new_sym);
    }
}

#[test]
fn check_static_resolve() {
    let mut interner = Interner::default();

    for string in COMMON_STRINGS
        .into_iter()
        .copied()
        .chain(["my test str", "hello world", ";"].into_iter())
    {
        let sym = interner.get_or_intern_static(string);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(JStrRef::from(string), resolved);

        let new_sym = interner.get_or_intern(string);

        assert_eq!(sym, new_sym);
    }

    for string in [
        &utf16!("MY TEST STR")[..],
        utf16!("HELLO WORLD"),
        utf16!(";"),
    ] {
        let sym = interner.get_or_intern_static(string);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(JStrRef::from(string), resolved);

        let new_sym = interner.get_or_intern(string);

        assert_eq!(sym, new_sym);
    }
}
