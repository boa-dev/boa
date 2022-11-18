//! Cursor implementation for the parser.
mod buffered_lexer;

use crate::{
    lexer::{InputElement, Lexer, Token, TokenKind},
    parser::{statement::PrivateElement, OrAbrupt, ParseResult},
    Error,
};
use boa_ast::{Position, Punctuator};
use boa_interner::{Interner, Sym};
use buffered_lexer::BufferedLexer;
use rustc_hash::FxHashMap;
use std::io::Read;

/// The result of a peek for a semicolon.
#[derive(Debug)]
pub(super) enum SemicolonResult<'s> {
    Found(Option<&'s Token>),
    NotFound(&'s Token),
}

/// Token cursor.
///
/// This internal structure gives basic testable operations to the parser.
#[derive(Debug)]
pub(super) struct Cursor<R> {
    buffered_lexer: BufferedLexer<R>,

    /// Tracks the private identifiers used in code blocks.
    private_environments_stack: Vec<FxHashMap<Sym, Position>>,

    /// Tracks if the cursor is in a arrow function declaration.
    arrow: bool,

    /// Indicate if the cursor is used in `JSON.parse`.
    json_parse: bool,
}

impl<R> Cursor<R>
where
    R: Read,
{
    /// Creates a new cursor with the given reader.
    #[inline]
    pub(super) fn new(reader: R) -> Self {
        Self {
            buffered_lexer: Lexer::new(reader).into(),
            private_environments_stack: Vec::new(),
            arrow: false,
            json_parse: false,
        }
    }

    #[inline]
    pub(super) fn set_goal(&mut self, elm: InputElement) {
        self.buffered_lexer.set_goal(elm);
    }

    #[inline]
    pub(super) fn lex_regex(
        &mut self,
        start: Position,
        interner: &mut Interner,
    ) -> ParseResult<Token> {
        self.buffered_lexer.lex_regex(start, interner)
    }

    #[inline]
    pub(super) fn lex_template(
        &mut self,
        start: Position,
        interner: &mut Interner,
    ) -> ParseResult<Token> {
        self.buffered_lexer.lex_template(start, interner)
    }

    /// Advances the cursor and returns the next token.
    #[inline]
    pub(super) fn next(&mut self, interner: &mut Interner) -> ParseResult<Option<Token>> {
        self.buffered_lexer.next(true, interner)
    }

    /// Advances the cursor without returning the next token.
    ///
    /// # Panics
    ///
    /// This function will panic if there is no further token in the cursor.
    #[inline]
    #[track_caller]
    pub(super) fn advance(&mut self, interner: &mut Interner) {
        self.next(interner)
            .expect("tried to advance cursor, but the buffer was empty");
    }

    /// Peeks a future token, without consuming it or advancing the cursor.
    ///
    /// You can skip some tokens with the `skip_n` option.
    #[inline]
    pub(super) fn peek(
        &mut self,
        skip_n: usize,
        interner: &mut Interner,
    ) -> ParseResult<Option<&Token>> {
        self.buffered_lexer.peek(skip_n, true, interner)
    }

    /// Gets the current strict mode for the cursor.
    #[inline]
    pub(super) const fn strict_mode(&self) -> bool {
        self.buffered_lexer.strict_mode()
    }

    /// Sets the strict mode to strict or non-strict.
    #[inline]
    pub(super) fn set_strict_mode(&mut self, strict_mode: bool) {
        self.buffered_lexer.set_strict_mode(strict_mode);
    }

    /// Returns if the cursor is currently in an arrow function declaration.
    #[inline]
    pub(super) const fn arrow(&self) -> bool {
        self.arrow
    }

    /// Set if the cursor is currently in a arrow function declaration.
    #[inline]
    pub(super) fn set_arrow(&mut self, arrow: bool) {
        self.arrow = arrow;
    }

    /// Returns if the cursor is currently used in `JSON.parse`.
    #[inline]
    pub(super) const fn json_parse(&self) -> bool {
        self.json_parse
    }

    /// Set if the cursor is currently used in `JSON.parse`.
    #[inline]
    pub(super) fn set_json_parse(&mut self, json_parse: bool) {
        self.json_parse = json_parse;
    }

    /// Push a new private environment.
    #[inline]
    pub(super) fn push_private_environment(&mut self) {
        let new = FxHashMap::default();
        self.private_environments_stack.push(new);
    }

    /// Push a used private identifier.
    #[inline]
    pub(super) fn push_used_private_identifier(
        &mut self,
        identifier: Sym,
        position: Position,
    ) -> ParseResult<()> {
        self.private_environments_stack.last_mut().map_or_else(
            || {
                Err(Error::general(
                    "private identifier declared outside of class",
                    position,
                ))
            },
            |env| {
                env.entry(identifier).or_insert(position);
                Ok(())
            },
        )
    }

    /// Pop the last private environment.
    ///
    /// This function takes the private element names of the current class.
    /// If a used private identifier is not declared, this throws a syntax error.
    #[inline]
    pub(super) fn pop_private_environment(
        &mut self,
        identifiers: &FxHashMap<Sym, PrivateElement>,
    ) -> ParseResult<()> {
        let last = self
            .private_environments_stack
            .pop()
            .expect("private environment must exist");
        for (identifier, position) in &last {
            if !identifiers.contains_key(identifier) {
                if let Some(outer) = self.private_environments_stack.last_mut() {
                    outer.insert(*identifier, *position);
                } else {
                    return Err(Error::general(
                        "private identifier must be declared",
                        *position,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Returns an error if the next token is not of kind `kind`.
    #[inline]
    pub(super) fn expect<K>(
        &mut self,
        kind: K,
        context: &'static str,
        interner: &mut Interner,
    ) -> ParseResult<Token>
    where
        K: Into<TokenKind>,
    {
        let next_token = self.next(interner).or_abrupt()?;
        let kind = kind.into();

        if next_token.kind() == &kind {
            Ok(next_token)
        } else {
            Err(Error::expected(
                [kind.to_string(interner)],
                next_token.to_string(interner),
                next_token.span(),
                context,
            ))
        }
    }

    /// It will peek for the next token, to see if it's a semicolon.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    #[inline]
    pub(super) fn peek_semicolon(
        &mut self,
        interner: &mut Interner,
    ) -> ParseResult<SemicolonResult<'_>> {
        self.buffered_lexer.peek(0, false, interner)?.map_or(
            Ok(SemicolonResult::Found(None)),
            |tk| match tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon | Punctuator::CloseBlock)
                | TokenKind::LineTerminator => Ok(SemicolonResult::Found(Some(tk))),
                _ => Ok(SemicolonResult::NotFound(tk)),
            },
        )
    }

    /// Consumes the next token if it is a semicolon, or returns a `Errpr` if it's not.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    #[inline]
    pub(super) fn expect_semicolon(
        &mut self,
        context: &'static str,
        interner: &mut Interner,
    ) -> ParseResult<()> {
        match self.peek_semicolon(interner)? {
            SemicolonResult::Found(Some(tk)) => match *tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon) | TokenKind::LineTerminator => {
                    let _next = self.buffered_lexer.next(false, interner)?;
                    Ok(())
                }
                _ => Ok(()),
            },
            SemicolonResult::Found(None) => Ok(()),
            SemicolonResult::NotFound(tk) => Err(Error::expected(
                [";".to_owned()],
                tk.to_string(interner),
                tk.span(),
                context,
            )),
        }
    }

    /// It will make sure that the peeked token (skipping n tokens) is not a line terminator.
    ///
    /// It expects that the token stream does not end here.
    ///
    /// This is just syntatic sugar for a `.peek(skip_n)` call followed by a check that the result
    /// is not a line terminator or `None`.
    #[inline]
    pub(super) fn peek_expect_no_lineterminator(
        &mut self,
        skip_n: usize,
        context: &'static str,
        interner: &mut Interner,
    ) -> ParseResult<&Token> {
        let tok = self
            .buffered_lexer
            .peek(skip_n, false, interner)
            .or_abrupt()?;

        if tok.kind() == &TokenKind::LineTerminator {
            Err(Error::unexpected(
                tok.to_string(interner),
                tok.span(),
                context,
            ))
        } else {
            Ok(tok)
        }
    }

    /// Check if the peeked token is a line terminator.
    #[inline]
    pub(super) fn peek_is_line_terminator(
        &mut self,
        skip_n: usize,
        interner: &mut Interner,
    ) -> ParseResult<Option<bool>> {
        self.buffered_lexer
            .peek(skip_n, false, interner)?
            .map_or(Ok(None), |t| {
                Ok(Some(t.kind() == &TokenKind::LineTerminator))
            })
    }

    /// Advance the cursor to the next token and retrieve it, only if it's of `kind` type.
    ///
    /// When the next token is a `kind` token, get the token, otherwise return `None`.
    ///
    /// No next token also returns None.
    #[inline]
    pub(super) fn next_if<K>(
        &mut self,
        kind: K,
        interner: &mut Interner,
    ) -> ParseResult<Option<Token>>
    where
        K: Into<TokenKind>,
    {
        Ok(if let Some(token) = self.peek(0, interner)? {
            if token.kind() == &kind.into() {
                self.next(interner)?
            } else {
                None
            }
        } else {
            None
        })
    }
}
