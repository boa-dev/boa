use super::Cursor;
use crate::syntax::lexer::{Token, TokenKind};

#[test]
fn peek_skip_accending() {
    let buf: &[u8] = "a b c d e f g h i".as_bytes();

    let mut cur = Cursor::new(buf);

    assert_eq!(
        *cur.peek(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a")
    );
    assert_eq!(
        *cur.peek_skip(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b")
    );
    assert_eq!(
        *cur.peek_skip(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c")
    );
    assert_eq!(
        *cur.peek_skip(3, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("d")
    );
    assert_eq!(
        *cur.peek_skip(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c")
    );
    assert_eq!(
        *cur.peek_skip(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b")
    );
    assert_eq!(
        *cur.peek(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a")
    );
}

#[test]
fn peek_skip_next() {
    let buf: &[u8] = "a b c d e f g h i".as_bytes();

    let mut cur = Cursor::new(buf);

    assert_eq!(
        *cur.peek(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a")
    );
    assert_eq!(
        *cur.peek_skip(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b")
    );
    assert_eq!(
        *cur.peek_skip(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c")
    );
    assert_eq!(
        *cur.peek_skip(3, false)
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
        *cur.peek(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("f")
    );
    assert_eq!(
        *cur.peek_skip(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("g")
    );
    assert_eq!(
        *cur.peek_skip(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("h")
    );
    assert_eq!(
        *cur.peek_skip(3, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("i")
    );
}

#[test]
fn peek_skip_next_alternating() {
    let buf: &[u8] = "a b c d e f g h i".as_bytes();

    let mut cur = Cursor::new(buf);

    assert_eq!(
        *cur.peek(false)
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
        *cur.peek_skip(1, false)
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
        *cur.peek_skip(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("d")
    );
    assert_eq!(
        *cur.peek_skip(3, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("f")
    );
    assert_eq!(
        *cur.next(false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("c")
    );
    assert_eq!(
        *cur.peek_skip(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("f")
    );
    assert_eq!(
        *cur.peek_skip(3, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("g")
    );
}

#[test]
fn peek_next_till_end() {
    let buf: &[u8] = "a b c d e f g h i".as_bytes();

    let mut cur = Cursor::new(buf);

    loop {
        let peek = cur.peek(false).unwrap();
        let next = cur.next(false).unwrap();

        assert_eq!(peek, next);

        if peek.is_none() {
            break;
        }
    }
}

#[test]
fn peek_skip_next_till_end() {
    let mut cur = Cursor::new("a b c d e f g h i".as_bytes());

    let mut peeked: [Option<Token>; super::MAX_PEEK_SKIP + 1] =
        [None::<Token>, None::<Token>, None::<Token>, None::<Token>];

    loop {
        for i in 0..super::MAX_PEEK_SKIP {
            peeked[i] = cur.peek_skip(i, false).unwrap();
        }

        for i in 0..super::MAX_PEEK_SKIP {
            assert_eq!(cur.next(false).unwrap(), peeked[i]);
        }

        if peeked[super::MAX_PEEK_SKIP - 1].is_none() {
            break;
        }
    }
}
