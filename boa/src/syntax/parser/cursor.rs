//! Cursor implementation for the parser.

use crate::syntax::ast::token::Token;

/// Token cursor.
///
/// This internal structure gives basic testable operations to the parser.
#[derive(Debug, Clone, Default)]
pub struct Cursor {
    /// The tokens being input.
    tokens: Vec<Token>,
    /// The current position within the tokens.
    pos: usize,
}

impl Cursor {
    /// Creates a new cursor.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            ..Self::default()
        }
    }

    /// Moves the cursor to the next token and returns the token.
    pub fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);

        if self.pos != self.tokens.len() {
            self.pos += 1;
        }

        token
    }

    /// Moves the cursor to the next token after skipping tokens based on the predicate.
    pub fn next_skip<P>(&mut self, skip: P) -> Option<&Token>
    where
        P: FnMut(&Token) -> bool,
    {
        while let Some(token) = self.next() {
            if !skip(token) {
                return Some(token);
            }
        }
        None
    }

    /// Peeks the next token without moving the cursor.
    pub fn peek(&self, skip: usize) -> Option<&Token> {
        self.tokens.get(self.pos + skip)
    }

    /// Peeks the next token after skipping tokens based on the predicate.
    pub fn peek_skip<P>(&self, skip: P) -> Option<&Token>
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
    pub fn prev(&mut self) -> Option<&Token> {
        if self.pos == 0 {
            None
        } else {
            self.pos -= 1;
            self.tokens.get(self.pos)
        }
    }

    /// Peeks the previous token without moving the cursor.
    pub fn peek_prev(&self) -> Option<&Token> {
        if self.pos == 0 {
            None
        } else {
            self.tokens.get(self.pos - 1)
        }
    }
}
