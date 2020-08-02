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

#[test]
fn push_back_peek() {
    let mut cur = Cursor::new("a b c d e f g h i".as_bytes());

    let next = cur.next(false).unwrap().expect("Expected some");
    assert_eq!(
        *cur.peek_skip(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("b")
    );
    cur.push_back(next);
    assert_eq!(
        *cur.peek_skip(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("a")
    );
    assert_eq!(
        *cur.peek_skip(3, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("d")
    );
}

#[test]
fn skip_peeked_terminators() {
    // If skip_line_terminators is true then line terminators will be discarded.
    // / i.e. If there are tokens A, B, \n, C and peek_skip(0, false) is 'A' then the following will hold:
    // /         peek_skip(0, true) == 'A'
    // /         peek_skip(1, false) == 'B'
    // /         peek_skip(1, true) == 'B'
    // /         peek_skip(2, false) == \n
    // /         peek_skip(2, true) == 'C'
    // /         peek_skip(3, true) == None (End of stream)
    // /  Note:
    // /     peek_skip(3, false) == 'C' iff peek_skip(3, true) hasn't been called previously, this is because
    // /     with skip_line_terminators == true the '\n' would be discarded. This leads to the following statements
    // /     evaluating to true (in isolation from each other or any other previous cursor calls):
    // /         peek_skip(3, false) == peek_skip(3, false) == '\n'
    // /         peek_skip(3, true) == peek_skip(3, true) == None
    // /         peek_skip(3, true) == peek_skip(3, false) == None
    // /         (peek_skip(3, false) == 'C') != (peek_skip(3, true) == None)

    let mut cur = Cursor::new("A B \n C".as_bytes());
    assert_eq!(
        *cur.peek_skip(0, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("A")
    );
    assert_eq!(
        *cur.peek_skip(0, true)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("A")
    );
    assert_eq!(
        *cur.peek_skip(1, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("B")
    );
    assert_eq!(
        *cur.peek_skip(1, true)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("B")
    );
    assert_eq!(
        *cur.peek_skip(2, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::LineTerminator
    );
    assert_eq!(
        *cur.peek_skip(3, false)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("C")
    );
    println!("mark");
    assert_eq!(
        *cur.peek_skip(2, true)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier("C") // This value is after the line terminator.
    );

    // Note that now the line terminator is gone and any subsequent call will not return it.
    // This is because the previous peek_skip(2, true) call skipped (and therefore destroyed) it
    // because the returned value ("C") is after the line terminator.

    assert!(cur.peek_skip(3, false).unwrap().is_none());
    assert!(cur.peek_skip(3, true).unwrap().is_none());
}
