//! Cursor implementation for the parser.

use super::ParseError;
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::Punctuator,
        lexer::{InputElement, Lexer, Position, Token, TokenKind},
    },
};
use std::io::Read;

/// The fixed size of the buffer used for storing values that are peeked ahead.
const PEEK_BUF_SIZE: usize = 4;

/// Token cursor.
///
/// This internal structure gives basic testable operations to the parser.
#[derive(Debug)]
pub(super) struct Cursor<R> {
    lexer: Lexer<R>,
    peeked: [Option<Token>; PEEK_BUF_SIZE],
    front_index: usize,
    back_index: usize,
}

impl<R> Cursor<R>
where
    R: Read,
{
    /// Creates a new cursor.
    #[inline]
    pub(super) fn new(reader: R) -> Self {
        Self {
            lexer: Lexer::new(reader),
            peeked: [None::<Token>, None::<Token>, None::<Token>, None::<Token>],
            front_index: 0,
            back_index: 0,
        }
    }

    /// Sets the goal symbol for the lexer.
    #[inline]
    pub(super) fn set_goal(&mut self, elm: InputElement) {
        let _timer = BoaProfiler::global().start_event("cursor::set_goal()", "Parsing");
        self.lexer.set_goal(elm)
    }

    /// Lexes the next tokens as a regex assuming that the starting '/' has already been consumed.
    #[inline]
    pub(super) fn lex_regex(&mut self, start: Position) -> Result<Token, ParseError> {
        let _timer = BoaProfiler::global().start_event("cursor::lex_regex()", "Parsing");
        self.set_goal(InputElement::RegExp);
        self.lexer.lex_slash_token(start).map_err(|e| e.into())
    }

    /// Moves the cursor to the next token and returns the token.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    #[inline]
    pub(super) fn next(&mut self, skip_line_terminators: bool) -> Result<Option<Token>, ParseError> {
        let _timer = BoaProfiler::global().start_event("cursor::next()", "Parsing");

        if self.front_index == self.back_index {
            // No value has been peeked ahead already so need to go get the next value.
            Ok(self.lexer.next(skip_line_terminators)?)
        } else {
            println!("Next using cached value");
            let val = self.peeked[self.back_index].take();
            self.back_index = (self.back_index + 1) % PEEK_BUF_SIZE;

            if skip_line_terminators {
                if let Some(t) = val {
                    if *t.kind() == TokenKind::LineTerminator {
                        println!("Skipping line terminator at next()");
                        self.next(skip_line_terminators)
                    } else {
                        Ok(Some(t))
                    }
                } else {
                    Ok(None)
                }
            } else {
                Ok(val)
            }
        }
    }

    /// Peeks the next token without moving the cursor.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    pub(super) fn peek(&mut self, skip_line_terminators: bool) -> Result<Option<Token>, ParseError> {
        let _timer = BoaProfiler::global().start_event("cursor::peek()", "Parsing");
        if self.front_index == self.back_index {
            // No value has been peeked ahead already so need to go get the next value.

            let next = self.lexer.next(skip_line_terminators)?;
            self.peeked[self.front_index] = next;
            self.front_index = (self.front_index + 1) % PEEK_BUF_SIZE;
        }

        let val = self.peeked[self.back_index].clone();

        if skip_line_terminators {
            if let Some(token) =  val {
                if token.kind() == &TokenKind::LineTerminator {
                    println!("Removing line terminator from peek");
                    self.peeked[self.back_index].take();
                    self.back_index = (self.back_index + 1) % PEEK_BUF_SIZE;
                    self.peek(skip_line_terminators)
                } else {
                    Ok(Some(token))
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(val)
        }
    }

    /// Peeks the token after the next token.
    /// i.e. if there are tokens A, B, C and peek() returns A then peek_skip() will return B.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    pub(super) fn peek_skip(&mut self, skip_line_terminators: bool) -> Result<Option<Token>, ParseError> {
        let _timer = BoaProfiler::global().start_event("cursor::peek_skip()", "Parsing");
        if self.front_index == self.back_index {
            // No value has been peeked ahead already so need to go get the next value.

            self.peeked[self.front_index] = self.lexer.next(skip_line_terminators)?;
            self.front_index = (self.front_index + 1) % PEEK_BUF_SIZE;

            let index = self.front_index;

            self.peeked[self.front_index] = self.lexer.next(skip_line_terminators)?;
            self.front_index = (self.front_index + 1) % PEEK_BUF_SIZE;

            Ok(self.peeked[index].clone())
        } else if ((self.back_index + 1) % PEEK_BUF_SIZE) == self.front_index {
            // Indicates only a single value has been peeked ahead already
            let index = self.front_index;

            self.peeked[self.front_index] = self.lexer.next(skip_line_terminators)?;
            self.front_index = (self.front_index + 1) % PEEK_BUF_SIZE;

            Ok(self.peeked[index].clone())
        } else {
            Ok(self.peeked[(self.back_index + 1) % PEEK_BUF_SIZE].clone())
        }
    }

    /// Takes the given token and pushes it back onto the parser token queue.
    ///
    /// Note: it pushes it at the the front so the token will be returned on next .peek().
    #[inline]
    pub(super) fn push_back(&mut self, token: Token) {
        if ((self.front_index + 1) % PEEK_BUF_SIZE) == self.back_index {
            // Indicates that the buffer already contains a pushed back value and there is therefore
            // no space for another.
            unimplemented!("Push back more than once");
        }

        if self.front_index == self.back_index {
            // No value peeked already.
            self.peeked[self.front_index] = Some(token);
            self.front_index = (self.front_index + 1) % PEEK_BUF_SIZE;
        } else {
            if self.back_index == 0 {
                self.back_index = PEEK_BUF_SIZE - 1;
            } else {
                self.back_index = self.back_index - 1;
            }

            self.peeked[self.back_index] = Some(token);
        }
    }

    /// Returns an error if the next token is not of kind `kind`.
    ///
    /// Note: it will consume the next token only if the next token is the expected type.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    pub(super) fn expect<K>(&mut self, kind: K, context: &'static str, skip_line_terminators: bool) -> Result<Token, ParseError>
    where
        K: Into<TokenKind>,
    {
        let next_token = self.peek(skip_line_terminators)?.ok_or(ParseError::AbruptEnd)?;
        let kind = kind.into();

        if next_token.kind() == &kind {
            self.next(skip_line_terminators)?.expect("Token vanished");
            Ok(next_token)
        } else {
            Err(ParseError::expected(vec![kind], next_token, context))
        }
    }

    /// It will peek for the next token, to see if it's a semicolon.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    pub(super) fn peek_semicolon(&mut self) -> Result<(bool, Option<Token>), ParseError> {
        match self.peek(false)? {
            Some(tk) => match tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon) => Ok((true, Some(tk))),
                TokenKind::LineTerminator | TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    Ok((true, Some(tk)))
                }
                _ => Ok((false, Some(tk))),
            },
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
                    self.next(false)?.expect("Token vanished"); // Consume the token.
                    Ok(Some(tk))
                }
                _ => Ok(Some(tk)),
            },
            (true, None) => Ok(None),
            (false, Some(tk)) => Err(ParseError::expected(
                vec![TokenKind::Punctuator(Punctuator::Semicolon)],
                tk,
                context,
            )),
            (false, None) => unreachable!(),
        }
    }

    /// It will make sure that the next token is not a line terminator.
    ///
    /// It expects that the token stream does not end here.
    ///
    /// If skip is true then the token after the peek() token is checked instead.
    pub(super) fn peek_expect_no_lineterminator(&mut self, skip: bool) -> Result<(), ParseError> {
        let token = if skip {
            self.peek_skip(false)?
        } else {
            self.peek(false)?
        };

        if let Some(t) = token {
            if t.kind() == &TokenKind::LineTerminator {
                Err(ParseError::unexpected(t, None))
            } else {
                Ok(())
            }
        } else {
            Err(ParseError::AbruptEnd)
        }
    }

    /// Advance the cursor to the next token and retrieve it, only if it's of `kind` type.
    ///
    /// When the next token is a `kind` token, get the token, otherwise return `None`.
    ///
    /// No next token also returns None.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    pub(super) fn next_if<K>(&mut self, kind: K, skip_line_terminators: bool) -> Result<Option<Token>, ParseError>
    where
        K: Into<TokenKind>,
    {
        Ok(if let Some(token) = self.peek(skip_line_terminators)? {
            if token.kind() == &kind.into() {
                self.next(skip_line_terminators)?
            } else {
                None
            }
        } else {
            None
        })
    }

    /// Advance the cursor to skip 0, 1 or more line terminators.
    #[inline]
    pub(super) fn skip_line_terminators(&mut self) -> Result<(), ParseError> {
        while self.next_if(TokenKind::LineTerminator, false)?.is_some() {}
        Ok(())
    }
}
