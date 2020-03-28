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
use std::{collections::btree_map::BTreeMap, fmt};

/// `ParseError` is an enum which represents errors encounted during parsing an expression
#[derive(Debug, Clone)]
pub enum ParseError {
    /// When it expected a certain kind of token, but got another as part of something
    Expected(Vec<TokenKind>, Token, &'static str),
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
    /// End of the stream has been reached
    NormalEOF,
    /// Catch all General Error
    General(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Expected(expected, actual, routine) => write!(
                f,
                "Expected {}, got '{}' in routine '{}' at line {}, col {}",
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
                routine,
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
            ParseError::General(msg) => write!(f, "{}", msg),
            ParseError::NormalEOF => write!(f, "EOF"),
            ParseError::RangeError => write!(f, "RangeError!"),
        }
    }
}

pub type ParseResult = Result<Node, ParseError>;

#[derive(Debug)]
pub struct Parser {
    /// The tokens being input
    tokens: Vec<Token>,
    /// The current position within the tokens
    pos: usize,
}

macro_rules! expression { ( $name:ident, $lower:ident, [ $( $op:path ),* ] ) => {
    fn $name (&mut self) -> Result<Node, ParseError> {
        let mut lhs = self. $lower ()?;
        while let Ok(tok) = self.peek_skip_lineterminator() {
            match tok.kind {
                // Parse assign expression
                TokenKind::Punctuator(ref op) if op == &Punctuator::Assign => {
                    self.next_skip_lineterminator().unwrap();
                    lhs = Node::Assign(
                        Box::new(lhs),
                        Box::new(self. $lower ()?)
                    )
                }
                TokenKind::Punctuator(ref op) if $( op == &$op )||* => {
                    self.next_skip_lineterminator().unwrap();
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

impl Parser {
    /// Create a new parser, using `tokens` as input
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse all expressions in the token array
    pub fn parse_all(&mut self) -> ParseResult {
        self.read_statement_list()
    }

    // I hope to deprecate this
    fn get_token(&self, pos: usize) -> Result<Token, ParseError> {
        if pos < self.tokens.len() {
            Ok(self.tokens.get(pos).expect("failed getting token").clone())
        } else {
            Err(ParseError::AbruptEnd)
        }
    }

    /// consume the next token and increment position
    fn get_next_token(&mut self) -> Result<Token, ParseError> {
        let token = self.get_token(self.pos);
        self.pos += 1;
        token
    }

    /// Returns the current token  the cursor is sitting on
    fn get_current_token(&self) -> Result<Token, ParseError> {
        self.get_token(self.pos)
    }

    /// Move the cursor back 1
    fn step_back(&mut self) {
        self.pos -= 1;
    }

    /// peeks at the next token
    fn peek(&self, num: usize) -> Result<Token, ParseError> {
        self.get_token(self.pos + num)
    }

    /// get_current_pos
    fn get_prev_pos(&self) -> usize {
        self.pos - 1
    }

    /// get_current_pos
    fn get_current_pos(&self) -> usize {
        self.pos
    }

    /// set the current position
    fn set_current_pos(&mut self, pos: usize) -> Result<(), ParseError> {
        if pos < self.tokens.len() {
            self.pos = pos;
            Ok(())
        } else {
            Err(ParseError::RangeError)
        }
    }

    /// Peek the next token.
    /// Skipping line terminators.
    pub fn peek_skip_lineterminator(&mut self) -> Result<Token, ParseError> {
        let len = self.tokens.len();
        for i in self.pos..len {
            let tok = self.tokens[i].clone();
            if tok.kind != TokenKind::LineTerminator {
                return Ok(tok);
            }
        }

        Err(ParseError::NormalEOF)
    }

    /// Consume the next token.\
    /// Skipping line terminators.
    pub fn next_skip_lineterminator(&mut self) -> Result<Token, ParseError> {
        loop {
            let tok = self.get_next_token()?;
            if tok.kind != TokenKind::LineTerminator {
                return Ok(tok);
            }
        }
    }

    /// Peek the next token, if it's of `kind` type.
    ///
    /// When the next token is a `kind` token, get the token, otherwise return `None`. This
    /// function skips line terminators.
    fn next_if_skip_lineterminator(&mut self, kind: TokenKind) -> Result<bool, ParseError> {
        match self.peek_skip_lineterminator() {
            Ok(tok) => {
                if tok.kind == kind {
                    self.next_skip_lineterminator()?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Gets the next token, if the current token is of `kind` type.
    ///
    /// When the current token is a `kind` token, get the next token, otherwise return `None`.
    fn next_if(&mut self, kind: TokenKind) -> Option<Token> {
        match self.get_current_token() {
            Ok(tok) => {
                if tok.kind == kind {
                    Some(self.get_next_token().unwrap())
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<FormalParameter>, ParseError> {
        self.expect_punc(Punctuator::OpenParen, "function parameters ( expected")?;
        let mut args = Vec::new();
        let mut tk = self.get_token(self.pos)?;
        while tk.kind != TokenKind::Punctuator(Punctuator::CloseParen) {
            match tk.kind {
                TokenKind::Identifier(ref id) => args.push(FormalParameter::new(
                    id.clone(),
                    Some(Box::new(Node::Local(id.clone()))),
                    false,
                )),
                TokenKind::Punctuator(Punctuator::Spread) => {
                    args.push(FormalParameter::new(
                        String::new(),
                        Some(Box::new(self.parse()?)),
                        true,
                    ));
                    self.pos -= 1; // roll back so we're sitting on the closeParen ')'
                }
                _ => {
                    return Err(ParseError::Expected(
                        vec![TokenKind::Identifier("identifier".to_string())],
                        tk,
                        "function arguments",
                    ))
                }
            }
            self.pos += 1;
            if self.get_token(self.pos)?.kind == TokenKind::Punctuator(Punctuator::Comma) {
                self.pos += 1;
            }
            tk = self.get_token(self.pos)?;
        }

        self.expect_punc(Punctuator::CloseParen, "function parameters ) expected")?;
        Ok(args)
    }

    fn parse_struct(&mut self, keyword: Keyword) -> ParseResult {
        match keyword {
            Keyword::Throw => {
                let thrown = self.parse()?;
                Ok(Node::Throw(Box::new(thrown)))
            }
            // vars, lets and consts are similar in parsing structure, we can group them together
            Keyword::Var | Keyword::Let => {
                let mut vars = Vec::new();
                loop {
                    let name = match self.get_token(self.pos) {
                        Ok(Token {
                            kind: TokenKind::Identifier(ref name),
                            ..
                        }) => name.clone(),
                        Ok(tok) => {
                            return Err(ParseError::Expected(
                                vec![TokenKind::Identifier("identifier".to_string())],
                                tok,
                                "var/let declaration",
                            ))
                        }
                        Err(ParseError::AbruptEnd) => break,
                        Err(e) => return Err(e),
                    };
                    self.pos += 1;
                    match self.get_token(self.pos) {
                        Ok(Token {
                            kind: TokenKind::Punctuator(Punctuator::Assign),
                            ..
                        }) => {
                            self.pos += 1;
                            let val = self.parse()?;
                            vars.push((name, Some(val)));
                            match self.get_token(self.pos) {
                                Ok(Token {
                                    kind: TokenKind::Punctuator(Punctuator::Comma),
                                    ..
                                }) => self.pos += 1,
                                _ => break,
                            }
                        }
                        Ok(Token {
                            kind: TokenKind::Punctuator(Punctuator::Comma),
                            ..
                        }) => {
                            self.pos += 1;
                            vars.push((name, None));
                        }
                        _ => {
                            vars.push((name, None));
                            break;
                        }
                    }
                }

                match keyword {
                    Keyword::Let => Ok(Node::LetDecl(vars)),
                    _ => Ok(Node::VarDecl(vars)),
                }
            }
            Keyword::Const => {
                let mut vars = Vec::new();
                loop {
                    let name = match self.get_token(self.pos) {
                        Ok(Token {
                            kind: TokenKind::Identifier(ref name),
                            ..
                        }) => name.clone(),
                        Ok(tok) => {
                            return Err(ParseError::Expected(
                                vec![TokenKind::Identifier("identifier".to_string())],
                                tok,
                                "const declaration",
                            ))
                        }
                        Err(ParseError::AbruptEnd) => break,
                        Err(e) => return Err(e),
                    };
                    self.pos += 1;
                    match self.get_token(self.pos) {
                        Ok(Token {
                            kind: TokenKind::Punctuator(Punctuator::Assign),
                            ..
                        }) => {
                            self.pos += 1;
                            let val = self.parse()?;
                            vars.push((name, val));
                            match self.get_token(self.pos) {
                                Ok(Token {
                                    kind: TokenKind::Punctuator(Punctuator::Comma),
                                    ..
                                }) => self.pos += 1,
                                _ => break,
                            }
                        }
                        Ok(tok) => {
                            return Err(ParseError::Expected(
                                vec![TokenKind::Punctuator(Punctuator::Assign)],
                                tok,
                                "const declration",
                            ))
                        }
                        _ => break,
                    }
                }

                Ok(Node::ConstDecl(vars))
            }
            Keyword::Return => match self.get_token(self.pos)?.kind {
                TokenKind::Punctuator(Punctuator::Semicolon)
                | TokenKind::Punctuator(Punctuator::CloseBlock) => Ok(Node::Return(None)),
                _ => Ok(Node::Return(Some(Box::new(self.parse()?)))),
            },
            Keyword::New => {
                let start_pos = self.pos;
                let call = self.parse()?;
                if let Node::Call(ref func, ref args) = call {
                    Ok(Node::Construct(func.clone(), args.clone()))
                } else {
                    let token = self.get_token(start_pos)?;
                    Err(ParseError::ExpectedExpr("constructor", call, token.pos))
                }
            }
            Keyword::TypeOf => Ok(Node::TypeOf(Box::new(self.parse()?))),
            Keyword::If => {
                self.expect_punc(Punctuator::OpenParen, "if block")?;
                let cond = self.parse()?;
                self.expect_punc(Punctuator::CloseParen, "if block")?;
                let expr = self.parse()?;
                let next = self.get_token(self.pos);
                Ok(Node::If(
                    Box::new(cond),
                    Box::new(expr),
                    if next.is_ok()
                        && next.expect("Could not get next value").kind
                            == TokenKind::Keyword(Keyword::Else)
                    {
                        self.pos += 1;
                        Some(Box::new(self.parse()?))
                    } else {
                        None
                    },
                ))
            }
            Keyword::While => {
                self.expect_punc(Punctuator::OpenParen, "while condition")?;
                let cond = self.parse()?;
                self.expect_punc(Punctuator::CloseParen, "while condition")?;
                let expr = self.parse()?;
                Ok(Node::WhileLoop(Box::new(cond), Box::new(expr)))
            }
            Keyword::Switch => {
                self.expect_punc(Punctuator::OpenParen, "switch value")?;
                let value = self.parse();
                self.expect_punc(Punctuator::CloseParen, "switch value")?;
                self.expect_punc(Punctuator::OpenBlock, "switch block")?;
                let mut cases = Vec::new();
                let mut default = None;
                while self.pos.wrapping_add(1) < self.tokens.len() {
                    let tok = self.get_token(self.pos)?;
                    self.pos += 1;
                    match tok.kind {
                        TokenKind::Keyword(Keyword::Case) => {
                            let cond = self.parse();
                            let mut block = Vec::new();
                            self.expect_punc(Punctuator::Colon, "switch case")?;
                            loop {
                                match self.get_token(self.pos)?.kind {
                                    TokenKind::Keyword(Keyword::Case)
                                    | TokenKind::Keyword(Keyword::Default)
                                    | TokenKind::Punctuator(Punctuator::CloseBlock) => break,
                                    _ => block.push(self.parse()?),
                                }
                            }
                            cases.push((cond.expect("No condition supplied"), block));
                        }
                        TokenKind::Keyword(Keyword::Default) => {
                            let mut block = Vec::new();
                            self.expect_punc(Punctuator::Colon, "default switch case")?;
                            loop {
                                match self.get_token(self.pos)?.kind {
                                    TokenKind::Keyword(Keyword::Case)
                                    | TokenKind::Keyword(Keyword::Default)
                                    | TokenKind::Punctuator(Punctuator::CloseBlock) => break,
                                    _ => block.push(self.parse()?),
                                }
                            }
                            default = Some(Node::Block(block));
                        }
                        TokenKind::Punctuator(Punctuator::CloseBlock) => break,
                        _ => {
                            return Err(ParseError::Expected(
                                vec![
                                    TokenKind::Keyword(Keyword::Case),
                                    TokenKind::Keyword(Keyword::Default),
                                    TokenKind::Punctuator(Punctuator::CloseBlock),
                                ],
                                tok,
                                "switch block",
                            ))
                        }
                    }
                }
                self.expect_punc(Punctuator::CloseBlock, "switch block")?;
                Ok(Node::Switch(
                    Box::new(value.expect("Could not get value")),
                    cases,
                    match default {
                        Some(v) => Some(Box::new(v)),
                        None => None,
                    },
                ))
            }
            Keyword::Function => {
                // function [identifier] () { etc }
                let tk = self.get_token(self.pos)?;
                let name = match tk.kind {
                    TokenKind::Identifier(ref name) => {
                        self.pos += 1;
                        Some(name.clone())
                    }
                    TokenKind::Punctuator(Punctuator::OpenParen) => None,
                    _ => {
                        return Err(ParseError::Expected(
                            vec![TokenKind::Identifier("identifier".to_string())],
                            tk,
                            "function name",
                        ))
                    }
                };
                // Now we have the function identifier we should have an open paren for arguments ( )
                let args = self.parse_function_parameters()?;
                let block = self.parse()?;
                Ok(Node::FunctionDecl(name, args, Box::new(block)))
            }
            _ => {
                let token = self.get_token(self.pos - 1)?; // Gets the offending token
                Err(ParseError::UnexpectedKeyword(keyword, token.pos))
            }
        }
    }

    /// Parse a single expression
    pub fn parse(&mut self) -> ParseResult {
        if self.pos > self.tokens.len() {
            return Err(ParseError::AbruptEnd);
        }
        let token = self.get_token(self.pos)?;
        self.pos += 1;
        let node: Node = match token.kind {
            TokenKind::NumericLiteral(num) => Node::Const(Const::Num(num)),
            TokenKind::NullLiteral => Node::Const(Const::Null),
            TokenKind::StringLiteral(text) => Node::Const(Const::String(text)),
            TokenKind::BooleanLiteral(val) => Node::Const(Const::Bool(val)),
            TokenKind::Identifier(ref s) if s == "undefined" => Node::Const(Const::Undefined),
            TokenKind::Identifier(s) => Node::Local(s),
            TokenKind::Keyword(keyword) => self.parse_struct(keyword)?,
            TokenKind::RegularExpressionLiteral(body, flags) => Node::Construct(
                Box::new(Node::Local("RegExp".to_string())),
                vec![
                    Node::Const(Const::String(body)),
                    Node::Const(Const::String(flags)),
                ],
            ),
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                match self.get_token(self.pos)?.kind {
                    TokenKind::Punctuator(Punctuator::CloseParen)
                        if self.get_token(self.pos.wrapping_add(1))?.kind
                            == TokenKind::Punctuator(Punctuator::Arrow) =>
                    {
                        self.pos += 2;
                        let expr = self.parse()?;
                        Node::ArrowFunctionDecl(Vec::new(), Box::new(expr))
                    }
                    _ => {
                        let next = self.parse()?;
                        let next_tok = self.get_token(self.pos)?;
                        self.pos += 1;
                        match next_tok.kind {
                            TokenKind::Punctuator(Punctuator::CloseParen) => next,
                            TokenKind::Punctuator(Punctuator::Comma) => {
                                // at this point it's probably gonna be an arrow function
                                // if first param captured all arguments, we should expect a close paren
                                if let Node::UnaryOp(UnaryOp::Spread, _) = next {
                                    return Err(ParseError::Expected(
                                        vec![TokenKind::Punctuator(Punctuator::CloseParen)],
                                        next_tok,
                                        "arrow function",
                                    ));
                                }

                                let mut args = vec![
                                    match next {
                                        Node::Local(ref name) => FormalParameter::new(
                                            name.clone(),
                                            Some(Box::new(Node::Local((*name).clone()))),
                                            false,
                                        ),
                                        _ => FormalParameter::new(String::new(), None, false),
                                    },
                                    match self.get_token(self.pos)?.kind {
                                        TokenKind::Identifier(ref id) => FormalParameter::new(
                                            id.clone(),
                                            Some(Box::new(Node::Local(id.clone()))),
                                            false,
                                        ),
                                        _ => FormalParameter::new(String::new(), None, false),
                                    },
                                ];
                                let mut expect_ident = true;
                                loop {
                                    self.pos += 1;
                                    let curr_tk = self.get_token(self.pos)?;
                                    match curr_tk.kind {
                                        TokenKind::Identifier(ref id) if expect_ident => {
                                            let arg = FormalParameter::new(
                                                id.clone(),
                                                Some(Box::new(Node::Local(id.clone()))),
                                                false,
                                            );
                                            args.push(arg);
                                            expect_ident = false;
                                        }
                                        TokenKind::Punctuator(Punctuator::Comma) => {
                                            expect_ident = true;
                                        }
                                        TokenKind::Punctuator(Punctuator::Spread) => {
                                            let ident_token = self.get_token(self.pos + 1)?;
                                            if let TokenKind::Identifier(ref _id) = ident_token.kind
                                            {
                                                args.push(FormalParameter::new(
                                                    String::new(),
                                                    Some(Box::new(self.parse()?)),
                                                    false,
                                                ));
                                                self.pos -= 1;
                                                expect_ident = false;
                                            } else {
                                                return Err(ParseError::Expected(
                                                    vec![TokenKind::Identifier(
                                                        "identifier".to_string(),
                                                    )],
                                                    ident_token,
                                                    "arrow function",
                                                ));
                                            }
                                        }
                                        TokenKind::Punctuator(Punctuator::CloseParen) => {
                                            self.pos += 1;
                                            break;
                                        }
                                        _ if expect_ident => {
                                            return Err(ParseError::Expected(
                                                vec![TokenKind::Identifier(
                                                    "identifier".to_string(),
                                                )],
                                                curr_tk,
                                                "arrow function",
                                            ))
                                        }
                                        _ => {
                                            return Err(ParseError::Expected(
                                                vec![
                                                    TokenKind::Punctuator(Punctuator::Comma),
                                                    TokenKind::Punctuator(Punctuator::CloseParen),
                                                    TokenKind::Punctuator(Punctuator::Spread),
                                                ],
                                                curr_tk,
                                                "arrow function",
                                            ))
                                        }
                                    }
                                }
                                self.expect(
                                    TokenKind::Punctuator(Punctuator::Arrow),
                                    "arrow function",
                                )?;
                                let expr = self.parse()?;
                                Node::ArrowFunctionDecl(args, Box::new(expr))
                            }
                            _ => {
                                return Err(ParseError::Expected(
                                    vec![TokenKind::Punctuator(Punctuator::CloseParen)],
                                    next_tok,
                                    "brackets",
                                ))
                            }
                        }
                    }
                }
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let mut exprs = Vec::new();
                loop {
                    if self.get_token(self.pos)?.kind
                        == TokenKind::Punctuator(Punctuator::CloseBlock)
                    {
                        break;
                    } else {
                        exprs.push(self.parse()?);
                    }
                }
                self.pos += 1;
                Node::Block(exprs)
            }
            // Empty Block
            TokenKind::Punctuator(Punctuator::CloseBlock)
                if self.get_token(self.pos.wrapping_sub(2))?.kind
                    == TokenKind::Punctuator(Punctuator::OpenBlock) =>
            {
                Node::Block(vec![])
            }
            TokenKind::Punctuator(Punctuator::Spread) => {
                Node::UnaryOp(UnaryOp::Spread, Box::new(self.parse()?))
            }
            _ => return Err(ParseError::Expected(Vec::new(), token.clone(), "script")),
        };
        if self.pos >= self.tokens.len() {
            Ok(node)
        } else {
            self.parse_next(node)
        }
    }

    fn parse_next(&mut self, node: Node) -> ParseResult {
        let next = self.get_token(self.pos)?;
        let mut carry_on = true;
        let mut result = node.clone();
        match next.kind {
            TokenKind::Punctuator(Punctuator::Dot) => {
                self.pos += 1;
                let tk = self.get_token(self.pos)?;
                match tk.kind {
                    TokenKind::Identifier(ref s) => {
                        result = Node::GetConstField(Box::new(node), s.to_string())
                    }
                    _ => {
                        return Err(ParseError::Expected(
                            vec![TokenKind::Identifier("identifier".to_string())],
                            tk,
                            "field access",
                        ))
                    }
                }
                self.pos += 1;
            }
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                let mut args = Vec::new();
                let mut expect_comma_or_end = self.get_token(self.pos.wrapping_add(1))?.kind
                    == TokenKind::Punctuator(Punctuator::CloseParen);
                loop {
                    self.pos += 1;
                    let token = self.get_token(self.pos)?;
                    if token.kind == TokenKind::Punctuator(Punctuator::CloseParen)
                        && expect_comma_or_end
                    {
                        self.pos += 1;
                        break;
                    } else if token.kind == TokenKind::Punctuator(Punctuator::Comma)
                        && expect_comma_or_end
                    {
                        expect_comma_or_end = false;
                    } else if expect_comma_or_end {
                        return Err(ParseError::Expected(
                            vec![
                                TokenKind::Punctuator(Punctuator::Comma),
                                TokenKind::Punctuator(Punctuator::CloseParen),
                            ],
                            token,
                            "function call arguments",
                        ));
                    } else {
                        let parsed = self.parse()?;
                        self.pos -= 1;
                        args.push(parsed);
                        expect_comma_or_end = true;
                    }
                }
                result = Node::Call(Box::new(node), args);
            }
            TokenKind::Punctuator(Punctuator::Question) => {
                self.pos += 1;
                let if_e = self.parse()?;
                self.expect(TokenKind::Punctuator(Punctuator::Colon), "if expression")?;
                let else_e = self.parse()?;
                result = Node::If(Box::new(node), Box::new(if_e), Some(Box::new(else_e)));
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                self.pos += 1;
                let index = self.parse()?;
                self.expect(
                    TokenKind::Punctuator(Punctuator::CloseBracket),
                    "array index",
                )?;
                result = Node::GetField(Box::new(node), Box::new(index));
            }
            TokenKind::Punctuator(Punctuator::Semicolon)
            | TokenKind::LineTerminator => {
                self.pos += 1;
            }
            _ => carry_on = false,
        };
        if carry_on && self.pos < self.tokens.len() {
            self.parse_next(result)
        } else {
            Ok(result)
        }
    }

    /// Returns an error if the next Punctuator is not `tk`
    fn expect(&mut self, tk: TokenKind, routine: &'static str) -> Result<(), ParseError> {
        self.pos += 1;
        let curr_tk = self.get_token(self.pos.wrapping_sub(1))?;
        if curr_tk.kind == tk {
            Ok(())
        } else {
            Err(ParseError::Expected(vec![tk], curr_tk, routine))
        }
    }

    /// Returns an error if the next symbol is not `tk`
    fn expect_no_lineterminator(
        &mut self,
        tk: TokenKind,
        routine: &'static str,
    ) -> Result<(), ParseError> {
        let curr_tk = self.next_skip_lineterminator()?;
        if curr_tk.kind == tk {
            Ok(())
        } else {
            Err(ParseError::Expected(vec![tk], curr_tk, routine))
        }
    }

    /// Returns an error if the next symbol is not the punctuator `p`
    /// Consumes the next symbol otherwise
    fn expect_punc(&mut self, p: Punctuator, routine: &'static str) -> Result<(), ParseError> {
        self.expect(TokenKind::Punctuator(p), routine)
    }

    // New Stuff

    fn read_statement_list(&mut self) -> Result<Node, ParseError> {
        self.read_statements(false, false)
    }

    /// Starts after `{`
    fn read_block_statement(&mut self) -> Result<Node, ParseError> {
        self.read_statements(true, true)
    }

    /// Read a list of statements and stop after `}`
    ///
    /// Note: It starts after `{`.
    fn read_block(&mut self) -> Result<Node, ParseError> {
        self.read_statements(true, false)
    }

    fn read_statements(
        &mut self,
        break_when_closingbrase: bool,
        is_block_statement: bool,
    ) -> Result<Node, ParseError> {
        let mut items = vec![];

        loop {
            match self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::CloseBlock)) {
                Ok(true) => {
                    if break_when_closingbrase {
                        break;
                    } else {
                        return Err(ParseError::Unexpected(
                            self.peek_skip_lineterminator()?,
                            None,
                        ));
                    }
                }
                Ok(false) => {}
                Err(ParseError::NormalEOF) => {}
                Err(e) => return Err(e),
            }

            if let Err(ParseError::NormalEOF) = self.peek_skip_lineterminator() {
                if break_when_closingbrase {
                    return Err(ParseError::Unexpected(
                        self.get_current_token()?,
                        Some("expected '}'"),
                    ));
                } else {
                    break;
                }
            };

            let item = self.read_statement_list_item().map_err(|e| {
                if let ParseError::NormalEOF = e {
                    ParseError::AbruptEnd
                } else {
                    e
                }
            })?;
            items.push(item);

            while match self
                .next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Semicolon))
            {
                Ok(succ) => succ,
                Err(ParseError::NormalEOF) => false,
                Err(e) => return Err(e),
            } {}
        }

        if is_block_statement {
            Ok(Node::Block(items))
        } else {
            Ok(Node::StatementList(items))
        }
    }

    /// <https://tc39.es/ecma262/#prod-StatementListItem>
    fn read_statement_list_item(&mut self) -> Result<Node, ParseError> {
        if let Ok(tok) = self.peek_skip_lineterminator() {
            match tok.kind {
                TokenKind::Keyword(Keyword::Function)
                | TokenKind::Keyword(Keyword::Const)
                | TokenKind::Keyword(Keyword::Let) => self.read_declaration(),
                _ => self.read_statement(),
            }
        } else {
            Err(ParseError::NormalEOF)
        }
    }

    /// <https://tc39.es/ecma262/#prod-Declaration>
    fn read_declaration(&mut self) -> Result<Node, ParseError> {
        let tok = self.next_skip_lineterminator()?;
        match tok.kind {
            TokenKind::Keyword(Keyword::Function) => self.read_function_declaration(),
            TokenKind::Keyword(Keyword::Const) => self.read_lexical_declaration(true),
            TokenKind::Keyword(Keyword::Let) => self.read_lexical_declaration(false),
            _ => unreachable!(),
        }
    }

    /// <https://tc39.es/ecma262/#prod-LexicalDeclaration>
    fn read_lexical_declaration(&mut self, is_const: bool) -> Result<Node, ParseError> {
        // Create vectors to store the variable declarations
        // Const and Let signatures are slightly different, Const needs definitions, Lets don't
        let mut let_decls = vec![];
        let mut const_decls = vec![];

        loop {
            let token = self.next_skip_lineterminator()?;
            let name = if let TokenKind::Identifier(ref name) = token.kind {
                name.clone()
            } else {
                return Err(ParseError::Expected(
                    vec![TokenKind::Identifier(String::from("Identifier"))],
                    token,
                    "Expect identifier.",
                ));
            };

            if self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Assign))? {
                let init = Some(self.read_initializer()?);
                if is_const {
                    const_decls.push((name, init.unwrap()));
                } else {
                    let_decls.push((name, init));
                };
            } else if is_const {
                return Err(ParseError::Expected(
                    vec![TokenKind::Identifier(String::from("Expression"))],
                    token,
                    "Expected Expression for Const!",
                ));
            } else {
                let_decls.push((name, None));
            }

            if !self.variable_declaration_continuation()? {
                break;
            }
        }

        if is_const {
            Ok(Node::ConstDecl(const_decls))
        } else {
            Ok(Node::LetDecl(let_decls))
        }
    }

    /// <https://tc39.es/ecma262/#prod-FunctionDeclaration>
    fn read_function_declaration(&mut self) -> Result<Node, ParseError> {
        let token = self.next_skip_lineterminator()?;
        let name = if let TokenKind::Identifier(name) = token.kind {
            name
        } else {
            return Err(ParseError::Expected(
                vec![TokenKind::Identifier(String::from("function name"))],
                token,
                "expected function name",
            ));
        };

        self.expect(TokenKind::Punctuator(Punctuator::OpenParen), "expected '('")?;

        let params = self.read_formal_parameters()?;

        self.expect(
            TokenKind::Punctuator(Punctuator::OpenBlock),
            "function declaration",
        )?;

        let body = self.read_block()?;

        Ok(Node::FunctionDecl(Some(name), params, Box::new(body)))
    }

    /// <https://tc39.es/ecma262/#prod-Statement>
    fn read_statement(&mut self) -> Result<Node, ParseError> {
        let tok = self.next_skip_lineterminator()?;

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
            TokenKind::Punctuator(Punctuator::OpenBlock) => self.read_block_statement(),
            // TODO: https://tc39.es/ecma262/#prod-LabelledStatement
            // TokenKind::Punctuator(Punctuator::Semicolon) => {
            //     return Ok(Node::new(NodeBase::Nope, tok.pos))
            // }
            _ => {
                self.step_back();
                is_expression_statement = true;
                self.read_expression_statement()
            }
        };

        match self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Semicolon)) {
            Ok(true) | Err(ParseError::NormalEOF) => {}
            Ok(false) => {
                if is_expression_statement {
                    match self.peek(0)?.kind {
                        TokenKind::LineTerminator
                        | TokenKind::Punctuator(Punctuator::CloseBlock) => {}
                        _ => {
                            return Err(ParseError::Unexpected(self.get_current_token()?, None));
                        }
                    }
                }
            }
            Err(e) => return Err(e),
        }

        stmt
    }

    /// <https://tc39.es/ecma262/#sec-expression-statement>
    fn read_expression_statement(&mut self) -> Result<Node, ParseError> {
        self.read_expression()
    }

    /// <https://tc39.es/ecma262/#sec-break-statement>
    fn read_break_statement(&mut self) -> Result<Node, ParseError> {
        let tok = self.get_next_token()?;
        match tok.kind {
            TokenKind::LineTerminator
            | TokenKind::Punctuator(Punctuator::Semicolon)
            | TokenKind::Punctuator(Punctuator::CloseParen) => {
                self.step_back();
                Ok(Node::Break(None))
            }
            TokenKind::Identifier(name) => Ok(Node::Break(Some(name))),
            _ => Err(ParseError::Unexpected(
                tok,
                Some("expected ';', identifier or line terminator"),
            )),
        }
    }

    /// <https://tc39.es/ecma262/#sec-continue-statement>
    fn read_continue_statement(&mut self) -> Result<Node, ParseError> {
        let tok = self.get_next_token()?;
        match tok.kind {
            TokenKind::LineTerminator
            | TokenKind::Punctuator(Punctuator::Semicolon)
            | TokenKind::Punctuator(Punctuator::CloseBlock) => {
                self.step_back();
                Ok(Node::Continue(None))
            }
            TokenKind::Identifier(name) => Ok(Node::Continue(Some(name))),
            _ => Err(ParseError::Unexpected(
                tok,
                Some("expected ';', identifier or line terminator"),
            )),
        }
    }

    /// <https://tc39.github.io/ecma262/#prod-ThrowStatement>
    fn read_throw_statement(&mut self) -> Result<Node, ParseError> {
        // no LineTerminator here
        if self.next_if(TokenKind::LineTerminator).is_some() {
            return Err(ParseError::General("Illegal new line after throw"));
        }

        if self
            .next_if(TokenKind::Punctuator(Punctuator::Semicolon))
            .is_some()
        {
            return Err(ParseError::General("Unexpected token ;"));
        }

        if self.peek(0)?.kind == TokenKind::Punctuator(Punctuator::CloseBlock) {
            return Err(ParseError::Unexpected(self.get_next_token()?, None));
        }

        let expr = self.read_expression()?;
        self.next_if(TokenKind::Punctuator(Punctuator::Semicolon));

        Ok(Node::Throw(Box::new(expr)))
    }

    /// <https://tc39.es/ecma262/#prod-ReturnStatement>
    fn read_return_statement(&mut self) -> Result<Node, ParseError> {
        if self.next_if(TokenKind::LineTerminator).is_some() {
            return Ok(Node::Return(None));
        }

        if self
            .next_if(TokenKind::Punctuator(Punctuator::Semicolon))
            .is_some()
        {
            return Ok(Node::Return(None));
        }

        if self.peek(0)?.kind == TokenKind::Punctuator(Punctuator::CloseBlock) {
            return Ok(Node::Return(None));
        }

        let expr = self.read_expression()?;

        self.next_if(TokenKind::Punctuator(Punctuator::Semicolon));

        Ok(Node::Return(Some(Box::new(expr))))
    }

    /// <https://tc39.es/ecma262/#sec-if-statement>
    fn read_if_statement(&mut self) -> Result<Node, ParseError> {
        let oparen = self.get_next_token()?;
        if oparen.kind != TokenKind::Punctuator(Punctuator::OpenParen) {
            return Err(ParseError::Expected(
                vec![TokenKind::Punctuator(Punctuator::OpenParen)],
                oparen,
                "Expected '('",
            ));
        }
        let cond = self.read_expression()?;
        let cparen = self.get_next_token()?;
        if cparen.kind != TokenKind::Punctuator(Punctuator::CloseParen) {
            return Err(ParseError::Expected(
                vec![TokenKind::Punctuator(Punctuator::OpenParen)],
                cparen,
                "Expected ')'",
            ));
        }

        let then_ = self.read_statement()?;

        if let Ok(expect_else_tok) = self.get_next_token() {
            if expect_else_tok.kind == TokenKind::Keyword(Keyword::Else) {
                let else_ = self.read_statement()?;
                return Ok(Node::If(
                    Box::new(cond),
                    Box::new(then_),
                    Some(Box::new(else_)),
                ));
            } else {
                self.step_back();
            }
        }

        Ok(Node::If(Box::new(cond), Box::new(then_), None))
    }

    /// <https://tc39.es/ecma262/#sec-while-statement>
    fn read_while_statement(&mut self) -> Result<Node, ParseError> {
        self.expect(TokenKind::Punctuator(Punctuator::OpenParen), "expected '('")?;

        let cond = self.read_expression()?;

        self.expect(
            TokenKind::Punctuator(Punctuator::CloseParen),
            "expected ')'",
        )?;

        let body = self.read_statement()?;

        Ok(Node::WhileLoop(Box::new(cond), Box::new(body)))
    }

    /// <https://tc39.es/ecma262/#sec-try-statement>
    fn read_try_statement(&mut self) -> Result<Node, ParseError> {
        // TRY
        self.expect_punc(Punctuator::OpenBlock, "Expected open brace {")?;
        let try_clause = self.read_block_statement()?;
        let is_catch = self
            .next_if_skip_lineterminator(TokenKind::Keyword(Keyword::Catch))
            .unwrap_or(false);

        // CATCH
        let (catch, param) = if is_catch {
            self.expect_punc(Punctuator::OpenParen, "Expected opening parenthesis (")?;
            // TODO: should accept BindingPattern
            let tok = self.get_next_token()?;
            let catch_param = if let TokenKind::Identifier(s) = tok.kind {
                Node::Local(s)
            } else {
                return Err(ParseError::Unexpected(tok, Some("expected identifier")));
            };
            self.expect_punc(Punctuator::CloseParen, "Expected )")?;
            self.expect_punc(Punctuator::OpenBlock, "Expected {")?;
            (
                Some(Box::new(self.read_block()?)),
                Some(Box::new(catch_param)),
            )
        } else {
            (None, None)
        };

        // FINALLY
        let is_finally = self
            .next_if_skip_lineterminator(TokenKind::Keyword(Keyword::Finally))
            .unwrap_or(false);
        let finally = if is_finally {
            self.expect_punc(Punctuator::OpenBlock, "Expected {")?;
            Some(Box::new(self.read_block_statement()?))
        } else {
            None
        };

        Ok(Node::Try(Box::new(try_clause), catch, param, finally))
    }

    /// <https://tc39.es/ecma262/#sec-for-statement>
    fn read_for_statement(&mut self) -> Result<Node, ParseError> {
        self.expect(TokenKind::Punctuator(Punctuator::OpenParen), "expected '('")?;

        let init = match self.peek(0)?.kind {
            TokenKind::Keyword(Keyword::Var) => {
                assert_eq!(
                    self.get_next_token()?.kind,
                    TokenKind::Keyword(Keyword::Var)
                );
                Some(Box::new(self.read_variable_declaration_list()?))
            }
            TokenKind::Keyword(Keyword::Let) | TokenKind::Keyword(Keyword::Const) => {
                Some(Box::new(self.read_declaration()?))
            }
            TokenKind::Punctuator(Punctuator::Semicolon) => None,
            _ => Some(Box::new(self.read_expression()?)),
        };

        self.expect(TokenKind::Punctuator(Punctuator::Semicolon), "expect ';'")?;

        let cond = if self
            .next_if(TokenKind::Punctuator(Punctuator::Semicolon))
            .is_some()
        {
            Some(Box::new(Node::Const(Const::Bool(true))))
        } else {
            let step = Some(Box::new(self.read_expression()?));
            self.expect(TokenKind::Punctuator(Punctuator::Semicolon), "expected ';'")?;
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
                "expected ')'",
            )?;
            Some(Box::new(step))
        };

        let body = Box::new(self.read_statement()?);

        let for_node = Node::ForLoop(init, cond, step, body);

        Ok(Node::Block(vec![for_node]))
    }
    /// <https://tc39.es/ecma262/#prod-VariableStatement>
    fn read_variable_statement(&mut self) -> Result<Node, ParseError> {
        self.read_variable_declaration_list()
    }

    /// <https://tc39.es/ecma262/#prod-VariableDeclarationList>
    fn read_variable_declaration_list(&mut self) -> Result<Node, ParseError> {
        let mut list = Vec::new();

        loop {
            list.push(self.read_variable_declaration()?);
            if !self.variable_declaration_continuation()? {
                break;
            }
        }

        Ok(Node::VarDecl(list))
    }

    fn variable_declaration_continuation(&mut self) -> Result<bool, ParseError> {
        let mut newline_found = false;

        for _ in 0.. {
            match self.peek(0) {
                Ok(tok) => match tok.kind {
                    TokenKind::LineTerminator => newline_found = true,
                    TokenKind::Punctuator(Punctuator::Semicolon) => {
                        return Ok(false);
                    }
                    TokenKind::Punctuator(Punctuator::Comma) => {
                        self.next_skip_lineterminator().unwrap();
                        return Ok(true);
                    }
                    _ if newline_found => return Ok(false),
                    _ => break,
                },
                Err(_) => return Ok(false),
            }
        }

        Err(ParseError::Expected(
            vec![
                TokenKind::Punctuator(Punctuator::Semicolon),
                TokenKind::LineTerminator,
            ],
            self.get_current_token()?,
            "variable declaration",
        ))
    }

    /// <https://tc39.es/ecma262/#prod-VariableDeclaration>
    fn read_variable_declaration(&mut self) -> Result<(String, Option<Node>), ParseError> {
        let tok = self.next_skip_lineterminator()?;
        let name = if let TokenKind::Identifier(name) = tok.kind {
            name
        } else {
            return Err(ParseError::Expected(
                vec![TokenKind::Identifier("identifier".to_string())],
                tok,
                "Expect identifier.",
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

    /// <https://tc39.es/ecma262/#prod-Initializer>
    fn read_initializer(&mut self) -> Result<Node, ParseError> {
        self.read_assignment_expression()
    }

    // https://tc39.github.io/ecma262/#prod-Expression
    expression!(
        read_expression,
        read_assignment_expression,
        [Punctuator::Comma]
    );

    /// <https://tc39.es/ecma262/#prod-AssignmentExpression>
    fn read_assignment_expression(&mut self) -> Result<Node, ParseError> {
        // Arrow function
        let next_token = self.peek(0)?;
        match next_token.kind {
            // (a,b)=>{}
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                let save_pos = self.get_current_pos();
                let f = self.read_arrow_function(true);
                if f.is_err() {
                    self.set_current_pos(save_pos)?;
                } else {
                    return f;
                }
            }
            // a=>{}
            TokenKind::Identifier(_) => {
                if let Ok(tok) = self.peek(1) {
                    if tok.kind == TokenKind::Punctuator(Punctuator::Arrow) {
                        return self.read_arrow_function(false);
                    }
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
                _ => self.step_back(),
            }
        }

        Ok(lhs)
    }

    /// <https://tc39.es/ecma262/#prod-ConditionalExpression>
    fn read_conditional_expression(&mut self) -> Result<Node, ParseError> {
        let lhs = self.read_logical_or_expression()?;

        if let Ok(tok) = self.get_next_token() {
            match tok.kind {
                TokenKind::Punctuator(Punctuator::Question) => {
                    let then_ = self.read_assignment_expression()?;
                    self.expect_punc(Punctuator::Colon, "expect ':'")?;
                    let else_ = self.read_assignment_expression()?;
                    return Ok(Node::ConditionalOp(
                        Box::new(lhs),
                        Box::new(then_),
                        Box::new(else_),
                    ));
                }
                _ => self.step_back(),
            }
        }

        Ok(lhs)
    }

    /// <https://tc39.es/ecma262/#sec-arrow-function-definitions>
    fn read_arrow_function(&mut self, is_parenthesized_param: bool) -> Result<Node, ParseError> {
        let params = if is_parenthesized_param {
            self.expect_punc(Punctuator::OpenParen, "expect '('")?;
            self.read_formal_parameters()?
        } else {
            let param_name = match self.get_next_token()?.kind {
                TokenKind::Identifier(s) => s,
                _ => unreachable!(),
            };
            vec![FormalParameter {
                init: None,
                name: param_name,
                is_rest_param: false,
            }]
        };

        self.expect_punc(Punctuator::Arrow, "arrow function declaration")?;

        let body =
            if self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::OpenBlock))? {
                self.read_block()?
            } else {
                Node::Return(Some(Box::new(self.read_assignment_expression()?)))
            };

        Ok(Node::ArrowFunctionDecl(params, Box::new(body)))
    }

    /// Collect parameters from functions or arrow functions
    fn read_formal_parameters(&mut self) -> Result<FormalParameters, ParseError> {
        if self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::CloseParen))? {
            return Ok(vec![]);
        }

        let mut params = vec![];

        loop {
            let mut rest_param = false;

            params.push(
                if self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Spread))? {
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
                    self.get_current_token()?,
                    Some("rest parameter must be the last formal parameter"),
                ));
            }

            self.expect(TokenKind::Punctuator(Punctuator::Comma), "expect ','")?;
        }

        Ok(params)
    }

