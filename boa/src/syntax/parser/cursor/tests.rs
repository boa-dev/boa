use super::Cursor;
use crate::syntax::lexer::TokenKind;

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
