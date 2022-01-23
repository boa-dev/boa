use super::BufferedLexer;
use crate::{
    syntax::lexer::{Token, TokenKind},
    Interner,
};

#[test]
fn peek_skip_accending() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let mut interner = Interner::default();

    assert_eq!(
        *cur.peek(0, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a"))
    );
    assert_eq!(
        *cur.peek(1, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b"))
    );
    assert_eq!(
        *cur.peek(2, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c"))
    );
    assert_eq!(
        *cur.peek(2, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c"))
    );
    assert_eq!(
        *cur.peek(1, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b"))
    );
    assert_eq!(
        *cur.peek(0, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a"))
    );
}

#[test]
fn peek_skip_next() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let mut interner = Interner::default();

    assert_eq!(
        *cur.peek(0, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a"))
    );
    assert_eq!(
        *cur.peek(1, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b"))
    );
    assert_eq!(
        *cur.peek(2, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c"))
    );
    assert_eq!(
        *cur.next(false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a"))
    );
    assert_eq!(
        *cur.next(false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b"))
    );
    assert_eq!(
        *cur.next(false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c"))
    );
    assert_eq!(
        *cur.next(false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("d"))
    );
    assert_eq!(
        *cur.next(false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("e"))
    );
    assert_eq!(
        *cur.peek(0, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("f"))
    );
    assert_eq!(
        *cur.peek(1, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("g"))
    );
    assert_eq!(
        *cur.peek(2, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("h"))
    );
}

#[test]
fn peek_skip_next_alternating() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let mut interner = Interner::default();

    assert_eq!(
        *cur.peek(0, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a"))
    );
    assert_eq!(
        *cur.next(false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a"))
    );
    assert_eq!(
        *cur.peek(1, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c"))
    );
    assert_eq!(
        *cur.next(false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b"))
    );
    assert_eq!(
        *cur.peek(1, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("d"))
    );
    assert_eq!(
        *cur.next(false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c"))
    );
    assert_eq!(
        *cur.peek(2, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("f"))
    );
}

#[test]
fn peek_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let mut interner = Interner::default();

    loop {
        let peek = cur.peek(0, false, &mut interner).unwrap().cloned();
        let next = cur.next(false, &mut interner).unwrap();

        assert_eq!(peek, next);

        if peek.is_none() {
            break;
        }
    }
}

#[test]
fn peek_skip_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let mut interner = Interner::default();

    let mut peeked: [Option<Token>; super::MAX_PEEK_SKIP + 1] =
        [None::<Token>, None::<Token>, None::<Token>, None::<Token>];

    loop {
        for (i, peek) in peeked.iter_mut().enumerate() {
            *peek = cur.peek(i, false, &mut interner).unwrap().cloned();
        }

        for peek in &peeked {
            assert_eq!(&cur.next(false, &mut interner).unwrap(), peek);
        }

        if peeked[super::MAX_PEEK_SKIP - 1].is_none() {
            break;
        }
    }
}

#[test]
fn skip_peeked_terminators() {
    let mut cur = BufferedLexer::from(&b"A \n B"[..]);
    let mut interner = Interner::default();

    assert_eq!(
        *cur.peek(0, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("A"))
    );
    assert_eq!(
        *cur.peek(0, true, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("A"))
    );

    assert_eq!(
        *cur.peek(1, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::LineTerminator,
    );
    assert_eq!(
        *cur.peek(1, true, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("B")) // This value is after the line terminator
    );

    assert_eq!(
        *cur.peek(2, false, &mut interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("B"))
    );
    // End of stream
    assert!(cur.peek(2, true, &mut interner).unwrap().is_none());
}