    /// <https://tc39.es/ecma262/#prod-FunctionRestParameter>
    fn read_function_rest_parameter(&mut self) -> Result<FormalParameter, ParseError> {
        let token = self.get_next_token()?;
        Ok(FormalParameter::new(
            if let TokenKind::Identifier(name) = token.kind {
                name
            } else {
                return Err(ParseError::Expected(
                    vec![TokenKind::Identifier(String::from("identifier"))],
                    token,
                    "rest params: expect identifier",
                ));
            },
            None,
            true,
        ))
    }

    /// <https://tc39.es/ecma262/#prod-FormalParameter>
    fn read_formal_parameter(&mut self) -> Result<FormalParameter, ParseError> {
        let token = self.next_skip_lineterminator()?;
        let name = if let TokenKind::Identifier(name) = token.kind {
            name
        } else {
            return Err(ParseError::Expected(
                vec![TokenKind::Identifier(String::from("identifier"))],
                token,
                "expect identifier (unsupported feature)",
            ));
        };

        // TODO: Implement initializer.
        Ok(FormalParameter::new(name, None, false))
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
    fn read_exponentiation_expression(&mut self) -> Result<Node, ParseError> {
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
                self.step_back();
            }
        }
        Ok(lhs)
    }

    // Checks by looking at the next token to see whether its a Unary operator or not.
    fn is_unary_expression(&mut self) -> bool {
        match self.peek(0) {
            Ok(ok) => match ok.kind {
                TokenKind::Keyword(Keyword::Delete)
                | TokenKind::Keyword(Keyword::Void)
                | TokenKind::Keyword(Keyword::TypeOf)
                | TokenKind::Punctuator(Punctuator::Add)
                | TokenKind::Punctuator(Punctuator::Sub)
                | TokenKind::Punctuator(Punctuator::Not)
                | TokenKind::Punctuator(Punctuator::Neg) => true,
                _ => false,
            },
            Err(_) => false,
        }
    }

    /// <https://tc39.es/ecma262/#prod-UnaryExpression>
    fn read_unary_expression(&mut self) -> Result<Node, ParseError> {
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
                self.step_back();
                self.read_update_expression()
            }
        }
    }

    /// <https://tc39.es/ecma262/#prod-UpdateExpression>
    fn read_update_expression(&mut self) -> Result<Node, ParseError> {
        let tok = self.peek_skip_lineterminator()?;
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
        if let Ok(tok) = self.peek(0) {
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

    /// <https://tc39.github.io/ecma262/#prod-LeftHandSideExpression>
    /// TODO: Implement NewExpression: new MemberExpression
    fn read_left_hand_side_expression(&mut self) -> Result<Node, ParseError> {
        let lhs = self.read_member_expression()?;
        match self.peek_skip_lineterminator() {
            Ok(ref tok) if tok.kind == TokenKind::Punctuator(Punctuator::OpenParen) => {
                self.read_call_expression(lhs)
            }
            _ => self.read_new_expression(lhs),
        }
    }

    /// <https://tc39.github.io/ecma262/#prod-NewExpression>
    fn read_new_expression(&mut self, first_member_expr: Node) -> Result<Node, ParseError> {
        Ok(first_member_expr)
    }

    /// <https://tc39.es/ecma262/#prod-MemberExpression>
    fn read_member_expression(&mut self) -> Result<Node, ParseError> {
        let mut lhs = if self.peek_skip_lineterminator()?.kind == TokenKind::Keyword(Keyword::New) {
            self.next_skip_lineterminator()?;
            let lhs = self.read_member_expression()?;
            self.expect_punc(Punctuator::OpenParen, "expect '('.")?;
            let args = self.read_arguments()?;
            let call_node = Node::Call(Box::new(lhs), args);

            Node::New(Box::new(call_node))
        } else {
            self.read_primary_expression()?
        };
        while let Ok(tok) = self.next_skip_lineterminator() {
            match tok.kind {
                TokenKind::Punctuator(Punctuator::Dot) => {
                    match self.next_skip_lineterminator()?.kind {
                        TokenKind::Identifier(name) => {
                            lhs = Node::GetConstField(Box::new(lhs), name)
                        }
                        TokenKind::Keyword(kw) => {
                            lhs = Node::GetConstField(Box::new(lhs), kw.to_string())
                        }
                        _ => {
                            return Err(ParseError::Expected(
                                vec![TokenKind::Identifier(String::from("Identifier"))],
                                tok,
                                "expect identifier",
                            ));
                        }
                    }
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    let idx = self.read_expression()?;
                    self.expect_punc(Punctuator::CloseBracket, "Expected ]")?;
                    lhs = Node::GetField(Box::new(lhs), Box::new(idx));
                }
                _ => {
                    self.step_back();
                    break;
                }
            }
        }

        Ok(lhs)
    }

    /// <https://tc39.github.io/ecma262/#prod-CallExpression>
    fn read_call_expression(&mut self, first_member_expr: Node) -> Result<Node, ParseError> {
        let mut lhs = first_member_expr;
        if let Ok(true) =
            self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::OpenParen))
        {
            let args = self.read_arguments()?;
            lhs = Node::Call(Box::new(lhs), args);
        } else {
            panic!("CallExpression MUST start with MemberExpression.");
        }

        while let Ok(tok) = self.next_skip_lineterminator() {
            match tok.kind {
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    let args = self.read_arguments()?;
                    lhs = Node::Call(Box::new(lhs), args);
                }
                TokenKind::Punctuator(Punctuator::Dot) => {
                    match self.next_skip_lineterminator()?.kind {
                        TokenKind::Identifier(name) => {
                            lhs = Node::GetConstField(Box::new(lhs), name);
                        }
                        TokenKind::Keyword(kw) => {
                            lhs = Node::GetConstField(Box::new(lhs), kw.to_string());
                        }
                        _ => {
                            return Err(ParseError::Expected(
                                vec![TokenKind::Identifier(String::from("Identifier"))],
                                tok,
                                "expect identifier",
                            ));
                        }
                    }
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    let idx = self.read_expression()?;
                    self.expect_punc(Punctuator::CloseBracket, "expected ]")?;
                    lhs = Node::GetField(Box::new(lhs), Box::new(idx));
                }
                _ => {
                    self.step_back();
                    break;
                }
            }
        }

        Ok(lhs)
    }

    /// <https://tc39.es/ecma262/#prod-Arguments>
    fn read_arguments(&mut self) -> Result<Vec<Node>, ParseError> {
        let mut args = vec![];
        loop {
            match self.next_skip_lineterminator() {
                Ok(ref tok) if tok.kind == TokenKind::Punctuator(Punctuator::CloseParen) => break,
                Ok(ref tok) if tok.kind == TokenKind::Punctuator(Punctuator::Comma) => {
                    if args.is_empty() {
                        return Err(ParseError::Unexpected(tok.clone(), None));
                    }
                    if self.next_if_skip_lineterminator(TokenKind::Punctuator(
                        Punctuator::CloseParen,
                    ))? {
                        break;
                    }
                }
                Ok(tok) => {
                    if !args.is_empty() {
                        return Err(ParseError::Expected(
                            vec![
                                TokenKind::Punctuator(Punctuator::Comma),
                                TokenKind::Punctuator(Punctuator::CloseParen),
                            ],
                            tok,
                            "expected ',' or ')'.",
                        ));
                    } else {
                        self.step_back();
                    }
                }
                Err(_) => {
                    return Err(ParseError::AbruptEnd);
                }
            }

            args.push(self.read_assignment_expression()?);
        }

        Ok(args)
    }

    /// <https://tc39.es/ecma262/#prod-PrimaryExpression>
    fn read_primary_expression(&mut self) -> Result<Node, ParseError> {
        let tok = self.next_skip_lineterminator()?;

        match tok.kind {
            TokenKind::Keyword(Keyword::This) => Ok(Node::This),
            // TokenKind::Keyword(Keyword::Arguments) => Ok(Node::new(NodeBase::Arguments, tok.pos)),
            TokenKind::Keyword(Keyword::Function) => self.read_function_expression(),
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                let expr = self.read_expression();
                self.expect_punc(Punctuator::CloseParen, "Expect punc")?;
                expr
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => self.read_array_literal(),
            TokenKind::Punctuator(Punctuator::OpenBlock) => self.read_object_literal(),
            TokenKind::BooleanLiteral(boolean) => Ok(Node::Const(Const::Bool(boolean))),
            // TODO: ADD TokenKind::UndefinedLiteral
            TokenKind::Identifier(ref i) if i == "undefined" => Ok(Node::Const(Const::Undefined)),
            TokenKind::NullLiteral => Ok(Node::Const(Const::Null)),
            TokenKind::Identifier(ident) => Ok(Node::Local(ident)),
            TokenKind::StringLiteral(s) => Ok(Node::Const(Const::String(s))),
            TokenKind::NumericLiteral(num) => Ok(Node::Const(Const::Num(num))),
            _ => Err(ParseError::Unexpected(tok, None)),
        }
    }

    /// <https://tc39.es/ecma262/#prod-FunctionDeclaration>
    fn read_function_expression(&mut self) -> Result<Node, ParseError> {
        let name = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            self.get_next_token()?;
            Some(name)
        } else {
            None
        };

        self.expect_punc(Punctuator::OpenParen, "expect '('")?;

        let params = self.read_formal_parameters()?;

        self.expect_punc(Punctuator::OpenBlock, "expect '{'")?;

        let body = self.read_block()?;

        Ok(Node::FunctionDecl(name, params, Box::new(body)))
    }

    /// <https://tc39.github.io/ecma262/#prod-ArrayLiteral>
    fn read_array_literal(&mut self) -> Result<Node, ParseError> {
        let mut elements = vec![];

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

            if self.tokens.is_empty() {
                return Err(ParseError::AbruptEnd);
            }

            if self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Spread))? {
                let node = self.read_assignment_expression()?;
                elements.push(Node::Spread(Box::new(node)));
            } else {
                elements.push(self.read_assignment_expression()?);
            }
            self.next_if(TokenKind::Punctuator(Punctuator::Comma));
        }

        Ok(Node::ArrayDecl(elements))
    }

    /// <https://tc39.github.io/ecma262/#prod-ObjectLiteral>
    fn read_object_literal(&mut self) -> Result<Node, ParseError> {
        let mut elements = Vec::new();

        loop {
            if self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::CloseBlock))? {
                break;
            }

            elements.push(self.read_property_definition()?);

            if self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::CloseBlock))? {
                break;
            }

            if !self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Comma))? {
                return Err(ParseError::Expected(
                    vec![
                        TokenKind::Punctuator(Punctuator::Comma),
                        TokenKind::Punctuator(Punctuator::CloseBlock),
                    ],
                    self.get_current_token()?,
                    "expect ',' or '}'.",
                ));
            }
        }

        Ok(Node::Object(elements))
    }

    /// <https://tc39.github.io/ecma262/#prod-PropertyDefinition>
    fn read_property_definition(&mut self) -> Result<PropertyDefinition, ParseError> {
        fn to_string(kind: TokenKind) -> String {
            match kind {
                TokenKind::Identifier(name) => name,
                TokenKind::NumericLiteral(n) => format!("{}", n),
                TokenKind::StringLiteral(s) => s,
                _ => unimplemented!(),
            }
        }

        if self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Spread))? {
            let node = self.read_assignment_expression()?;
            return Ok(PropertyDefinition::SpreadObject(node));
        }

        let tok = self.next_skip_lineterminator()?;

        if self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::Colon))? {
            let val = self.read_assignment_expression()?;
            return Ok(PropertyDefinition::Property(to_string(tok.kind), val));
        }

        // TODO: Slit into separate function: read_propery_method_definition
        if self.next_if_skip_lineterminator(TokenKind::Punctuator(Punctuator::OpenParen))? {
            let params = self.read_formal_parameters()?;

            self.expect(
                TokenKind::Punctuator(Punctuator::OpenBlock),
                "property method definition",
            )?;

            let body = self.read_block()?;

            return Ok(PropertyDefinition::MethodDefinition(
                MethodDefinitionKind::Ordinary,
                to_string(tok.kind),
                Node::FunctionDecl(None, params, Box::new(body)),
            ));
        }

        // TODO need to revisit this
        // if let TokenKind::Identifier(name) = tok.kind {
        //     if name == "get" || name == "set" {
        //         let may_identifier = self.peek_skip_lineterminator();
        //         if may_identifier.is_ok()
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

        Err(ParseError::General("Expect property definition."))
    }
}
