//! Cursor implementation for the parser.
mod buffered_lexer;

use crate::{
    lexer::{InputElement, Lexer, Token, TokenKind},
    parser::{OrAbrupt, ParseResult},
    Error,
};
use boa_ast::{Position, Punctuator};
use boa_interner::Interner;
use buffered_lexer::BufferedLexer;
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

    /// Tracks if the cursor is in a arrow function declaration.
    arrow: bool,

    /// Indicate if the cursor is used in `JSON.parse`.
    json_parse: bool,

    /// A unique identifier for each parser instance.
    /// This is used to generate unique identifiers tagged template literals.
    identifier: u32,

    /// Tracks the number of tagged templates that are currently being parsed.
    tagged_templates_count: u32,
}

impl<R> Cursor<R>
where
    R: Read,
{
    /// Creates a new cursor with the given reader.
    pub(super) fn new(reader: R) -> Self {
        Self {
            buffered_lexer: Lexer::new(reader).into(),
            arrow: false,
            json_parse: false,
            identifier: 0,
            tagged_templates_count: 0,
        }
    }

    /// Sets the goal symbol of the cursor to `Module`.
    pub(super) fn set_module(&mut self) {
        self.buffered_lexer.set_module(true);
    }

    /// Returns `true` if the cursor is currently parsing a `Module`.
    pub(super) const fn module(&self) -> bool {
        self.buffered_lexer.module()
    }

    pub(super) fn set_goal(&mut self, elm: InputElement) {
        self.buffered_lexer.set_goal(elm);
    }

    pub(super) fn lex_regex(
        &mut self,
        start: Position,
        interner: &mut Interner,
    ) -> ParseResult<Token> {
        self.buffered_lexer.lex_regex(start, interner)
    }

    pub(super) fn lex_template(
        &mut self,
        start: Position,
        interner: &mut Interner,
    ) -> ParseResult<Token> {
        self.buffered_lexer.lex_template(start, interner)
    }

    /// Advances the cursor and returns the next token.
    pub(super) fn next(&mut self, interner: &mut Interner) -> ParseResult<Option<Token>> {
        self.buffered_lexer.next(true, interner)
    }

    /// Advances the cursor without returning the next token.
    ///
    /// # Panics
    ///
    /// This function will panic if there is no further token in the cursor.
    #[track_caller]
    pub(super) fn advance(&mut self, interner: &mut Interner) {
        self.next(interner)
            .expect("tried to advance cursor, but the buffer was empty");
    }

    /// Peeks a future token, without consuming it or advancing the cursor.
    ///
    /// You can skip some tokens with the `skip_n` option.
    pub(super) fn peek(
        &mut self,
        skip_n: usize,
        interner: &mut Interner,
    ) -> ParseResult<Option<&Token>> {
        self.buffered_lexer.peek(skip_n, true, interner)
    }

    /// Gets the current strict mode for the cursor.
    pub(super) const fn strict(&self) -> bool {
        self.buffered_lexer.strict()
    }

    /// Sets the strict mode to strict or non-strict.
    pub(super) fn set_strict(&mut self, strict: bool) {
        self.buffered_lexer.set_strict(strict);
    }

    /// Returns if the cursor is currently in an arrow function declaration.
    pub(super) const fn arrow(&self) -> bool {
        self.arrow
    }

    /// Set if the cursor is currently in a arrow function declaration.
    pub(super) fn set_arrow(&mut self, arrow: bool) {
        self.arrow = arrow;
    }

    /// Returns if the cursor is currently used in `JSON.parse`.
    pub(super) const fn json_parse(&self) -> bool {
        self.json_parse
    }

    /// Set if the cursor is currently used in `JSON.parse`.
    pub(super) fn set_json_parse(&mut self, json_parse: bool) {
        self.json_parse = json_parse;
    }

    /// Set the identifier of the cursor.
    #[inline]
    pub(super) fn set_identifier(&mut self, identifier: u32) {
        self.identifier = identifier;
    }

    /// Get the identifier for a tagged template.
    #[inline]
    pub(super) fn tagged_template_identifier(&mut self) -> u64 {
        self.tagged_templates_count += 1;

        let identifier = u64::from(self.identifier);
        let count = u64::from(self.tagged_templates_count);

        (count << 32) | identifier
    }

    /// Returns an error if the next token is not of kind `kind`.
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
