use super::BufferedLexer;
use crate::syntax::lexer::{Token, TokenKind};

#[test]
fn peek_skip_accending() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);

    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a")
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b")
    );
    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c")
    );
    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c")
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b")
    );
    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a")
    );
}

#[test]
fn peek_skip_next() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);

    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a")
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b")
    );
    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c")
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a")
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b")
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c")
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("d")
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("e")
    );
    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("f")
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("g")
    );
    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("h")
    );
}

#[test]
fn peek_skip_next_alternating() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);

    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a")
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a")
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c")
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b")
    );
    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("d")
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c")
    );
    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("f")
    );
}

#[test]
fn peek_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);

    loop {
        let peek = cur.peek(0, false).unwrap().cloned();
        let next = cur.next(false).unwrap();

        assert_eq!(peek, next);

        if peek.is_none() {
            break;
        }
    }
}

#[test]
fn peek_skip_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);

    let mut peeked: [Option<Token>; super::MAX_PEEK_SKIP + 1] =
        [None::<Token>, None::<Token>, None::<Token>];

    loop {
        for (i, peek) in peeked.iter_mut().enumerate() {
            *peek = cur.peek(i, false).unwrap().cloned();
        }

        for peek in &peeked {
            assert_eq!(&cur.next(false).unwrap(), peek);
        }

        if peeked[super::MAX_PEEK_SKIP - 1].is_none() {
            break;
        }
    }
}

#[test]
fn skip_peeked_terminators() {
    let mut cur = BufferedLexer::from(&b"A \n B"[..]);
    assert_eq!(
        *cur.peek(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("A")
    );
    assert_eq!(
        *cur.peek(0, true)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("A")
    );

    assert_eq!(
        *cur.peek(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::LineTerminator,
    );
    assert_eq!(
        *cur.peek(1, true)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("B") // This value is after the line terminator
    );

    assert_eq!(
        *cur.peek(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("B")
    );
    // End of stream
    assert!(cur.peek(2, true).unwrap().is_none());
}
