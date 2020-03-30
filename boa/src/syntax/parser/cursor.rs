//! Cursor implementation for the parser.

use crate::syntax::ast::token::Token;

/// Token cursor.
///
/// This internal structure gives basic testable operations to the parser.
#[derive(Debug, Clone, Default)]
pub(super) struct Cursor {
    /// The tokens being input.
    tokens: Vec<Token>,
    /// The current position within the tokens.
    pos: usize,
}

impl Cursor {
    /// Creates a new cursor.
    pub(super) fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            ..Self::default()
        }
    }

    /// Retrieves the current position of the cursor in the token stream.
    pub(super) fn pos(&self) -> usize {
        self.pos
    }

    /// Moves the cursor to the given position.
    ///
    /// This is intended to be used *always* with `Cursor::pos()`.
    ///
    /// # Example:
    /// ```
    /// # let mut cursor = Cursor::new(Vec::new());
    /// let pos_save = cursor.pos();
    /// // Do some stuff that might change the cursor position...
    /// cursor.seek(pos_save);
    /// ```
    pub(super) fn seek(&mut self, pos: usize) {
        self.pos = pos
    }

    /// Moves the cursor to the next token and returns the token.
    pub(super) fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);

        if self.pos != self.tokens.len() {
            self.pos += 1;
        }

        token
    }

    /// Moves the cursor to the next token after skipping tokens based on the predicate.
    pub(super) fn next_skip<P>(&mut self, mut skip: P) -> Option<&Token>
    where
        P: FnMut(&Token) -> bool,
    {
        while let Some(token) = self.tokens.get(self.pos) {
            self.pos += 1;

            if !skip(token) {
                return Some(token);
            }
        }
        None
    }

    /// Peeks the next token without moving the cursor.
    pub(super) fn peek(&self, skip: usize) -> Option<&Token> {
        self.tokens.get(self.pos + skip)
    }

    /// Peeks the next token after skipping tokens based on the predicate.
    pub(super) fn peek_skip<P>(&self, mut skip: P) -> Option<&Token>
    where
        P: FnMut(&Token) -> bool,
    {
        let mut current = self.pos;
        while let Some(token) = self.tokens.get(current) {
            if !skip(token) {
                return Some(token);
            }
            current += 1;
        }

        None
    }

    /// Moves the cursor to the previous token and returns the token.
    pub(super) fn back(&mut self) {
        assert!(
            self.pos > 0,
            "cannot go back in a cursor that is at the beginning of the list of tokens"
        );

        self.pos -= 1;
    }

    /// Peeks the previous token without moving the cursor.
    pub(super) fn peek_prev(&self) -> Option<&Token> {
        if self.pos == 0 {
            None
        } else {
            self.tokens.get(self.pos - 1)
        }
    }
}
