//! Boa parser implementation.

mod cursor;
#[cfg(test)]
mod tests;

use crate::syntax::ast::{
    constant::Const,
    keyword::Keyword,
    node::{FormalParameter, FormalParameters, MethodDefinitionKind, Node, PropertyDefinition},
    op::{AssignOp, BinOp, NumOp, UnaryOp},
    pos::Position,
    punc::Punctuator,
    token::{Token, TokenKind},
};
use cursor::Cursor;
use std::fmt;

/// `ParseError` is an enum which represents errors encounted during parsing an expression
#[derive(Debug, Clone)]
pub enum ParseError {
    /// When it expected a certain kind of token, but got another as part of something
    Expected(Vec<TokenKind>, Token, Option<&'static str>),
    /// When it expected a certain expression, but got another
    ExpectedExpr(&'static str, Node, Position),
    /// When it didn't expect this keyword
    UnexpectedKeyword(Keyword, Position),
    /// When a token is unexpected
    Unexpected(Token, Option<&'static str>),
    /// When there is an abrupt end to the parsing
    AbruptEnd,
    /// Out of range error, attempting to set a position where there is no token
    RangeError,
    /// Catch all General Error
    General(&'static str, Option<Position>),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Expected(expected, actual, routine) => write!(
                f,
                "Expected {}, got '{}'{} at line {}, col {}",
                if expected.len() == 1 {
                    format!(
                        "token '{}'",
                        expected.first().map(TokenKind::to_string).unwrap()
                    )
                } else {
                    format!(
                        "one of {}",
                        expected
                            .iter()
                            .enumerate()
                            .map(|(i, t)| {
                                format!(
                                    "{}'{}'",
                                    if i == 0 {
                                        ""
                                    } else if i == expected.len() - 1 {
                                        " or "
                                    } else {
                                        ", "
                                    },
                                    t
                                )
                            })
                            .collect::<String>()
                    )
                },
                actual,
                if let Some(routine) = routine {
                    format!(" in {}", routine)
                } else {
                    String::new()
                },
                actual.pos.line_number,
                actual.pos.column_number
            ),
            ParseError::ExpectedExpr(expected, actual, pos) => write!(
                f,
                "Expected expression '{}', got '{}' at line {}, col {}",
                expected, actual, pos.line_number, pos.column_number
            ),
            ParseError::UnexpectedKeyword(keyword, pos) => write!(
                f,
                "Unexpected keyword: '{}' at line {}, col {}",
                keyword, pos.line_number, pos.column_number
            ),
            ParseError::Unexpected(tok, msg) => write!(
                f,
                "Unexpected Token '{}'{} at line {}, col {}",
                tok,
                if let Some(m) = msg {
                    format!(", {}", m)
                } else {
                    String::new()
                },
                tok.pos.line_number,
                tok.pos.column_number
            ),
            ParseError::AbruptEnd => write!(f, "Abrupt End"),
            ParseError::General(msg, pos) => write!(
                f,
                "{}{}",
                msg,
                if let Some(pos) = pos {
                    format!(" at line {}, col {}", pos.line_number, pos.column_number)
                } else {
                    String::new()
                }
            ),
            ParseError::RangeError => write!(f, "RangeError!"),
        }
    }
}

pub type ParseResult = Result<Node, ParseError>;

#[derive(Debug)]
pub struct Parser<'a> {
    /// Cursor in the parser, the internal structure used to read tokens.
    cursor: Cursor<'a>,
}

macro_rules! expression { ( $name:ident, $lower:ident, [ $( $op:path ),* ] ) => {
    fn $name (&mut self) -> ParseResult {
        let mut lhs = self. $lower ()?;
        while let Some(tok) = self.peek_skip_lineterminator() {
            match tok.kind {
                // Parse assign expression
                TokenKind::Punctuator(ref op) if op == &Punctuator::Assign => {
                    let _ = self.next_skip_lineterminator().expect("token disappeared");
                    lhs = Node::Assign(
                        Box::new(lhs),
                        Box::new(self. $lower ()?)
                    )
                }
                TokenKind::Punctuator(ref op) if $( op == &$op )||* => {
                    let _ = self.next_skip_lineterminator().expect("token disappeared");
                    lhs = Node::BinOp(
                        op.as_binop().unwrap(),
                        Box::new(lhs),
                        Box::new(self. $lower ()?)
                    )
                }
                _ => break
            }
        }
        Ok(lhs)
    }
} }

