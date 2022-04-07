use crate::{Interner, Sym};
use std::num::NonZeroUsize;

#[track_caller]
fn sym_from_usize(index: usize) -> Sym {
    Sym::from_raw(NonZeroUsize::new(index).expect("Invalid NonZeroUsize"))
}

#[test]
fn check_static_strings() {
    let mut interner = Interner::default();

    for (i, str) in Interner::STATIC_STRINGS.into_iter().enumerate() {
        assert_eq!(interner.get_or_intern(str), sym_from_usize(i + 1));
    }
}

#[test]
fn check_constants() {
    assert_eq!(Sym::EMPTY_STRING, sym_from_usize(1));
    assert_eq!(Sym::ARGUMENTS, sym_from_usize(2));
    assert_eq!(Sym::AWAIT, sym_from_usize(3));
    assert_eq!(Sym::YIELD, sym_from_usize(4));
    assert_eq!(Sym::EVAL, sym_from_usize(5));
    assert_eq!(Sym::DEFAULT, sym_from_usize(6));
    assert_eq!(Sym::NULL, sym_from_usize(7));
    assert_eq!(Sym::REGEXP, sym_from_usize(8));
    assert_eq!(Sym::GET, sym_from_usize(9));
    assert_eq!(Sym::SET, sym_from_usize(10));
    assert_eq!(Sym::MAIN, sym_from_usize(11));
    assert_eq!(Sym::RAW, sym_from_usize(12));
    assert_eq!(Sym::STATIC, sym_from_usize(13));
    assert_eq!(Sym::PROTOTYPE, sym_from_usize(14));
    assert_eq!(Sym::CONSTRUCTOR, sym_from_usize(15));
    assert_eq!(Sym::IMPLEMENTS, sym_from_usize(16));
    assert_eq!(Sym::INTERFACE, sym_from_usize(17));
    assert_eq!(Sym::LET, sym_from_usize(18));
    assert_eq!(Sym::PACKAGE, sym_from_usize(19));
    assert_eq!(Sym::PRIVATE, sym_from_usize(20));
    assert_eq!(Sym::PROTECTED, sym_from_usize(21));
    assert_eq!(Sym::PUBLIC, sym_from_usize(22));
}

#[test]
fn check_new_string() {
    let mut interner = Interner::default();

    assert!(
        interner.get_or_intern("my test string").as_raw().get() > Interner::STATIC_STRINGS.len()
    );
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

    for string in Interner::STATIC_STRINGS
        .into_iter()
        .chain(["my test str", "hello world", ";"].into_iter())
    {
        let sym = interner.get_or_intern_static(string);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(string, resolved);

        let new_sym = interner.get_or_intern(string);

        assert_eq!(sym, new_sym);
    }
}
