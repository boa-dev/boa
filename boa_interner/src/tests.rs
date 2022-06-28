use crate::{Interner, Sym, COMMON_STRINGS};

#[track_caller]
fn sym_from_usize(index: usize) -> Sym {
    Sym::new(index).expect("Invalid NonZeroUsize")
}

#[test]
fn check_static_strings() {
    let mut interner = Interner::default();

    for (i, str) in COMMON_STRINGS.into_iter().enumerate() {
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

    let strings = ["test string", "arguments", "hello"];

    for string in strings {
        let sym = interner.get_or_intern(string);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(string, resolved);

        let new_sym = interner.get_or_intern(string);

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
        assert_eq!(string, resolved);

        let new_sym = interner.get_or_intern(string);

        assert_eq!(sym, new_sym);
    }
}
