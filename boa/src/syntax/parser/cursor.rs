//! Cursor implementation for the parser.

use super::ParseError;
use crate::syntax::ast::Punctuator;
use crate::syntax::lexer::Lexer;
use crate::syntax::lexer::{Token, TokenKind};

use std::collections::VecDeque;
use std::io::Read;

/// The maximum number of values stored by the cursor to allow back().
const BACK_QUEUE_MAX_LEN: usize = 3;

/// Token cursor.
///
/// This internal structure gives basic testable operations to the parser.
#[derive(Debug)]
pub(super) struct Cursor<R> {
    /// The tokens being input.
    // tokens: &'a [Token],
    lexer: Lexer<R>,
    // The current position within the tokens.
    // pos: usize,

    // peeked: Option<Option<Token>>,
    peeked: VecDeque<Option<Token>>,
    // Values are added to this queue when they are retrieved (next) to allow moving backwards.
    // back_queue: VecDeque<Option<Token>>,

    // peeked: Option<Option<Token>>,
}

impl<R> Cursor<R>
where
    R: Read,
{
    /// Creates a new cursor.
    pub(super) fn new(reader: R) -> Self {
        Self {
            lexer: Lexer::new(reader),
            peeked: VecDeque::new(),
            // back_queue: VecDeque::new(),
        }
    }

    /// Moves the cursor to the next token and returns the token.
    pub(super) fn next(&mut self) -> Option<Result<Token, ParseError>> {
        match self.peeked.pop_front() {
            Some(None) => {
                // if self.back_queue.len() >= BACK_QUEUE_MAX_LEN {
                //     self.back_queue.pop_front(); // Remove the value from the front of the queue.
                // }

                // self.back_queue.push_back(None);

                return None;
            }
            Some(Some(token)) => {
                // if self.back_queue.len() >= BACK_QUEUE_MAX_LEN {
                //     self.back_queue.pop_front(); // Remove the value from the front of the queue.
                // }

                // self.back_queue.push_back(Some(token.clone()));

                return Some(Ok(token));
            }
            None => {} // No value has been peeked ahead already so need to go get the next value.
        }

        loop {
            match self.lexer.next() {
                Some(Ok(tk)) => {
                    return Some(Ok(tk));

                    // if tk.kind != TokenKind::LineTerminator {
                    //     // if self.back_queue.len() >= BACK_QUEUE_MAX_LEN {
                    //     //     self.back_queue.pop_front(); // Remove the value from the front of the queue.
                    //     // }

                    //     // self.back_queue.push_back(Some(tk.clone()));

                    //     return Some(Ok(tk));
                    // }
                }
                Some(Err(e)) => {
                    return Some(Err(ParseError::lex(e)));
                }
                None => {
                    // if self.back_queue.len() >= BACK_QUEUE_MAX_LEN {
                    //     self.back_queue.pop_front(); // Remove the value from the front of the queue.
                    // }

                    // self.back_queue.push_back(None);

                    return None;
                }
            }
        }
    }

    /// Peeks the next token without moving the cursor.
    pub(super) fn peek(&mut self) -> Option<Result<Token, ParseError>> {
        match self.peeked.pop_front() {
            Some(None) => {
                self.peeked.push_front(None); // Push the value back onto the peeked stack.
                return None;
            }
            Some(Some(token)) => {
                self.peeked.push_front(Some(token.clone())); // Push the value back onto the peeked stack.
                return Some(Ok(token));
            }
            None => {} // No value has been peeked ahead already so need to go get the next value.
        }

        match self.next() {
            Some(Ok(token)) => {
                self.peeked.push_back(Some(token.clone()));
                Some(Ok(token))
            }
            Some(Err(e)) => Some(Err(e)),
            None => {
                self.peeked.push_back(None);
                None
            }
        }
    }

    pub(super) fn peek_more(&mut self, skip: usize) -> Option<Result<Token, ParseError>> {
        if skip != 1 {
            // I don't believe we ever need to skip more than a single token?
            unimplemented!("Attempting to peek ahead more than a single token");
        }

        // Add elements to the peeked buffer upto the amount required to skip the given amount ahead.
        while self.peeked.len() < skip + 1 {
            match self.lexer.next() {
                Some(Ok(token)) => self.peeked.push_back(Some(token.clone())),
                Some(Err(e)) => return Some(Err(ParseError::lex(e))),
                None => self.peeked.push_back(None),
            }
        }

        let temp = self.peeked.pop_front().unwrap();
        let ret = self.peeked.pop_front().unwrap();

        self.peeked.push_front(ret.clone());
        self.peeked.push_front(temp);

        ret.map(|token| Ok(token))
    }

    pub(super) fn push_back(&mut self, token: Token) {
        self.peeked.push_front(Some(token));
    }

    // /// Moves the cursor to the previous token and returns the token.
    // pub(super) fn back(&mut self) -> Option<Result<Token, ParseError>> {
    //     unimplemented!();

    //     // debug_assert!(
    //     //     self.back_queue.len() > 0,
    //     //     "cannot go back in a cursor that is at the beginning of the list of tokens"
    //     // );

    //     // let token = self.back_queue.pop_back().unwrap();

    //     // self.peeked.push_front(token.clone());

    //     // token.map(|t| Ok(t))

    //     // unimplemented!();

    //     // debug_assert!(
    //     //     self.pos > 0,
    //     //     "cannot go back in a cursor that is at the beginning of the list of tokens"
    //     // );

    //     // self.pos -= 1;
    //     // while self
    //     //     .tokens
    //     //     .get(self.pos - 1)
    //     //     .expect("token disappeared")
    //     //     .kind
    //     //     == TokenKind::LineTerminator
    //     //     && self.pos > 0
    //     // {
    //     //     self.pos -= 1;
    //     // }
    // }

    // /// Peeks the previous token without moving the cursor.
    // pub(super) fn peek_prev(&self) -> Option<Result<&Token, ParseError>> {
    //     unimplemented!();
    //     // if self.pos == 0 {
    //     //     None
    //     // } else {
    //     //     let mut back = 1;
    //     //     let mut tok = self.tokens.get(self.pos - back).expect("token disappeared");
    //     //     while self.pos >= back && tok.kind == TokenKind::LineTerminator {
    //     //         back += 1;
    //     //         tok = self.tokens.get(self.pos - back).expect("token disappeared");
    //     //     }

    //     //     if back == self.pos {
    //     //         None
    //     //     } else {
    //     //         Some(tok)
    //     //     }
    //     // }
    // }

    /// Returns an error if the next token is not of kind `kind`.
    ///
    /// Note: it will consume the next token only if the next token is the expected type.
    pub(super) fn expect<K>(&mut self, kind: K, context: &'static str) -> Result<Token, ParseError>
    where
        K: Into<TokenKind>,
    {
        let next_token = self.peek().ok_or(ParseError::AbruptEnd)??;
        let kind = kind.into();

        if next_token.kind() == &kind {
            self.next();
            Ok(next_token)
        } else {
            Err(ParseError::expected(
                vec![kind],
                next_token.clone(),
                context,
            ))
        }
    }

    /// It will peek for the next token, to see if it's a semicolon.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    pub(super) fn peek_semicolon(&mut self) -> Result<(bool, Option<Token>), ParseError> {
        match self.peek() {
            Some(Ok(tk)) => {
                match tk.kind() {
                    TokenKind::Punctuator(Punctuator::Semicolon) => Ok((true, Some(tk))),
                    TokenKind::LineTerminator | TokenKind::Punctuator(Punctuator::CloseBlock) => {
                        Ok((true, Some(tk)))
                    }
                    _ => Ok((false, Some(tk))),
                }
            }
            Some(Err(e)) => Err(e),
            None => Ok((true, None)),
        }
    }

    /// It will check if the next token is a semicolon.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    pub(super) fn expect_semicolon(
        &mut self,
        context: &'static str,
    ) -> Result<Option<Token>, ParseError> {

        match self.peek_semicolon()? {
            (true, Some(tk)) => match tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon) | TokenKind::LineTerminator => {
                    self.next(); // Consume the token.
                    Ok(Some(tk))
                }
                _ => Ok(Some(tk)),
            },
            (true, None) => Ok(None),
            (false, Some(tk)) => Err(ParseError::expected(
                vec![TokenKind::Punctuator(Punctuator::Semicolon)],
                tk.clone(),
                context,
            )),
            (false, None) => unreachable!(),
        }
    }

    /// It will make sure that the next token is not a line terminator.
    ///
    /// It expects that the token stream does not end here.
    pub(super) fn peek_expect_no_lineterminator(&mut self, skip: usize) -> Result<(), ParseError> {
        let token = if skip == 0 {
            self.peek()
        } else {
            self.peek_more(skip)
        };

        match token {
            Some(Ok(t)) => {
                if t.kind() == &TokenKind::LineTerminator {
                    Err(ParseError::unexpected(t, None))
                } else {
                    Ok(())
                }
            }
            Some(Err(e)) => Err(e),
            None => Err(ParseError::AbruptEnd),
        }
    }

    /// Advance the cursor to the next token and retrieve it, only if it's of `kind` type.
    ///
    /// When the next token is a `kind` token, get the token, otherwise return `None`.
    pub(super) fn next_if<K>(&mut self, kind: K) -> Option<Result<Token, ParseError>>
    where
        K: Into<TokenKind>,
    {
        match self.peek() {
            Some(Ok(token)) => {
                if token.kind() == &kind.into() {
                    self.next()
                } else {
                    None
                }
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    /// Advance the cursor to skip 0, 1 or more line terminators.
    pub(super) fn skip_line_terminators(&mut self) {
        while self.next_if(TokenKind::LineTerminator).is_some() {}
    }
}
