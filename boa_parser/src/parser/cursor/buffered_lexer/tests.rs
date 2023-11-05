use crate::{
    lexer::{Token, TokenKind},
    parser::cursor::BufferedLexer,
};
use boa_interner::Interner;

#[test]
fn peek_skip_accending() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("a"))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("b"))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("c"))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("c"))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("b"))
    );
    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("a"))
    );
}

#[test]
fn peek_skip_next() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("a"))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("b"))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("c"))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("a"))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("b"))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("c"))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("d"))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("e"))
    );
    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("f"))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("g"))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("h"))
    );
}

#[test]
fn peek_skip_next_alternating() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("a"))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("a"))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("c"))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("b"))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("d"))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("c"))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("f"))
    );
}

#[test]
fn peek_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    loop {
        let peek = cur.peek(0, false, interner).unwrap().cloned();
        let next = cur.next(false, interner).unwrap();

        assert_eq!(peek, next);

        if peek.is_none() {
            break;
        }
    }
}

#[test]
fn peek_skip_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    let mut peeked: [Option<Token>; super::MAX_PEEK_SKIP + 1] =
        [None::<Token>, None::<Token>, None::<Token>, None::<Token>];

    loop {
        for (i, peek) in peeked.iter_mut().enumerate() {
            *peek = cur.peek(i, false, interner).unwrap().cloned();
        }

        for peek in &peeked {
            assert_eq!(&cur.next(false, interner).unwrap(), peek);
        }

        if peeked[super::MAX_PEEK_SKIP - 1].is_none() {
            break;
        }
    }
}

#[test]
fn skip_peeked_terminators() {
    let mut cur = BufferedLexer::from(&b"A \n B"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("A"))
    );
    assert_eq!(
        *cur.peek(0, true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("A"))
    );

    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::LineTerminator,
    );
    assert_eq!(
        *cur.peek(1, true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("B")) /* This value is after the line terminator */
    );

    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern("B"))
    );
    // End of stream
    assert!(cur.peek(2, true, interner).unwrap().is_none());
}

#[test]
fn issue_1768() {
    let mut cur = BufferedLexer::from(&b"\n(\nx\n)\n"[..]);
    let interner = &mut Interner::default();

    assert!(cur.peek(3, true, interner).unwrap().is_none());
}