impl<'a> Parser<'a> {
    /// Create a new parser, using `tokens` as input
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            cursor: Cursor::new(tokens),
        }
    }

    /// Parse all expressions in the token array
    pub fn parse_all(&mut self) -> ParseResult {
        self.read_statement_list()
    }

    /// consume the next token and increment position
    fn get_next_token(&mut self) -> Result<&Token, ParseError> {
        self.cursor.next().ok_or(ParseError::AbruptEnd)
    }

    /// Peek the next token skipping line terminators.
    pub fn peek_skip_lineterminator(&mut self) -> Option<&'a Token> {
        self.cursor
            .peek_skip(|tk| tk.kind == TokenKind::LineTerminator)
    }

    /// Consume the next token skipping line terminators.
    pub fn next_skip_lineterminator(&mut self) -> Option<&'a Token> {
        self.cursor
            .next_skip(|tk| tk.kind == TokenKind::LineTerminator)
    }

    /// Advance the cursor to the next token and retrieve it, only if it's of `kind` type.
    ///
    /// When the next token is a `kind` token, get the token, otherwise return `None`.
    fn next_if(&mut self, kind: TokenKind) -> Option<&'a Token> {
        let next_token = self.cursor.peek(0)?;

        if next_token.kind == kind {
            self.cursor.next()
        } else {
            None
        }
    }

    /// Advance the cursor to the next token and retrieve it, only if it's of `kind` type.
    ///
    /// When the next token is a `kind` token, get the token, otherwise return `None`. This
    /// function skips line terminators.
    fn next_if_skip_lineterminator(&mut self, kind: TokenKind) -> Option<&'a Token> {
        let next_token = self.peek_skip_lineterminator()?;

        if next_token.kind == kind {
            self.next_skip_lineterminator()
        } else {
            None
        }
    }

    /// Returns an error if the next Punctuator is not `tk`
    fn expect(&mut self, kind: TokenKind, routine: Option<&'static str>) -> Result<(), ParseError> {
        let next_token = self.cursor.next().ok_or(ParseError::AbruptEnd)?;
        if next_token.kind == kind {
            Ok(())
        } else {
            Err(ParseError::Expected(
                vec![kind],
                next_token.clone(),
                routine,
            ))
        }
    }

    /// Returns an error if the next symbol is not `tk`
    fn expect_no_lineterminator(
        &mut self,
        kind: TokenKind,
        routine: Option<&'static str>,
    ) -> Result<(), ParseError> {
        let next_token = self
            .cursor
            .next_skip(|tk| tk.kind == TokenKind::LineTerminator)
            .ok_or(ParseError::AbruptEnd)?;

        if next_token.kind == kind {
            Ok(())
        } else {
            Err(ParseError::Expected(
                vec![kind],
                next_token.clone(),
                routine,
            ))
        }
    }

    /// Returns an error if the next symbol is not the punctuator `p`
    /// Consumes the next symbol otherwise
    fn expect_punc(
        &mut self,
        p: Punctuator,
        routine: Option<&'static str>,
    ) -> Result<(), ParseError> {
        self.expect(TokenKind::Punctuator(p), routine)
    }

    /// Reads a list of statements as a `Node::StatementList`.
    ///
    /// It will end at the end of file.
    fn read_statement_list(&mut self) -> ParseResult {
        self.read_statements(false).map(Node::StatementList)
    }

    /// Reads a code block as a `Node::Block`.
    ///
    /// Note: it will start after the `{` character and stop after reading `}`.
    fn read_block_statement(&mut self) -> ParseResult {
        self.read_statements(true).map(Node::Block)
    }

    /// Read a list of statements and stop after `}`
    ///
    /// Note: it will start after the `{` character and stop after reading `}`.
    fn read_block(&mut self) -> ParseResult {
        self.read_statements(true).map(Node::StatementList)
    }

    /// Reads a list of statements.
    ///
    /// If `break_when_closingbrase` is `true`, it will stop as soon as it finds a `}` character.
    fn read_statements(&mut self, break_when_closingbrase: bool) -> Result<Vec<Node>, ParseError> {
        let mut items = Vec::new();

        loop {
            if let Some(token) =
                self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::CloseBlock))
            {
                if break_when_closingbrase {
                    break;
                } else {
                    return Err(ParseError::Unexpected(token.clone(), None));
                }
            }

            if self.peek_skip_lineterminator().is_none() {
                if break_when_closingbrase {
                    return Err(ParseError::Expected(
                        vec![TokenKind::Punctuator(Punctuator::CloseBlock)],
                        self.get_next_token()?.clone(),
                        None,
                    ));
                } else {
                    break;
                }
            };

            let item = self.read_statement_list_item()?;
            items.push(item);

            // move the cursor forward for any consecutive semicolon.
            while self
                .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Semicolon))
                .is_some()
            {}
        }

        Ok(items)
    }

    /// Reads an individual statement list item.
    ///
    /// A statement list item can either be an statement or a declaration.
    ///
    /// More information:
    ///  - ECMAScript reference: <https://tc39.es/ecma262/#prod-StatementListItem>.
    ///  - MDN information page about statements and declarations:
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements>.
    fn read_statement_list_item(&mut self) -> ParseResult {
        if let Some(tok) = self.peek_skip_lineterminator() {
            match tok.kind {
                TokenKind::Keyword(Keyword::Function)
                | TokenKind::Keyword(Keyword::Const)
                | TokenKind::Keyword(Keyword::Let) => self.read_declaration(),
                _ => self.read_statement(),
            }
        } else {
            Err(ParseError::AbruptEnd)
        }
    }

    /// Parses a declaration.
    ///
    /// More information:: <https://tc39.es/ecma262/#prod-Declaration>
    fn read_declaration(&mut self) -> ParseResult {
        if let Some(tok) = self.next_skip_lineterminator() {
            match tok.kind {
                TokenKind::Keyword(Keyword::Function) => self.read_function_declaration(),
                TokenKind::Keyword(Keyword::Const) => self.read_binding_list(true),
                TokenKind::Keyword(Keyword::Let) => self.read_binding_list(false),
                _ => unreachable!(),
            }
        } else {
            Err(ParseError::AbruptEnd)
        }
    }

    /// Reads a binding list.
    ///
    /// It will return an error if a `const` declaration is being parsed and there is no
    /// initializer.
    ///
    /// More information: <https://tc39.es/ecma262/#prod-BindingList>.
    fn read_binding_list(&mut self, is_const: bool) -> ParseResult {
        // Create vectors to store the variable declarations
        // Const and Let signatures are slightly different, Const needs definitions, Lets don't
        let mut let_decls = Vec::new();
        let mut const_decls = Vec::new();

        loop {
            let token = self
                .next_skip_lineterminator()
                .ok_or(ParseError::AbruptEnd)?;
            let name = if let TokenKind::Identifier(ref name) = token.kind {
                name.clone()
            } else {
                return Err(ParseError::Expected(
                    vec![TokenKind::Identifier("identifier".to_owned())],
                    token.clone(),
                    if is_const {
                        Some("const declaration")
                    } else {
                        Some("let declaration")
                    },
                ));
            };

            if self
                .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Assign))
                .is_some()
            {
                let init = Some(self.read_initializer()?);
                if is_const {
                    const_decls.push((name, init.unwrap()));
                } else {
                    let_decls.push((name, init));
                };
            } else if is_const {
                return Err(ParseError::Expected(
                    vec![TokenKind::Punctuator(Punctuator::Assign)],
                    self.next_skip_lineterminator()
                        .ok_or(ParseError::AbruptEnd)?
                        .clone(),
                    Some("const declaration"),
                ));
            } else {
                let_decls.push((name, None));
            }

            if !self.lexical_declaration_continuation()? {
                break;
            }
        }

        if is_const {
            Ok(Node::ConstDecl(const_decls))
        } else {
            Ok(Node::LetDecl(let_decls))
        }
    }

    /// Parses a function declaration.
    ///
    /// More information:
    ///  - ECMAScript specification: <https://tc39.es/ecma262/#prod-FunctionDeclaration>.
    ///  - MDN documentation:
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function>
    fn read_function_declaration(&mut self) -> ParseResult {
        let token = self
            .next_skip_lineterminator()
            .ok_or(ParseError::AbruptEnd)?;
        let name = if let TokenKind::Identifier(name) = &token.kind {
            name.clone()
        } else {
            return Err(ParseError::Expected(
                vec![TokenKind::Identifier(String::from("function name"))],
                token.clone(),
                Some("function declaration"),
            ));
        };

        self.expect(
            TokenKind::Punctuator(Punctuator::OpenParen),
            Some("function declaration"),
        )?;

        let params = self.read_formal_parameters()?;

        self.expect(
            TokenKind::Punctuator(Punctuator::OpenBlock),
            Some("function declaration"),
        )?;

        let body = self.read_block()?;

        Ok(Node::FunctionDecl(Some(name), params, Box::new(body)))
    }

    /// <https://tc39.es/ecma262/#prod-Statement>
    fn read_statement(&mut self) -> ParseResult {
        let tok = self
            .next_skip_lineterminator()
            .ok_or(ParseError::AbruptEnd)?;

        let mut is_expression_statement = false;
        let stmt = match tok.kind {
            TokenKind::Keyword(Keyword::If) => self.read_if_statement(),
            TokenKind::Keyword(Keyword::Var) => self.read_variable_statement(),
            TokenKind::Keyword(Keyword::While) => self.read_while_statement(),
            TokenKind::Keyword(Keyword::For) => self.read_for_statement(),
            TokenKind::Keyword(Keyword::Return) => self.read_return_statement(),
            TokenKind::Keyword(Keyword::Break) => self.read_break_statement(),
            TokenKind::Keyword(Keyword::Continue) => self.read_continue_statement(),
            TokenKind::Keyword(Keyword::Try) => self.read_try_statement(),
            TokenKind::Keyword(Keyword::Throw) => self.read_throw_statement(),
            TokenKind::Keyword(Keyword::Switch) => self.read_switch_statement(),
            TokenKind::Punctuator(Punctuator::OpenBlock) => self.read_block_statement(),
            // TODO: https://tc39.es/ecma262/#prod-LabelledStatement
            // TokenKind::Punctuator(Punctuator::Semicolon) => {
            //     return Ok(Node::new(NodeBase::Nope, tok.pos))
            // }
            _ => {
                self.cursor.back();
                is_expression_statement = true;
                self.read_expression_statement()
            }
        };

        if self
            .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Semicolon))
            .is_none()
            && is_expression_statement
        {
            if let Some(tok) = self.cursor.peek(0) {
                if tok.kind != TokenKind::LineTerminator
                    && tok.kind != TokenKind::Punctuator(Punctuator::CloseBlock)
                {
                    return Err(ParseError::Expected(
                        vec![
                            TokenKind::Punctuator(Punctuator::Semicolon),
                            TokenKind::Punctuator(Punctuator::CloseBlock),
                            TokenKind::LineTerminator,
                        ],
                        tok.clone(),
                        None,
                    ));
                }
            }
        }

        stmt
    }

    /// <https://tc39.es/ecma262/#prod-SwitchStatement>
    fn read_switch_statement(&mut self) -> ParseResult {
        // TODO: Reimplement the switch statement in the new parser.
        unimplemented!("Switch statement parsing is not implemented");
    }

    /// <https://tc39.es/ecma262/#sec-expression-statement>
    fn read_expression_statement(&mut self) -> ParseResult {
        self.read_expression()
    }

    /// <https://tc39.es/ecma262/#sec-break-statement>
    fn read_break_statement(&mut self) -> ParseResult {
        let tok = self.get_next_token()?;
        match &tok.kind {
            TokenKind::LineTerminator
            | TokenKind::Punctuator(Punctuator::Semicolon)
            | TokenKind::Punctuator(Punctuator::CloseParen) => {
                self.cursor.back();
                Ok(Node::Break(None))
            }
            TokenKind::Identifier(name) => Ok(Node::Break(Some(name.clone()))),
            _ => Err(ParseError::Expected(
                vec![
                    TokenKind::Punctuator(Punctuator::Semicolon),
                    TokenKind::Punctuator(Punctuator::CloseParen),
                    TokenKind::LineTerminator,
                    TokenKind::Identifier("identifier".to_owned()),
                ],
                tok.clone(),
                Some("break statement"),
            )),
        }
    }

    /// <https://tc39.es/ecma262/#sec-continue-statement>
    fn read_continue_statement(&mut self) -> ParseResult {
        let tok = self.get_next_token()?;
        match &tok.kind {
            TokenKind::LineTerminator
            | TokenKind::Punctuator(Punctuator::Semicolon)
            | TokenKind::Punctuator(Punctuator::CloseBlock) => {
                self.cursor.back();
                Ok(Node::Continue(None))
            }
            TokenKind::Identifier(name) => Ok(Node::Continue(Some(name.clone()))),
            _ => Err(ParseError::Expected(
                vec![
                    TokenKind::Punctuator(Punctuator::Semicolon),
                    TokenKind::LineTerminator,
                    TokenKind::Punctuator(Punctuator::CloseBlock),
                ],
                tok.clone(),
                Some("continue statement"),
            )),
        }
    }

    /// <https://tc39.es/ecma262/#prod-ThrowStatement>
    fn read_throw_statement(&mut self) -> ParseResult {
        if let Some(tok) = self.cursor.peek(0) {
            match tok.kind {
                TokenKind::LineTerminator // no `LineTerminator` here
                | TokenKind::Punctuator(Punctuator::Semicolon)
                | TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    return Err(ParseError::Unexpected(tok.clone(), Some("throw statement")));
                }
                _ => {}
            }
        }

        let expr = self.read_expression()?;
        if let Some(tok) = self.cursor.peek(0) {
            if tok.kind == TokenKind::Punctuator(Punctuator::Semicolon) {
                let _ = self.cursor.next();
            }
        }

        Ok(Node::Throw(Box::new(expr)))
    }

    /// <https://tc39.es/ecma262/#prod-ReturnStatement>
    fn read_return_statement(&mut self) -> ParseResult {
        if let Some(tok) = self.cursor.peek(0) {
            match tok.kind {
                TokenKind::LineTerminator | TokenKind::Punctuator(Punctuator::Semicolon) => {
                    let _ = self.cursor.next();
                    return Ok(Node::Return(None));
                }
                TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    return Ok(Node::Return(None));
                }
                _ => {}
            }
        }

        let expr = self.read_expression()?;
        if let Some(tok) = self.cursor.peek(0) {
            if tok.kind == TokenKind::Punctuator(Punctuator::CloseBlock) {
                let _ = self.cursor.next();
            }
        }

        Ok(Node::Return(Some(Box::new(expr))))
    }

    /// <https://tc39.es/ecma262/#sec-if-statement>
    fn read_if_statement(&mut self) -> ParseResult {
        self.expect_punc(Punctuator::OpenParen, Some("if statement"))?;

        let cond = self.read_expression()?;

        self.expect_punc(Punctuator::CloseParen, Some("if statement"))?;

        let then_stm = self.read_statement()?;

        if let Some(else_tok) = self.cursor.next() {
            if else_tok.kind == TokenKind::Keyword(Keyword::Else) {
                let else_stm = self.read_statement()?;
                return Ok(Node::If(
                    Box::new(cond),
                    Box::new(then_stm),
                    Some(Box::new(else_stm)),
                ));
            } else {
                self.cursor.back();
            }
        }

        Ok(Node::If(Box::new(cond), Box::new(then_stm), None))
    }

    /// <https://tc39.es/ecma262/#sec-while-statement>
    fn read_while_statement(&mut self) -> ParseResult {
        self.expect_punc(Punctuator::OpenParen, Some("while statement"))?;

        let cond = self.read_expression()?;

        self.expect_punc(Punctuator::CloseParen, Some("while statement"))?;

        let body = self.read_statement()?;

        Ok(Node::WhileLoop(Box::new(cond), Box::new(body)))
    }

    /// <https://tc39.es/ecma262/#sec-try-statement>
    fn read_try_statement(&mut self) -> ParseResult {
        // TRY
        self.expect_punc(Punctuator::OpenBlock, Some("try statement"))?;
        let try_clause = self.read_block_statement()?;

        let next_token = self
            .peek_skip_lineterminator()
            .ok_or(ParseError::AbruptEnd)?;

        if next_token.kind != TokenKind::Keyword(Keyword::Catch)
            && next_token.kind != TokenKind::Keyword(Keyword::Finally)
        {
            return Err(ParseError::Expected(
                vec![
                    TokenKind::Keyword(Keyword::Catch),
                    TokenKind::Keyword(Keyword::Finally),
                ],
                next_token.clone(),
                Some("try statement"),
            ));
        }

        // CATCH
        let (catch, param) = if next_token.kind == TokenKind::Keyword(Keyword::Catch) {
            let _ = self.next_skip_lineterminator(); // Advance the cursor

            // Catch binding
            self.expect_punc(Punctuator::OpenParen, Some("catch in try statement"))?;
            // TODO: should accept BindingPattern
            let tok = self.get_next_token()?;
            let catch_param = if let TokenKind::Identifier(s) = &tok.kind {
                Node::Local(s.clone())
            } else {
                return Err(ParseError::Expected(
                    vec![TokenKind::Identifier("identifier".to_owned())],
                    tok.clone(),
                    Some("catch in try statement"),
                ));
            };
            self.expect_punc(Punctuator::CloseParen, Some("catch in try statement"))?;

            // Catch block
            self.expect_punc(Punctuator::OpenBlock, Some("catch in try statement"))?;
            (
                Some(Box::new(self.read_block()?)),
                Some(Box::new(catch_param)),
            )
        } else {
            (None, None)
        };

        // FINALLY
        let finally_block = if self
            .next_if_skip_lineterminator(TokenKind::Keyword(Keyword::Finally))
            .is_some()
        {
            self.expect_punc(Punctuator::OpenBlock, Some("finally in try statement"))?;
            Some(Box::new(self.read_block_statement()?))
        } else {
            None
        };

        Ok(Node::Try(Box::new(try_clause), catch, param, finally_block))
    }

    /// <https://tc39.es/ecma262/#sec-for-statement>
    fn read_for_statement(&mut self) -> ParseResult {
        self.expect_punc(Punctuator::OpenParen, Some("for statement"))?;

        let init = match self.cursor.peek(0).ok_or(ParseError::AbruptEnd)?.kind {
            TokenKind::Keyword(Keyword::Var) => {
                Some(Box::new(self.read_variable_declaration_list()?))
            }
            TokenKind::Keyword(Keyword::Let) | TokenKind::Keyword(Keyword::Const) => {
                Some(Box::new(self.read_declaration()?))
            }
            TokenKind::Punctuator(Punctuator::Semicolon) => None,
            _ => Some(Box::new(self.read_expression()?)),
        };

        self.expect_punc(Punctuator::Semicolon, Some("for statement"))?;

        let cond = if self
            .next_if(TokenKind::Punctuator(Punctuator::Semicolon))
            .is_some()
        {
            Some(Box::new(Node::Const(Const::Bool(true))))
        } else {
            let step = Some(Box::new(self.read_expression()?));
            self.expect_punc(Punctuator::Semicolon, Some("for statement"))?;
            step
        };

        let step = if self
            .next_if(TokenKind::Punctuator(Punctuator::CloseParen))
            .is_some()
        {
            None
        } else {
            let step = self.read_expression()?;
            self.expect(
                TokenKind::Punctuator(Punctuator::CloseParen),
                Some("for statement"),
            )?;
            Some(Box::new(step))
        };

        let body = Box::new(self.read_statement()?);

        let for_node = Node::ForLoop(init, cond, step, body);

        Ok(Node::Block(vec![for_node]))
    }
    /// <https://tc39.es/ecma262/#prod-VariableStatement>
    fn read_variable_statement(&mut self) -> ParseResult {
        self.read_variable_declaration_list()
    }

    /// <https://tc39.es/ecma262/#prod-VariableDeclarationList>
    fn read_variable_declaration_list(&mut self) -> ParseResult {
        let mut list = Vec::new();

        loop {
            list.push(self.read_variable_declaration()?);
            if !self.lexical_declaration_continuation()? {
                break;
            }
        }

        Ok(Node::VarDecl(list))
    }

    /// Checks if the lexical declaration continues with more bindings.
    ///
    /// If it does, it will advance the internal cursor to the next identifier token.
    /// A Lexical Declaration continues its binding list if we find a `,` character. A New line
    /// indicates the same as a `;`.
    ///
    /// More information: <https://tc39.es/ecma262/#prod-LexicalDeclaration>.
    fn lexical_declaration_continuation(&mut self) -> Result<bool, ParseError> {
        if let Some(tok) = self.cursor.peek(0) {
            match tok.kind {
                TokenKind::LineTerminator => {
                    let _ = self.cursor.next().ok_or(ParseError::AbruptEnd)?;
                    Ok(false)
                }
                TokenKind::Punctuator(Punctuator::Semicolon) => Ok(false),
                TokenKind::Punctuator(Punctuator::Comma) => {
                    let _ = self
                        .next_skip_lineterminator()
                        .ok_or(ParseError::AbruptEnd)?;
                    Ok(true)
                }
                _ => Err(ParseError::Expected(
                    vec![
                        TokenKind::Punctuator(Punctuator::Semicolon),
                        TokenKind::LineTerminator,
                    ],
                    self.cursor.next().ok_or(ParseError::AbruptEnd)?.clone(),
                    Some("lexical declaration"),
                )),
            }
        } else {
            Ok(false)
        }
    }

    /// <https://tc39.es/ecma262/#prod-VariableDeclaration>
    fn read_variable_declaration(&mut self) -> Result<(String, Option<Node>), ParseError> {
        let tok = self
            .next_skip_lineterminator()
            .ok_or(ParseError::AbruptEnd)?;
        let name = if let TokenKind::Identifier(name) = &tok.kind {
            name.clone()
        } else {
            return Err(ParseError::Expected(
                vec![TokenKind::Identifier("identifier".to_string())],
                tok.clone(),
                Some("variable declaration"),
            ));
        };

        if self
            .next_if(TokenKind::Punctuator(Punctuator::Assign))
            .is_some()
        {
            Ok((name, Some(self.read_initializer()?)))
        } else {
            Ok((name, None))
        }
    }

    /// Reads an initializer of variables.
    ///
    /// Note: it will expect the `=` token to have been already read.
    ///
    /// More information: <https://tc39.es/ecma262/#prod-Initializer>
    fn read_initializer(&mut self) -> ParseResult {
        self.read_assignment_expression()
    }

    // https://tc39.es/ecma262/#prod-Expression
    expression!(
        read_expression,
        read_assignment_expression,
        [Punctuator::Comma]
    );

    /// <https://tc39.es/ecma262/#prod-AssignmentExpression>
    fn read_assignment_expression(&mut self) -> ParseResult {
        // Arrow function
        let next_token = self.cursor.peek(0).ok_or(ParseError::AbruptEnd)?;
        match next_token.kind {
            // a=>{}
            TokenKind::Identifier(_) => {
                if let Some(tok) = self.cursor.peek(1) {
                    if tok.kind == TokenKind::Punctuator(Punctuator::Arrow) {
                        return self.read_arrow_function();
                    }
                }
            }
            // (a,b)=>{}
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                // TODO: breakpoints in the cursor.
                let save_pos = self.cursor.pos();
                let f = self.read_arrow_function(); // We try to read an arrow function.
                if f.is_err() {
                    self.cursor.seek(save_pos);
                } else {
                    return f;
                }
            }
            _ => {}
        }

        let mut lhs = self.read_conditional_expression()?;
        // let mut lhs = self.read_block()?;

        if let Ok(tok) = self.get_next_token() {
            match tok.kind {
                TokenKind::Punctuator(Punctuator::Assign) => {
                    lhs = Node::Assign(Box::new(lhs), Box::new(self.read_assignment_expression()?))
                }
                TokenKind::Punctuator(Punctuator::AssignAdd) => {
                    let expr = self.read_assignment_expression()?;
                    lhs = Node::BinOp(BinOp::Assign(AssignOp::Add), Box::new(lhs), Box::new(expr));
                }
                TokenKind::Punctuator(Punctuator::AssignSub) => {
                    let expr = self.read_assignment_expression()?;
                    lhs = Node::BinOp(BinOp::Assign(AssignOp::Sub), Box::new(lhs), Box::new(expr));
                }
                TokenKind::Punctuator(Punctuator::AssignMul) => {
                    let expr = self.read_assignment_expression()?;
                    lhs = Node::BinOp(BinOp::Assign(AssignOp::Mul), Box::new(lhs), Box::new(expr));
                }
                TokenKind::Punctuator(Punctuator::AssignDiv) => {
                    let expr = self.read_assignment_expression()?;
                    lhs = Node::BinOp(BinOp::Assign(AssignOp::Div), Box::new(lhs), Box::new(expr));
                }
                TokenKind::Punctuator(Punctuator::AssignAnd) => {
                    let expr = self.read_assignment_expression()?;
                    lhs = Node::BinOp(BinOp::Assign(AssignOp::And), Box::new(lhs), Box::new(expr));
                }
                TokenKind::Punctuator(Punctuator::AssignOr) => {
                    let expr = self.read_assignment_expression()?;
                    lhs = Node::BinOp(BinOp::Assign(AssignOp::Or), Box::new(lhs), Box::new(expr));
                }
                TokenKind::Punctuator(Punctuator::AssignXor) => {
                    let expr = self.read_assignment_expression()?;
                    lhs = Node::BinOp(BinOp::Assign(AssignOp::Xor), Box::new(lhs), Box::new(expr));
                }
                TokenKind::Punctuator(Punctuator::AssignRightSh) => {
                    let expr = self.read_assignment_expression()?;
                    lhs = Node::BinOp(BinOp::Assign(AssignOp::Shr), Box::new(lhs), Box::new(expr));
                }
                TokenKind::Punctuator(Punctuator::AssignLeftSh) => {
                    let expr = self.read_assignment_expression()?;
                    lhs = Node::BinOp(BinOp::Assign(AssignOp::Shl), Box::new(lhs), Box::new(expr));
                }
                TokenKind::Punctuator(Punctuator::AssignMod) => {
                    let expr = self.read_assignment_expression()?;
                    lhs = Node::BinOp(BinOp::Assign(AssignOp::Mod), Box::new(lhs), Box::new(expr));
                }
                TokenKind::Punctuator(Punctuator::AssignPow) => {
                    let expr = self.read_assignment_expression()?;
                    lhs = Node::BinOp(BinOp::Assign(AssignOp::Exp), Box::new(lhs), Box::new(expr));
                }
                _ => {
                    self.cursor.back();
                }
            }
        }

        Ok(lhs)
    }

    /// <https://tc39.es/ecma262/#prod-ConditionalExpression>
    fn read_conditional_expression(&mut self) -> ParseResult {
        let lhs = self.read_logical_or_expression()?;

        if let Ok(tok) = self.get_next_token() {
            if tok.kind == TokenKind::Punctuator(Punctuator::Question) {
                let then_ = self.read_assignment_expression()?;
                self.expect_punc(Punctuator::Colon, Some("conditional expression"))?;
                let else_ = self.read_assignment_expression()?;
                return Ok(Node::ConditionalOp(
                    Box::new(lhs),
                    Box::new(then_),
                    Box::new(else_),
                ));
            } else {
                self.cursor.back();
            }
        }

        Ok(lhs)
    }

    /// <https://tc39.es/ecma262/#sec-arrow-function-definitions>
    fn read_arrow_function(&mut self) -> ParseResult {
        let next_token = self.get_next_token()?;
        let params = match &next_token.kind {
            TokenKind::Punctuator(Punctuator::OpenParen) => self.read_formal_parameters()?,
            TokenKind::Identifier(param_name) => vec![FormalParameter {
                init: None,
                name: param_name.clone(),
                is_rest_param: false,
            }],
            _ => {
                return Err(ParseError::Expected(
                    vec![
                        TokenKind::Punctuator(Punctuator::OpenParen),
                        TokenKind::Identifier("identifier".to_owned()),
                    ],
                    next_token.clone(),
                    Some("arrow function"),
                ))
            }
        };

        self.expect_punc(Punctuator::Arrow, Some("arrow function"))?;

        let body = if self
            .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::OpenBlock))
            .is_some()
        {
            self.read_block()?
        } else {
            Node::Return(Some(Box::new(self.read_assignment_expression()?)))
        };

        Ok(Node::ArrowFunctionDecl(params, Box::new(body)))
    }

    /// Collect parameters from functions or arrow functions
    fn read_formal_parameters(&mut self) -> Result<FormalParameters, ParseError> {
        let mut params = Vec::new();

        if self
            .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::CloseParen))
            .is_some()
        {
            return Ok(params);
        }

        loop {
            let mut rest_param = false;

            params.push(
                if self
                    .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Spread))
                    .is_some()
                {
                    rest_param = true;
                    self.read_function_rest_parameter()?
                } else {
                    self.read_formal_parameter()?
                },
            );

            if self
                .next_if(TokenKind::Punctuator(Punctuator::CloseParen))
                .is_some()
            {
                break;
            }

            if rest_param {
                return Err(ParseError::Unexpected(
                    self.cursor
                        .peek_prev()
                        .expect("current token disappeared")
                        .clone(),
                    Some("rest parameter must be the last formal parameter"),
                ));
            }

            self.expect_punc(Punctuator::Comma, Some("parameter list"))?;
        }

        Ok(params)
    }

    /// <https://tc39.es/ecma262/#prod-FunctionRestParameter>
    fn read_function_rest_parameter(&mut self) -> Result<FormalParameter, ParseError> {
        let token = self.get_next_token()?;
        Ok(FormalParameter::new(
            if let TokenKind::Identifier(name) = &token.kind {
                name.clone()
            } else {
                return Err(ParseError::Expected(
                    vec![TokenKind::Identifier("identifier".to_owned())],
                    token.clone(),
                    Some("rest parameter"),
                ));
            },
            None,
            true,
        ))
    }

    /// <https://tc39.es/ecma262/#prod-FormalParameter>
    fn read_formal_parameter(&mut self) -> Result<FormalParameter, ParseError> {
        let token = self
            .next_skip_lineterminator()
            .ok_or(ParseError::AbruptEnd)?;
        let name = if let TokenKind::Identifier(name) = &token.kind {
            name
        } else {
            return Err(ParseError::Expected(
                vec![TokenKind::Identifier("identifier".to_owned())],
                token.clone(),
                Some("formal parameter"),
            ));
        };

        // TODO: Implement initializer.
        Ok(FormalParameter::new(name.clone(), None, false))
    }

    // <https://tc39.es/ecma262/#prod-LogicalORExpression>
    expression!(
        read_logical_or_expression,
        read_logical_and_expression,
        [Punctuator::BoolOr]
    );

    // <https://tc39.es/ecma262/#prod-LogicalANDExpression>
    expression!(
        read_logical_and_expression,
        read_bitwise_or_expression,
        [Punctuator::BoolAnd]
    );

    // https://tc39.es/ecma262/#prod-BitwiseORExpression
    expression!(
        read_bitwise_or_expression,
        read_bitwise_xor_expression,
        [Punctuator::Or]
    );

    // https://tc39.es/ecma262/#prod-BitwiseXORExpression
    expression!(
        read_bitwise_xor_expression,
        read_bitwise_and_expression,
        [Punctuator::Xor]
    );

    // <https://tc39.es/ecma262/#prod-BitwiseANDExpression>
    expression!(
        read_bitwise_and_expression,
        read_equality_expression,
        [Punctuator::And]
    );

    // <https://tc39.es/ecma262/#prod-EqualityExpression>
    expression!(
        read_equality_expression,
        read_relational_expression,
        [
            Punctuator::Eq,
            Punctuator::NotEq,
            Punctuator::StrictEq,
            Punctuator::StrictNotEq
        ]
    );

    // <https://tc39.es/ecma262/#prod-RelationalExpression>
    expression!(
        read_relational_expression,
        read_shift_expression,
        [
            Punctuator::LessThan,
            Punctuator::GreaterThan,
            Punctuator::LessThanOrEq,
            Punctuator::GreaterThanOrEq
        ]
    );

    // <https://tc39.es/ecma262/#prod-ShiftExpression>
    expression!(
        read_shift_expression,
        read_additive_expression,
        [
            Punctuator::LeftSh,
            Punctuator::RightSh,
            Punctuator::URightSh
        ]
    );

    // <https://tc39.es/ecma262/#prod-AdditiveExpression>
    expression!(
        read_additive_expression,
        read_multiplicate_expression,
        [Punctuator::Add, Punctuator::Sub]
    );

    // <https://tc39.es/ecma262/#prod-MultiplicativeExpression>
    expression!(
        read_multiplicate_expression,
        read_exponentiation_expression,
        [Punctuator::Mul, Punctuator::Div, Punctuator::Mod]
    );

    /// <https://tc39.es/ecma262/#prod-MultiplicativeExpression>
    fn read_exponentiation_expression(&mut self) -> ParseResult {
        if self.is_unary_expression() {
            return self.read_unary_expression();
        }

        let lhs = self.read_update_expression()?;
        if let Ok(tok) = self.get_next_token() {
            if let TokenKind::Punctuator(Punctuator::Exp) = tok.kind {
                return Ok(Node::BinOp(
                    BinOp::Num(NumOp::Exp),
                    Box::new(lhs),
                    Box::new(self.read_exponentiation_expression()?),
                ));
            } else {
                self.cursor.back();
            }
        }
        Ok(lhs)
    }

    // Checks by looking at the next token to see whether its a Unary operator or not.
    fn is_unary_expression(&mut self) -> bool {
        if let Some(tok) = self.cursor.peek(0) {
            match tok.kind {
                TokenKind::Keyword(Keyword::Delete)
                | TokenKind::Keyword(Keyword::Void)
                | TokenKind::Keyword(Keyword::TypeOf)
                | TokenKind::Punctuator(Punctuator::Add)
                | TokenKind::Punctuator(Punctuator::Sub)
                | TokenKind::Punctuator(Punctuator::Not)
                | TokenKind::Punctuator(Punctuator::Neg) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// <https://tc39.es/ecma262/#prod-UnaryExpression>
    fn read_unary_expression(&mut self) -> ParseResult {
        let tok = self.get_next_token()?;
        match tok.kind {
            TokenKind::Keyword(Keyword::Delete) => Ok(Node::UnaryOp(
                UnaryOp::Delete,
                Box::new(self.read_unary_expression()?),
            )),
            TokenKind::Keyword(Keyword::Void) => Ok(Node::UnaryOp(
                UnaryOp::Void,
                Box::new(self.read_unary_expression()?),
            )),
            TokenKind::Keyword(Keyword::TypeOf) => Ok(Node::UnaryOp(
                UnaryOp::TypeOf,
                Box::new(self.read_unary_expression()?),
            )),
            TokenKind::Punctuator(Punctuator::Add) => Ok(Node::UnaryOp(
                UnaryOp::Plus,
                Box::new(self.read_unary_expression()?),
            )),
            TokenKind::Punctuator(Punctuator::Sub) => Ok(Node::UnaryOp(
                UnaryOp::Minus,
                Box::new(self.read_unary_expression()?),
            )),
            TokenKind::Punctuator(Punctuator::Neg) => Ok(Node::UnaryOp(
                UnaryOp::Tilde,
                Box::new(self.read_unary_expression()?),
            )),
            TokenKind::Punctuator(Punctuator::Not) => Ok(Node::UnaryOp(
                UnaryOp::Not,
                Box::new(self.read_unary_expression()?),
            )),
            _ => {
                self.cursor.back();
                self.read_update_expression()
            }
        }
    }

    /// <https://tc39.es/ecma262/#prod-UpdateExpression>
    fn read_update_expression(&mut self) -> ParseResult {
        let tok = self
            .peek_skip_lineterminator()
            .ok_or(ParseError::AbruptEnd)?;
        match tok.kind {
            TokenKind::Punctuator(Punctuator::Inc) => {
                self.next_skip_lineterminator().unwrap();
                return Ok(Node::UnaryOp(
                    UnaryOp::IncrementPre,
                    Box::new(self.read_left_hand_side_expression()?),
                ));
            }
            TokenKind::Punctuator(Punctuator::Dec) => {
                self.next_skip_lineterminator().unwrap();
                return Ok(Node::UnaryOp(
                    UnaryOp::DecrementPre,
                    Box::new(self.read_left_hand_side_expression()?),
                ));
            }
            _ => {}
        }

        let lhs = self.read_left_hand_side_expression()?;
        if let Some(tok) = self.cursor.peek(0) {
            match tok.kind {
                TokenKind::Punctuator(Punctuator::Inc) => {
                    self.get_next_token().unwrap();
                    return Ok(Node::UnaryOp(UnaryOp::IncrementPost, Box::new(lhs)));
                }
                TokenKind::Punctuator(Punctuator::Dec) => {
                    self.get_next_token().unwrap();
                    return Ok(Node::UnaryOp(UnaryOp::DecrementPost, Box::new(lhs)));
                }
                _ => {}
            }
        }

        Ok(lhs)
    }

    /// <https://tc39.es/ecma262/#prod-LeftHandSideExpression>
    fn read_left_hand_side_expression(&mut self) -> ParseResult {
        // TODO: Implement NewExpression: new MemberExpression
        let lhs = self.read_member_expression()?;
        match self.peek_skip_lineterminator() {
            Some(ref tok) if tok.kind == TokenKind::Punctuator(Punctuator::OpenParen) => {
                self.read_call_expression(lhs)
            }
            _ => self.read_new_expression(lhs),
        }
    }

    /// <https://tc39.es/ecma262/#prod-NewExpression>
    fn read_new_expression(&mut self, first_member_expr: Node) -> ParseResult {
        Ok(first_member_expr)
    }

    /// <https://tc39.es/ecma262/#prod-MemberExpression>
    fn read_member_expression(&mut self) -> ParseResult {
        let mut lhs = if self
            .peek_skip_lineterminator()
            .ok_or(ParseError::AbruptEnd)?
            .kind
            == TokenKind::Keyword(Keyword::New)
        {
            let _ = self
                .next_skip_lineterminator()
                .expect("keyword disappeared");
            let lhs = self.read_member_expression()?;
            self.expect_punc(Punctuator::OpenParen, Some("member expression"))?;
            let args = self.read_arguments()?;
            let call_node = Node::Call(Box::new(lhs), args);

            Node::New(Box::new(call_node))
        } else {
            self.read_primary_expression()?
        };
        while let Some(tok) = self.peek_skip_lineterminator() {
            match &tok.kind {
                TokenKind::Punctuator(Punctuator::Dot) => {
                    let _ = self
                        .next_skip_lineterminator()
                        .ok_or(ParseError::AbruptEnd)?; // We move the cursor forward.
                    match &self
                        .next_skip_lineterminator()
                        .ok_or(ParseError::AbruptEnd)?
                        .kind
                    {
                        TokenKind::Identifier(name) => {
                            lhs = Node::GetConstField(Box::new(lhs), name.clone())
                        }
                        TokenKind::Keyword(kw) => {
                            lhs = Node::GetConstField(Box::new(lhs), kw.to_string())
                        }
                        _ => {
                            return Err(ParseError::Expected(
                                vec![TokenKind::Identifier("identifier".to_owned())],
                                tok.clone(),
                                Some("member expression"),
                            ));
                        }
                    }
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    let _ = self
                        .next_skip_lineterminator()
                        .ok_or(ParseError::AbruptEnd)?; // We move the cursor forward.
                    let idx = self.read_expression()?;
                    self.expect_punc(Punctuator::CloseBracket, Some("member expression"))?;
                    lhs = Node::GetField(Box::new(lhs), Box::new(idx));
                }
                _ => break,
            }
        }

        Ok(lhs)
    }

    /// <https://tc39.es/ecma262/#prod-CallExpression>
    fn read_call_expression(&mut self, first_member_expr: Node) -> ParseResult {
        let mut lhs = first_member_expr;
        if self
            .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::OpenParen))
            .is_some()
        {
            let args = self.read_arguments()?;
            lhs = Node::Call(Box::new(lhs), args);
        } else {
            let next_token = self
                .next_skip_lineterminator()
                .ok_or(ParseError::AbruptEnd)?;
            return Err(ParseError::Expected(
                vec![TokenKind::Punctuator(Punctuator::OpenParen)],
                next_token.clone(),
                Some("call expression"),
            ));
        }

        while let Some(tok) = self.peek_skip_lineterminator() {
            match tok.kind {
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    let _ = self
                        .next_skip_lineterminator()
                        .ok_or(ParseError::AbruptEnd)?; // We move the cursor.
                    let args = self.read_arguments()?;
                    lhs = Node::Call(Box::new(lhs), args);
                }
                TokenKind::Punctuator(Punctuator::Dot) => {
                    let _ = self
                        .next_skip_lineterminator()
                        .ok_or(ParseError::AbruptEnd)?; // We move the cursor.
                    match &self
                        .next_skip_lineterminator()
                        .ok_or(ParseError::AbruptEnd)?
                        .kind
                    {
                        TokenKind::Identifier(name) => {
                            lhs = Node::GetConstField(Box::new(lhs), name.clone());
                        }
                        TokenKind::Keyword(kw) => {
                            lhs = Node::GetConstField(Box::new(lhs), kw.to_string());
                        }
                        _ => {
                            return Err(ParseError::Expected(
                                vec![TokenKind::Identifier("identifier".to_owned())],
                                tok.clone(),
                                Some("call expression"),
                            ));
                        }
                    }
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    let _ = self
                        .next_skip_lineterminator()
                        .ok_or(ParseError::AbruptEnd)?; // We move the cursor.
                    let idx = self.read_expression()?;
                    self.expect_punc(Punctuator::CloseBracket, Some("call expression"))?;
                    lhs = Node::GetField(Box::new(lhs), Box::new(idx));
                }
                _ => break,
            }
        }

        Ok(lhs)
    }

    /// <https://tc39.es/ecma262/#prod-Arguments>
    fn read_arguments(&mut self) -> Result<Vec<Node>, ParseError> {
        let mut args = Vec::new();
        loop {
            let next_token = self
                .next_skip_lineterminator()
                .ok_or(ParseError::AbruptEnd)?;
            match next_token.kind {
                TokenKind::Punctuator(Punctuator::CloseParen) => break,
                TokenKind::Punctuator(Punctuator::Comma) => {
                    if args.is_empty() {
                        return Err(ParseError::Unexpected(next_token.clone(), None));
                    }

                    if self
                        .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::CloseParen))
                        .is_some()
                    {
                        break;
                    }
                }
                _ => {
                    if !args.is_empty() {
                        return Err(ParseError::Expected(
                            vec![
                                TokenKind::Punctuator(Punctuator::Comma),
                                TokenKind::Punctuator(Punctuator::CloseParen),
                            ],
                            next_token.clone(),
                            Some("argument list"),
                        ));
                    } else {
                        self.cursor.back();
                    }
                }
            }

            if self
                .next_if(TokenKind::Punctuator(Punctuator::Spread))
                .is_some()
            {
                args.push(Node::Spread(Box::new(self.read_assignment_expression()?)));
            } else {
                args.push(self.read_assignment_expression()?);
            }
        }

        Ok(args)
    }

    /// <https://tc39.es/ecma262/#prod-PrimaryExpression>
    fn read_primary_expression(&mut self) -> ParseResult {
        let tok = self
            .next_skip_lineterminator()
            .ok_or(ParseError::AbruptEnd)?;

        match &tok.kind {
            TokenKind::Keyword(Keyword::This) => Ok(Node::This),
            // TokenKind::Keyword(Keyword::Arguments) => Ok(Node::new(NodeBase::Arguments, tok.pos)),
            TokenKind::Keyword(Keyword::Function) => self.read_function_expression(),
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                let expr = self.read_expression();
                self.expect_punc(Punctuator::CloseParen, Some("primary expression"))?;
                expr
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => self.read_array_literal(),
            TokenKind::Punctuator(Punctuator::OpenBlock) => self.read_object_literal(),
            TokenKind::BooleanLiteral(boolean) => Ok(Node::Const(Const::Bool(*boolean))),
            // TODO: ADD TokenKind::UndefinedLiteral
            TokenKind::Identifier(ref i) if i == "undefined" => Ok(Node::Const(Const::Undefined)),
            TokenKind::NullLiteral => Ok(Node::Const(Const::Null)),
            TokenKind::Identifier(ident) => Ok(Node::Local(ident.clone())),
            TokenKind::StringLiteral(s) => Ok(Node::Const(Const::String(s.clone()))),
            TokenKind::NumericLiteral(num) => Ok(Node::Const(Const::Num(*num))),
            TokenKind::RegularExpressionLiteral(body, flags) => {
                Ok(Node::New(Box::new(Node::Call(
                    Box::new(Node::Local("RegExp".to_string())),
                    vec![
                        Node::Const(Const::String(body.clone())),
                        Node::Const(Const::String(flags.clone())),
                    ],
                ))))
            }
            _ => Err(ParseError::Unexpected(
                tok.clone(),
                Some("primary expression"),
            )),
        }
    }

    /// <https://tc39.es/ecma262/#prod-FunctionExpression>
    fn read_function_expression(&mut self) -> ParseResult {
        let name = if let TokenKind::Identifier(name) =
            &self.cursor.peek(0).ok_or(ParseError::AbruptEnd)?.kind
        {
            Some(name.clone())
        } else {
            None
        };
        if name.is_some() {
            // We move the cursor forward.
            let _ = self.get_next_token()?;
        }

        self.expect_punc(Punctuator::OpenParen, Some("function expression"))?;

        let params = self.read_formal_parameters()?;

        self.expect_punc(Punctuator::OpenBlock, Some("function expression"))?;

        let body = self.read_block()?;

        Ok(Node::FunctionDecl(name, params, Box::new(body)))
    }

    /// <https://tc39.es/ecma262/#prod-ArrayLiteral>
    fn read_array_literal(&mut self) -> ParseResult {
        let mut elements = Vec::new();

        loop {
            // TODO: Support all features.
            while self
                .next_if(TokenKind::Punctuator(Punctuator::Comma))
                .is_some()
            {
                elements.push(Node::Const(Const::Undefined));
            }

            if self
                .next_if(TokenKind::Punctuator(Punctuator::CloseBracket))
                .is_some()
            {
                break;
            }

            let _ = self.cursor.peek(0).ok_or(ParseError::AbruptEnd)?; // Check that there are more tokens to read.

            if self
                .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Spread))
                .is_some()
            {
                let node = self.read_assignment_expression()?;
                elements.push(Node::Spread(Box::new(node)));
            } else {
                elements.push(self.read_assignment_expression()?);
            }
            self.next_if(TokenKind::Punctuator(Punctuator::Comma));
        }

        Ok(Node::ArrayDecl(elements))
    }

    /// <https://tc39.es/ecma262/#prod-ObjectLiteral>
    fn read_object_literal(&mut self) -> ParseResult {
        let mut elements = Vec::new();

        loop {
            if self
                .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::CloseBlock))
                .is_some()
            {
                break;
            }

            elements.push(self.read_property_definition()?);

            if self
                .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::CloseBlock))
                .is_some()
            {
                break;
            }

            if self
                .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Comma))
                .is_none()
            {
                let next_token = self
                    .next_skip_lineterminator()
                    .ok_or(ParseError::AbruptEnd)?;
                return Err(ParseError::Expected(
                    vec![
                        TokenKind::Punctuator(Punctuator::Comma),
                        TokenKind::Punctuator(Punctuator::CloseBlock),
                    ],
                    next_token.clone(),
                    Some("object literal"),
                ));
            }
        }

        Ok(Node::Object(elements))
    }

    /// <https://tc39.es/ecma262/#prod-PropertyDefinition>
    fn read_property_definition(&mut self) -> Result<PropertyDefinition, ParseError> {
        fn to_string(kind: &TokenKind) -> String {
            match kind {
                TokenKind::Identifier(name) => name.clone(),
                TokenKind::NumericLiteral(n) => format!("{}", n),
                TokenKind::StringLiteral(s) => s.clone(),
                _ => unimplemented!("{:?}", kind),
            }
        }

        if self
            .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Spread))
            .is_some()
        {
            let node = self.read_assignment_expression()?;
            return Ok(PropertyDefinition::SpreadObject(node));
        }

        let prop_name = self
            .next_skip_lineterminator()
            .map(|tok| to_string(&tok.kind))
            .ok_or(ParseError::AbruptEnd)?;

        if self
            .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Colon))
            .is_some()
        {
            let val = self.read_assignment_expression()?;
            return Ok(PropertyDefinition::Property(prop_name, val));
        }

        // TODO: Split into separate function: read_property_method_definition
        if self
            .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::OpenParen))
            .is_some()
        {
            let params = self.read_formal_parameters()?;

            self.expect(
                TokenKind::Punctuator(Punctuator::OpenBlock),
                Some("property method definition"),
            )?;

            let body = self.read_block()?;

            return Ok(PropertyDefinition::MethodDefinition(
                MethodDefinitionKind::Ordinary,
                prop_name,
                Node::FunctionDecl(None, params, Box::new(body)),
            ));
        }

        // TODO need to revisit this
        // if let TokenKind::Identifier(name) = tok.kind {
        //     if name == "get" || name == "set" {
        //         let may_identifier = self.peek_skip_lineterminator();
        //         if may_identifier.is_some()
        //             && matches!(may_identifier.unwrap().kind, TokenKind::Identifier(_))
        //         {
        //             let f = self.read_function_expression()?;
        //             let func_name = if let NodeBase::FunctionExpr(ref name, _, _) = f.base {
        //                 name.clone().unwrap()
        //             } else {
        //                 panic!()
        //             };
        //             return Ok(PropertyDefinition::MethodDefinition(
        //                 if name == "get" {
        //                     MethodDefinitionKind::Get
        //                 } else {
        //                     MethodDefinitionKind::Set
        //                 },
        //                 func_name,
        //                 f,
        //             ));
        //         }
        //     }

        //     return Ok(PropertyDefinition::IdentifierReference(name));
        // }

        let pos = self
            .cursor
            .peek(0)
            .map(|tok| tok.pos)
            .ok_or(ParseError::AbruptEnd)?;
        Err(ParseError::General(
            "expected property definition",
            Some(pos),
        ))
    }
}
