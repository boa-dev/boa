use crate::syntax::ast::constant::Const;
use crate::syntax::ast::expr::{Expr, ExprDef};
use crate::syntax::ast::keyword::Keyword;
use crate::syntax::ast::op::{AssignOp, BinOp, BitOp, CompOp, LogOp, NumOp, Operator, UnaryOp};
use crate::syntax::ast::punc::Punctuator;
use crate::syntax::ast::token::{Token, TokenData};
use std::collections::btree_map::BTreeMap;
use std::fmt;

/// `ParseError` is an enum which represents errors encounted during parsing an expression
#[derive(Debug, Clone)]
pub enum ParseError {
    /// When it expected a certain kind of token, but got another as part of something
    Expected(Vec<TokenData>, Token, &'static str),
    /// When it expected a certain expression, but got another
    ExpectedExpr(&'static str, Expr),
    /// When it didn't expect this keyword
    UnexpectedKeyword(Keyword),
    /// When there is an abrupt end to the parsing
    AbruptEnd,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::Expected(expected, actual, routine) => write!(
                f,
                "Expected token '{}', got '{}' in routine '{}'",
                expected
                    .first()
                    .map(|t| t.to_string())
                    .unwrap_or_else(String::new),
                actual,
                routine
            ),
            ParseError::ExpectedExpr(expected, actual) => {
                write!(f, "Expected expression '{}', got '{}'", expected, actual)
            }
            ParseError::UnexpectedKeyword(keyword) => write!(f, "Unexpected keyword: {}", keyword),
            ParseError::AbruptEnd => write!(f, "Abrupt End"),
        }
    }
}

pub type ParseResult = Result<Expr, ParseError>;

#[derive(Debug)]
pub struct Parser {
    /// The tokens being input
    tokens: Vec<Token>,
    /// The current position within the tokens
    pos: usize,
}

impl Parser {
    /// Create a new parser, using `tokens` as input
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse all expressions in the token array
    pub fn parse_all(&mut self) -> ParseResult {
        let mut exprs = Vec::new();
        while self.pos < self.tokens.len() {
            let result = self.parse()?;
            exprs.push(result);
        }

        Ok(Expr::new(ExprDef::Block(exprs)))
    }

    fn get_token(&self, pos: usize) -> Result<Token, ParseError> {
        if pos < self.tokens.len() {
            Ok(self.tokens.get(pos).expect("failed getting token").clone())
        } else {
            Err(ParseError::AbruptEnd)
        }
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Expr>, ParseError> {
        self.expect_punc(Punctuator::OpenParen, "function parameters ( expected")?;
        let mut args = Vec::new();
        let mut tk = self.get_token(self.pos)?;
        while tk.data != TokenData::Punctuator(Punctuator::CloseParen) {
            match tk.data {
                TokenData::Identifier(ref id) => args.push(Expr::new(ExprDef::Local(id.clone()))),
                TokenData::Punctuator(Punctuator::Spread) => {
                    args.push(self.parse()?);
                    self.pos -= 1; // roll back so we're sitting on the closeParen ')'
                }
                _ => {
                    return Err(ParseError::Expected(
                        vec![TokenData::Identifier("identifier".to_string())],
                        tk,
                        "function arguments",
                    ))
                }
            }
            self.pos += 1;
            if self.get_token(self.pos)?.data == TokenData::Punctuator(Punctuator::Comma) {
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
                Ok(Expr::new(ExprDef::Throw(Box::new(thrown))))
            }
            // vars, lets and consts are similar in parsing structure, we can group them together
            Keyword::Var | Keyword::Let => {
                let mut vars = Vec::new();
                loop {
                    let name = match self.get_token(self.pos) {
                        Ok(Token {
                            data: TokenData::Identifier(ref name),
                            ..
                        }) => name.clone(),
                        Ok(tok) => {
                            return Err(ParseError::Expected(
                                vec![TokenData::Identifier("identifier".to_string())],
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
                            data: TokenData::Punctuator(Punctuator::Assign),
                            ..
                        }) => {
                            self.pos += 1;
                            let val = self.parse()?;
                            vars.push((name, Some(val)));
                            match self.get_token(self.pos) {
                                Ok(Token {
                                    data: TokenData::Punctuator(Punctuator::Comma),
                                    ..
                                }) => self.pos += 1,
                                _ => break,
                            }
                        }
                        Ok(Token {
                            data: TokenData::Punctuator(Punctuator::Comma),
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
                    Keyword::Let => Ok(Expr::new(ExprDef::LetDecl(vars))),
                    _ => Ok(Expr::new(ExprDef::VarDecl(vars))),
                }
            }
            Keyword::Const => {
                let mut vars = Vec::new();
                loop {
                    let name = match self.get_token(self.pos) {
                        Ok(Token {
                            data: TokenData::Identifier(ref name),
                            ..
                        }) => name.clone(),
                        Ok(tok) => {
                            return Err(ParseError::Expected(
                                vec![TokenData::Identifier("identifier".to_string())],
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
                            data: TokenData::Punctuator(Punctuator::Assign),
                            ..
                        }) => {
                            self.pos += 1;
                            let val = self.parse()?;
                            vars.push((name, val));
                            match self.get_token(self.pos) {
                                Ok(Token {
                                    data: TokenData::Punctuator(Punctuator::Comma),
                                    ..
                                }) => self.pos += 1,
                                _ => break,
                            }
                        }
                        Ok(tok) => {
                            return Err(ParseError::Expected(
                                vec![TokenData::Punctuator(Punctuator::Assign)],
                                tok,
                                "const declration",
                            ))
                        }
                        _ => break,
                    }
                }

                Ok(Expr::new(ExprDef::ConstDecl(vars)))
            }
            Keyword::Return => Ok(Expr::new(ExprDef::Return(Some(Box::new(self.parse()?))))),
            Keyword::New => {
                let call = self.parse()?;
                match call.def {
                    ExprDef::Call(ref func, ref args) => {
                        Ok(Expr::new(ExprDef::Construct(func.clone(), args.clone())))
                    }
                    _ => Err(ParseError::ExpectedExpr("constructor", call)),
                }
            }
            Keyword::TypeOf => Ok(Expr::new(ExprDef::TypeOf(Box::new(self.parse()?)))),
            Keyword::If => {
                self.expect_punc(Punctuator::OpenParen, "if block")?;
                let cond = self.parse()?;
                self.expect_punc(Punctuator::CloseParen, "if block")?;
                let expr = self.parse()?;
                let next = self.get_token(self.pos);
                Ok(Expr::new(ExprDef::If(
                    Box::new(cond),
                    Box::new(expr),
                    if next.is_ok()
                        && next.expect("Could not get next value").data
                            == TokenData::Keyword(Keyword::Else)
                    {
                        self.pos += 1;
                        Some(Box::new(self.parse()?))
                    } else {
                        None
                    },
                )))
            }
            Keyword::While => {
                self.expect_punc(Punctuator::OpenParen, "while condition")?;
                let cond = self.parse()?;
                self.expect_punc(Punctuator::CloseParen, "while condition")?;
                let expr = self.parse()?;
                Ok(Expr::new(ExprDef::WhileLoop(
                    Box::new(cond),
                    Box::new(expr),
                )))
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
                    match tok.data {
                        TokenData::Keyword(Keyword::Case) => {
                            let cond = self.parse();
                            let mut block = Vec::new();
                            self.expect_punc(Punctuator::Colon, "switch case")?;
                            loop {
                                match self.get_token(self.pos)?.data {
                                    TokenData::Keyword(Keyword::Case)
                                    | TokenData::Keyword(Keyword::Default)
                                    | TokenData::Punctuator(Punctuator::CloseBlock) => break,
                                    _ => block.push(self.parse()?),
                                }
                            }
                            cases.push((cond.expect("No condition supplied"), block));
                        }
                        TokenData::Keyword(Keyword::Default) => {
                            let mut block = Vec::new();
                            self.expect_punc(Punctuator::Colon, "default switch case")?;
                            loop {
                                match self.get_token(self.pos)?.data {
                                    TokenData::Keyword(Keyword::Case)
                                    | TokenData::Keyword(Keyword::Default)
                                    | TokenData::Punctuator(Punctuator::CloseBlock) => break,
                                    _ => block.push(self.parse()?),
                                }
                            }
                            default = Some(Expr::new(ExprDef::Block(block)));
                        }
                        TokenData::Punctuator(Punctuator::CloseBlock) => break,
                        _ => {
                            return Err(ParseError::Expected(
                                vec![
                                    TokenData::Keyword(Keyword::Case),
                                    TokenData::Keyword(Keyword::Default),
                                    TokenData::Punctuator(Punctuator::CloseBlock),
                                ],
                                tok,
                                "switch block",
                            ))
                        }
                    }
                }
                self.expect_punc(Punctuator::CloseBlock, "switch block")?;
                Ok(Expr::new(ExprDef::Switch(
                    Box::new(value.expect("Could not get value")),
                    cases,
                    match default {
                        Some(v) => Some(Box::new(v)),
                        None => None,
                    },
                )))
            }
            Keyword::Function => {
                // function [identifier] () { etc }
                let tk = self.get_token(self.pos)?;
                let name = match tk.data {
                    TokenData::Identifier(ref name) => {
                        self.pos += 1;
                        Some(name.clone())
                    }
                    TokenData::Punctuator(Punctuator::OpenParen) => None,
                    _ => {
                        return Err(ParseError::Expected(
                            vec![TokenData::Identifier("identifier".to_string())],
                            tk,
                            "function name",
                        ))
                    }
                };
                // Now we have the function identifier we should have an open paren for arguments ( )
                let args = self.parse_function_parameters()?;
                let block = self.parse()?;
                Ok(Expr::new(ExprDef::FunctionDecl(
                    name,
                    args,
                    Box::new(block),
                )))
            }
            _ => Err(ParseError::UnexpectedKeyword(keyword)),
        }
    }

    /// Parse a single expression
    pub fn parse(&mut self) -> ParseResult {
        if self.pos > self.tokens.len() {
            return Err(ParseError::AbruptEnd);
        }
        let token = self.get_token(self.pos)?;
        self.pos += 1;
        let expr: Expr = match token.data {
            TokenData::Punctuator(Punctuator::Semicolon) | TokenData::Comment(_)
                if self.pos < self.tokens.len() =>
            {
                self.parse()?
            }
            TokenData::Punctuator(Punctuator::Semicolon) | TokenData::Comment(_) => {
                Expr::new(ExprDef::Const(Const::Undefined))
            }
            TokenData::NumericLiteral(num) => Expr::new(ExprDef::Const(Const::Num(num))),
            TokenData::NullLiteral => Expr::new(ExprDef::Const(Const::Null)),
            TokenData::StringLiteral(text) => Expr::new(ExprDef::Const(Const::String(text))),
            TokenData::BooleanLiteral(val) => Expr::new(ExprDef::Const(Const::Bool(val))),
            TokenData::Identifier(ref s) if s == "undefined" => {
                Expr::new(ExprDef::Const(Const::Undefined))
            }
            TokenData::Identifier(s) => Expr::new(ExprDef::Local(s)),
            TokenData::Keyword(keyword) => self.parse_struct(keyword)?,
            TokenData::RegularExpressionLiteral(body, flags) => Expr::new(ExprDef::Construct(
                Box::new(Expr::new(ExprDef::Local("RegExp".to_string()))),
                vec![
                    Expr::new(ExprDef::Const(Const::String(body))),
                    Expr::new(ExprDef::Const(Const::String(flags))),
                ],
            )),
            TokenData::Punctuator(Punctuator::OpenParen) => {
                match self.get_token(self.pos)?.data {
                    TokenData::Punctuator(Punctuator::CloseParen)
                        if self.get_token(self.pos.wrapping_add(1))?.data
                            == TokenData::Punctuator(Punctuator::Arrow) =>
                    {
                        self.pos += 2;
                        let expr = self.parse()?;
                        Expr::new(ExprDef::ArrowFunctionDecl(Vec::new(), Box::new(expr)))
                    }
                    _ => {
                        let next = self.parse()?;
                        let next_tok = self.get_token(self.pos)?;
                        self.pos += 1;
                        match next_tok.data {
                            TokenData::Punctuator(Punctuator::CloseParen) => next,
                            TokenData::Punctuator(Punctuator::Comma) => {
                                // at this point it's probably gonna be an arrow function
                                // if first param captured all arguments, we should expect a close paren
                                if let ExprDef::UnaryOp(UnaryOp::Spread, _) = next.def {
                                    return Err(ParseError::Expected(
                                        vec![TokenData::Punctuator(Punctuator::CloseParen)],
                                        next_tok,
                                        "arrow function",
                                    ));
                                }

                                let mut args = vec![
                                    match next.def {
                                        ExprDef::Local(ref name) => {
                                            Expr::new(ExprDef::Local((*name).clone()))
                                        }
                                        _ => Expr::new(ExprDef::Local("".to_string())),
                                    },
                                    match self.get_token(self.pos)?.data {
                                        TokenData::Identifier(ref id) => {
                                            Expr::new(ExprDef::Local(id.clone()))
                                        }
                                        _ => Expr::new(ExprDef::Local("".to_string())),
                                    },
                                ];
                                let mut expect_ident = true;
                                loop {
                                    self.pos += 1;
                                    let curr_tk = self.get_token(self.pos)?;
                                    match curr_tk.data {
                                        TokenData::Identifier(ref id) if expect_ident => {
                                            args.push(Expr::new(ExprDef::Local(id.clone())));
                                            expect_ident = false;
                                        }
                                        TokenData::Punctuator(Punctuator::Comma) => {
                                            expect_ident = true;
                                        }
                                        TokenData::Punctuator(Punctuator::Spread) => {
                                            let ident_token = self.get_token(self.pos + 1)?;
                                            if let TokenData::Identifier(ref _id) = ident_token.data
                                            {
                                                args.push(self.parse()?);
                                                self.pos -= 1;
                                                expect_ident = false;
                                            } else {
                                                return Err(ParseError::Expected(
                                                    vec![TokenData::Identifier(
                                                        "identifier".to_string(),
                                                    )],
                                                    ident_token,
                                                    "arrow function",
                                                ));
                                            }
                                        }
                                        TokenData::Punctuator(Punctuator::CloseParen) => {
                                            self.pos += 1;
                                            break;
                                        }
                                        _ if expect_ident => {
                                            return Err(ParseError::Expected(
                                                vec![TokenData::Identifier(
                                                    "identifier".to_string(),
                                                )],
                                                curr_tk,
                                                "arrow function",
                                            ))
                                        }
                                        _ => {
                                            return Err(ParseError::Expected(
                                                vec![
                                                    TokenData::Punctuator(Punctuator::Comma),
                                                    TokenData::Punctuator(Punctuator::CloseParen),
                                                    TokenData::Punctuator(Punctuator::Spread),
                                                ],
                                                curr_tk,
                                                "arrow function",
                                            ))
                                        }
                                    }
                                }
                                self.expect(
                                    TokenData::Punctuator(Punctuator::Arrow),
                                    "arrow function",
                                )?;
                                let expr = self.parse()?;
                                Expr::new(ExprDef::ArrowFunctionDecl(args, Box::new(expr)))
                            }
                            _ => {
                                return Err(ParseError::Expected(
                                    vec![TokenData::Punctuator(Punctuator::CloseParen)],
                                    next_tok,
                                    "brackets",
                                ))
                            }
                        }
                    }
                }
            }
            TokenData::Punctuator(Punctuator::OpenBracket) => {
                let mut array: Vec<Expr> = vec![];
                let mut saw_expr_last = false;
                loop {
                    let token = self.get_token(self.pos)?;
                    match token.data {
                        TokenData::Punctuator(Punctuator::CloseBracket) => {
                            self.pos += 1;
                            break;
                        }
                        TokenData::Punctuator(Punctuator::Comma) => {
                            if !saw_expr_last {
                                // An elision indicates that a space is saved in the array
                                array.push(Expr::new(ExprDef::Const(Const::Undefined)))
                            }
                            saw_expr_last = false;
                            self.pos += 1;
                        }
                        _ if saw_expr_last => {
                            // Two expressions in a row is not allowed, they must be comma-separated
                            return Err(ParseError::Expected(
                                vec![
                                    TokenData::Punctuator(Punctuator::Comma),
                                    TokenData::Punctuator(Punctuator::CloseBracket),
                                ],
                                token,
                                "array declaration",
                            ));
                        }
                        _ => {
                            let parsed = self.parse()?;
                            saw_expr_last = true;
                            array.push(parsed);
                        }
                    }
                }
                Expr::new(ExprDef::ArrayDecl(array))
            }
            TokenData::Punctuator(Punctuator::OpenBlock)
                if self.get_token(self.pos)?.data
                    == TokenData::Punctuator(Punctuator::CloseBlock) =>
            {
                self.pos += 1;
                Expr::new(ExprDef::ObjectDecl(Box::new(BTreeMap::new())))
            }
            TokenData::Punctuator(Punctuator::OpenBlock)
                if self.get_token(self.pos.wrapping_add(1))?.data
                    == TokenData::Punctuator(Punctuator::Colon) =>
            {
                let mut map = Box::new(BTreeMap::new());
                while self.get_token(self.pos.wrapping_sub(1))?.data
                    == TokenData::Punctuator(Punctuator::Comma)
                    || map.len() == 0
                {
                    let tk = self.get_token(self.pos)?;
                    let name = match tk.data {
                        TokenData::Identifier(ref id) => id.clone(),
                        TokenData::StringLiteral(ref str) => str.clone(),
                        TokenData::Punctuator(Punctuator::CloseBlock) => {
                            self.pos += 1;
                            break;
                        }
                        _ => {
                            return Err(ParseError::Expected(
                                vec![
                                    TokenData::Identifier("identifier".to_string()),
                                    TokenData::StringLiteral("string".to_string()),
                                ],
                                tk,
                                "object declaration",
                            ))
                        }
                    };
                    self.pos += 1;
                    let value = match self.get_token(self.pos)?.data {
                        TokenData::Punctuator(Punctuator::Colon) => {
                            self.pos += 1;
                            self.parse()?
                        }
                        TokenData::Punctuator(Punctuator::OpenParen) => {
                            let args = self.parse_function_parameters()?;
                            self.pos += 1; // {
                            let expr = self.parse()?;
                            self.pos += 1;
                            Expr::new(ExprDef::FunctionDecl(None, args, Box::new(expr)))
                        }
                        _ => {
                            return Err(ParseError::Expected(
                                vec![
                                    TokenData::Punctuator(Punctuator::Colon),
                                    TokenData::Punctuator(Punctuator::OpenParen),
                                ],
                                tk,
                                "object declaration",
                            ))
                        }
                    };
                    map.insert(name, value);
                    self.pos += 1;
                }
                Expr::new(ExprDef::ObjectDecl(map))
            }
            TokenData::Punctuator(Punctuator::OpenBlock) => {
                let mut exprs = Vec::new();
                loop {
                    if self.get_token(self.pos)?.data
                        == TokenData::Punctuator(Punctuator::CloseBlock)
                    {
                        break;
                    } else {
                        exprs.push(self.parse()?);
                    }
                }
                self.pos += 1;
                Expr::new(ExprDef::Block(exprs))
            }
            // Empty Block
            TokenData::Punctuator(Punctuator::CloseBlock)
                if self.get_token(self.pos.wrapping_sub(2))?.data
                    == TokenData::Punctuator(Punctuator::OpenBlock) =>
            {
                Expr::new(ExprDef::Block(vec![]))
            }
            TokenData::Punctuator(Punctuator::Sub) => {
                Expr::new(ExprDef::UnaryOp(UnaryOp::Minus, Box::new(self.parse()?)))
            }
            TokenData::Punctuator(Punctuator::Add) => {
                Expr::new(ExprDef::UnaryOp(UnaryOp::Plus, Box::new(self.parse()?)))
            }
            TokenData::Punctuator(Punctuator::Not) => {
                Expr::new(ExprDef::UnaryOp(UnaryOp::Not, Box::new(self.parse()?)))
            }
            TokenData::Punctuator(Punctuator::Neg) => {
                Expr::new(ExprDef::UnaryOp(UnaryOp::Tilde, Box::new(self.parse()?)))
            }
            TokenData::Punctuator(Punctuator::Inc) => Expr::new(ExprDef::UnaryOp(
                UnaryOp::IncrementPre,
                Box::new(self.parse()?),
            )),
            TokenData::Punctuator(Punctuator::Dec) => Expr::new(ExprDef::UnaryOp(
                UnaryOp::DecrementPre,
                Box::new(self.parse()?),
            )),
            TokenData::Punctuator(Punctuator::Spread) => {
                Expr::new(ExprDef::UnaryOp(UnaryOp::Spread, Box::new(self.parse()?)))
            }
            _ => return Err(ParseError::Expected(Vec::new(), token.clone(), "script")),
        };
        if self.pos >= self.tokens.len() {
            Ok(expr)
        } else {
            self.parse_next(expr)
        }
    }

    fn parse_next(&mut self, expr: Expr) -> ParseResult {
        let next = self.get_token(self.pos)?;
        let mut carry_on = true;
        let mut result = expr.clone();
        match next.data {
            TokenData::Punctuator(Punctuator::Dot) => {
                self.pos += 1;
                let tk = self.get_token(self.pos)?;
                match tk.data {
                    TokenData::Identifier(ref s) => {
                        result = Expr::new(ExprDef::GetConstField(Box::new(expr), s.to_string()))
                    }
                    _ => {
                        return Err(ParseError::Expected(
                            vec![TokenData::Identifier("identifier".to_string())],
                            tk,
                            "field access",
                        ))
                    }
                }
                self.pos += 1;
            }
            TokenData::Punctuator(Punctuator::OpenParen) => {
                let mut args = Vec::new();
                let mut expect_comma_or_end = self.get_token(self.pos.wrapping_add(1))?.data
                    == TokenData::Punctuator(Punctuator::CloseParen);
                loop {
                    self.pos += 1;
                    let token = self.get_token(self.pos)?;
                    if token.data == TokenData::Punctuator(Punctuator::CloseParen)
                        && expect_comma_or_end
                    {
                        self.pos += 1;
                        break;
                    } else if token.data == TokenData::Punctuator(Punctuator::Comma)
                        && expect_comma_or_end
                    {
                        expect_comma_or_end = false;
                    } else if expect_comma_or_end {
                        return Err(ParseError::Expected(
                            vec![
                                TokenData::Punctuator(Punctuator::Comma),
                                TokenData::Punctuator(Punctuator::CloseParen),
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
                result = Expr::new(ExprDef::Call(Box::new(expr), args));
            }
            TokenData::Punctuator(Punctuator::Question) => {
                self.pos += 1;
                let if_e = self.parse()?;
                self.expect(TokenData::Punctuator(Punctuator::Colon), "if expression")?;
                let else_e = self.parse()?;
                result = Expr::new(ExprDef::If(
                    Box::new(expr),
                    Box::new(if_e),
                    Some(Box::new(else_e)),
                ));
            }
            TokenData::Punctuator(Punctuator::OpenBracket) => {
                self.pos += 1;
                let index = self.parse()?;
                self.expect(
                    TokenData::Punctuator(Punctuator::CloseBracket),
                    "array index",
                )?;
                result = Expr::new(ExprDef::GetField(Box::new(expr), Box::new(index)));
            }
            TokenData::Punctuator(Punctuator::Semicolon) | TokenData::Comment(_) => {
                self.pos += 1;
            }
            TokenData::Punctuator(Punctuator::Assign) => {
                self.pos += 1;
                let next = self.parse()?;
                result = Expr::new(ExprDef::Assign(Box::new(expr), Box::new(next)));
            }
            TokenData::Punctuator(Punctuator::AssignAdd) => {
                result = self.binop(BinOp::Assign(AssignOp::Add), expr)?
            }
            TokenData::Punctuator(Punctuator::AssignSub) => {
                result = self.binop(BinOp::Assign(AssignOp::Sub), expr)?
            }
            TokenData::Punctuator(Punctuator::AssignMul) => {
                result = self.binop(BinOp::Assign(AssignOp::Mul), expr)?
            }
            TokenData::Punctuator(Punctuator::AssignPow) => {
                result = self.binop(BinOp::Assign(AssignOp::Pow), expr)?
            }
            TokenData::Punctuator(Punctuator::AssignDiv) => {
                result = self.binop(BinOp::Assign(AssignOp::Div), expr)?
            }
            TokenData::Punctuator(Punctuator::AssignAnd) => {
                result = self.binop(BinOp::Assign(AssignOp::And), expr)?
            }
            TokenData::Punctuator(Punctuator::AssignOr) => {
                result = self.binop(BinOp::Assign(AssignOp::Or), expr)?
            }
            TokenData::Punctuator(Punctuator::AssignXor) => {
                result = self.binop(BinOp::Assign(AssignOp::Xor), expr)?
            }
            TokenData::Punctuator(Punctuator::AssignRightSh) => {
                result = self.binop(BinOp::Assign(AssignOp::Shr), expr)?
            }
            TokenData::Punctuator(Punctuator::AssignLeftSh) => {
                result = self.binop(BinOp::Assign(AssignOp::Shl), expr)?
            }
            TokenData::Punctuator(Punctuator::AssignMod) => {
                result = self.binop(BinOp::Assign(AssignOp::Mod), expr)?
            }
            TokenData::Punctuator(Punctuator::Arrow) => {
                self.pos += 1;
                let mut args = Vec::with_capacity(1);
                match result.def {
                    ExprDef::Local(ref name) => {
                        args.push(Expr::new(ExprDef::Local((*name).clone())))
                    }
                    ExprDef::UnaryOp(UnaryOp::Spread, _) => args.push(result),
                    _ => return Err(ParseError::ExpectedExpr("identifier", result)),
                }
                let next = self.parse()?;
                result = Expr::new(ExprDef::ArrowFunctionDecl(args, Box::new(next)));
            }
            TokenData::Punctuator(Punctuator::Add) => {
                result = self.binop(BinOp::Num(NumOp::Add), expr)?
            }
            TokenData::Punctuator(Punctuator::Sub) => {
                result = self.binop(BinOp::Num(NumOp::Sub), expr)?
            }
            TokenData::Punctuator(Punctuator::Mul) => {
                result = self.binop(BinOp::Num(NumOp::Mul), expr)?
            }
            TokenData::Punctuator(Punctuator::Pow) => {
                result = self.binop(BinOp::Num(NumOp::Pow), expr)?
            }
            TokenData::Punctuator(Punctuator::Div) => {
                result = self.binop(BinOp::Num(NumOp::Div), expr)?
            }
            TokenData::Punctuator(Punctuator::Mod) => {
                result = self.binop(BinOp::Num(NumOp::Mod), expr)?
            }
            TokenData::Punctuator(Punctuator::BoolAnd) => {
                result = self.binop(BinOp::Log(LogOp::And), expr)?
            }
            TokenData::Punctuator(Punctuator::BoolOr) => {
                result = self.binop(BinOp::Log(LogOp::Or), expr)?
            }
            TokenData::Punctuator(Punctuator::And) => {
                result = self.binop(BinOp::Bit(BitOp::And), expr)?
            }
            TokenData::Punctuator(Punctuator::Or) => {
                result = self.binop(BinOp::Bit(BitOp::Or), expr)?
            }
            TokenData::Punctuator(Punctuator::Xor) => {
                result = self.binop(BinOp::Bit(BitOp::Xor), expr)?
            }
            TokenData::Punctuator(Punctuator::LeftSh) => {
                result = self.binop(BinOp::Bit(BitOp::Shl), expr)?
            }
            TokenData::Punctuator(Punctuator::RightSh) => {
                result = self.binop(BinOp::Bit(BitOp::Shr), expr)?
            }
            TokenData::Punctuator(Punctuator::Eq) => {
                result = self.binop(BinOp::Comp(CompOp::Equal), expr)?
            }
            TokenData::Punctuator(Punctuator::NotEq) => {
                result = self.binop(BinOp::Comp(CompOp::NotEqual), expr)?
            }
            TokenData::Punctuator(Punctuator::StrictEq) => {
                result = self.binop(BinOp::Comp(CompOp::StrictEqual), expr)?
            }
            TokenData::Punctuator(Punctuator::StrictNotEq) => {
                result = self.binop(BinOp::Comp(CompOp::StrictNotEqual), expr)?
            }
            TokenData::Punctuator(Punctuator::LessThan) => {
                result = self.binop(BinOp::Comp(CompOp::LessThan), expr)?
            }
            TokenData::Punctuator(Punctuator::LessThanOrEq) => {
                result = self.binop(BinOp::Comp(CompOp::LessThanOrEqual), expr)?
            }
            TokenData::Punctuator(Punctuator::GreaterThan) => {
                result = self.binop(BinOp::Comp(CompOp::GreaterThan), expr)?
            }
            TokenData::Punctuator(Punctuator::GreaterThanOrEq) => {
                result = self.binop(BinOp::Comp(CompOp::GreaterThanOrEqual), expr)?
            }
            TokenData::Punctuator(Punctuator::Inc) => {
                result = Expr::new(ExprDef::UnaryOp(
                    UnaryOp::IncrementPost,
                    Box::new(self.parse()?),
                ))
            }
            TokenData::Punctuator(Punctuator::Dec) => {
                result = Expr::new(ExprDef::UnaryOp(
                    UnaryOp::DecrementPost,
                    Box::new(self.parse()?),
                ))
            }
            _ => carry_on = false,
        };
        if carry_on && self.pos < self.tokens.len() {
            self.parse_next(result)
        } else {
            Ok(result)
        }
    }

    fn binop(&mut self, op: BinOp, orig: Expr) -> Result<Expr, ParseError> {
        let (precedence, assoc) = op.get_precedence_and_assoc();
        self.pos += 1;
        let next = self.parse()?;
        Ok(match next.def {
            ExprDef::BinOp(ref op2, ref a, ref b) => {
                let other_precedence = op2.get_precedence();
                if precedence < other_precedence || (precedence == other_precedence && !assoc) {
                    Expr::new(ExprDef::BinOp(
                        op2.clone(),
                        b.clone(),
                        Box::new(Expr::new(ExprDef::BinOp(op, Box::new(orig), a.clone()))),
                    ))
                } else {
                    Expr::new(ExprDef::BinOp(op, Box::new(orig), Box::new(next.clone())))
                }
            }
            _ => Expr::new(ExprDef::BinOp(op, Box::new(orig), Box::new(next))),
        })
    }

    /// Returns an error if the next symbol is not `tk`
    fn expect(&mut self, tk: TokenData, routine: &'static str) -> Result<(), ParseError> {
        self.pos += 1;
        let curr_tk = self.get_token(self.pos.wrapping_sub(1))?;
        if curr_tk.data == tk {
            Ok(())
        } else {
            Err(ParseError::Expected(vec![tk], curr_tk, routine))
        }
    }

    /// Returns an error if the next symbol is not the punctuator `p`
    #[inline(always)]
    fn expect_punc(&mut self, p: Punctuator, routine: &'static str) -> Result<(), ParseError> {
        self.expect(TokenData::Punctuator(p), routine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::ast::{constant::Const, op::BinOp};
    use crate::syntax::{
        ast::expr::{Expr, ExprDef},
        lexer::Lexer,
    };

    fn create_bin_op(op: BinOp, exp1: Expr, exp2: Expr) -> Expr {
        Expr::new(ExprDef::BinOp(op, Box::new(exp1), Box::new(exp2)))
    }

    #[allow(clippy::result_unwrap_used)]
    fn check_parser(js: &str, expr: &[Expr]) {
        let mut lexer = Lexer::new(js);
        lexer.lex().expect("failed to lex");

        assert_eq!(
            Parser::new(lexer.tokens).parse_all().unwrap(),
            Expr::new(ExprDef::Block(expr.into()))
        );
    }

    fn check_invalid(js: &str) {
        let mut lexer = Lexer::new(js);
        lexer.lex().expect("failed to lex");

        assert!(Parser::new(lexer.tokens).parse_all().is_err());
    }

    #[test]
    fn check_string() {
        use crate::syntax::ast::constant::Const;

        // Check empty string
        check_parser(
            "\"\"",
            &[Expr::new(ExprDef::Const(Const::String(String::new())))],
        );

        // Check non-empty string
        check_parser(
            "\"hello\"",
            &[Expr::new(ExprDef::Const(Const::String(String::from(
                "hello",
            ))))],
        );
    }
    #[test]
    fn check_object_short_function() {
        // Testing short function syntax
        let mut object_properties: BTreeMap<String, Expr> = BTreeMap::new();
        object_properties.insert(
            String::from("a"),
            Expr::new(ExprDef::Const(Const::Bool(true))),
        );
        object_properties.insert(
            String::from("b"),
            Expr::new(ExprDef::FunctionDecl(
                None,
                vec![],
                Box::new(Expr::new(ExprDef::Block(vec![]))),
            )),
        );

        check_parser(
            "{
              a: true,
              b() {}
            };
            ",
            &[Expr::new(ExprDef::ObjectDecl(Box::new(object_properties)))],
        );
    }

    #[test]
    fn check_object_short_function_arguments() {
        // Testing short function syntax
        let mut object_properties: BTreeMap<String, Expr> = BTreeMap::new();
        object_properties.insert(
            String::from("a"),
            Expr::new(ExprDef::Const(Const::Bool(true))),
        );
        object_properties.insert(
            String::from("b"),
            Expr::new(ExprDef::FunctionDecl(
                None,
                vec![Expr::new(ExprDef::Local(String::from("test")))],
                Box::new(Expr::new(ExprDef::Block(vec![]))),
            )),
        );

        check_parser(
            "{
              a: true,
              b(test) {}
            };
            ",
            &[Expr::new(ExprDef::ObjectDecl(Box::new(object_properties)))],
        );
    }
    #[test]
    fn check_array() {
        use crate::syntax::ast::constant::Const;

        // Check empty array
        check_parser("[]", &[Expr::new(ExprDef::ArrayDecl(vec![]))]);

        // Check array with empty slot
        check_parser(
            "[,]",
            &[Expr::new(ExprDef::ArrayDecl(vec![Expr::new(
                ExprDef::Const(Const::Undefined),
            )]))],
        );

        // Check numeric array
        check_parser(
            "[1, 2, 3]",
            &[Expr::new(ExprDef::ArrayDecl(vec![
                Expr::new(ExprDef::Const(Const::Num(1.0))),
                Expr::new(ExprDef::Const(Const::Num(2.0))),
                Expr::new(ExprDef::Const(Const::Num(3.0))),
            ]))],
        );

        // Check numeric array with trailing comma
        check_parser(
            "[1, 2, 3,]",
            &[Expr::new(ExprDef::ArrayDecl(vec![
                Expr::new(ExprDef::Const(Const::Num(1.0))),
                Expr::new(ExprDef::Const(Const::Num(2.0))),
                Expr::new(ExprDef::Const(Const::Num(3.0))),
            ]))],
        );

        // Check numeric array with an elision
        check_parser(
            "[1, 2, , 3]",
            &[Expr::new(ExprDef::ArrayDecl(vec![
                Expr::new(ExprDef::Const(Const::Num(1.0))),
                Expr::new(ExprDef::Const(Const::Num(2.0))),
                Expr::new(ExprDef::Const(Const::Undefined)),
                Expr::new(ExprDef::Const(Const::Num(3.0))),
            ]))],
        );

        // Check numeric array with repeated elision
        check_parser(
            "[1, 2, ,, 3]",
            &[Expr::new(ExprDef::ArrayDecl(vec![
                Expr::new(ExprDef::Const(Const::Num(1.0))),
                Expr::new(ExprDef::Const(Const::Num(2.0))),
                Expr::new(ExprDef::Const(Const::Undefined)),
                Expr::new(ExprDef::Const(Const::Undefined)),
                Expr::new(ExprDef::Const(Const::Num(3.0))),
            ]))],
        );

        // Check combined array
        check_parser(
            "[1, \"a\", 2]",
            &[Expr::new(ExprDef::ArrayDecl(vec![
                Expr::new(ExprDef::Const(Const::Num(1.0))),
                Expr::new(ExprDef::Const(Const::String(String::from("a")))),
                Expr::new(ExprDef::Const(Const::Num(2.0))),
            ]))],
        );

        // Check combined array with empty string
        check_parser(
            "[1, \"\", 2]",
            &[Expr::new(ExprDef::ArrayDecl(vec![
                Expr::new(ExprDef::Const(Const::Num(1.0))),
                Expr::new(ExprDef::Const(Const::String(String::new()))),
                Expr::new(ExprDef::Const(Const::Num(2.0))),
            ]))],
        );
    }

    #[test]
    fn check_declarations() {
        use crate::syntax::ast::constant::Const;

        // Check `var` declaration
        check_parser(
            "var a = 5;",
            &[Expr::new(ExprDef::VarDecl(vec![(
                String::from("a"),
                Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
            )]))],
        );

        // Check `var` declaration with no spaces
        check_parser(
            "var a=5;",
            &[Expr::new(ExprDef::VarDecl(vec![(
                String::from("a"),
                Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
            )]))],
        );

        // Check empty `var` declaration
        check_parser(
            "var a;",
            &[Expr::new(ExprDef::VarDecl(vec![(String::from("a"), None)]))],
        );

        // Check multiple `var` declaration
        check_parser(
            "var a = 5, b, c = 6;",
            &[Expr::new(ExprDef::VarDecl(vec![
                (
                    String::from("a"),
                    Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
                ),
                (String::from("b"), None),
                (
                    String::from("c"),
                    Some(Expr::new(ExprDef::Const(Const::Num(6.0)))),
                ),
            ]))],
        );

        // Check `let` declaration
        check_parser(
            "let a = 5;",
            &[Expr::new(ExprDef::LetDecl(vec![(
                String::from("a"),
                Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
            )]))],
        );

        // Check `let` declaration with no spaces
        check_parser(
            "let a=5;",
            &[Expr::new(ExprDef::LetDecl(vec![(
                String::from("a"),
                Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
            )]))],
        );

        // Check empty `let` declaration
        check_parser(
            "let a;",
            &[Expr::new(ExprDef::LetDecl(vec![(String::from("a"), None)]))],
        );

        // Check multiple `let` declaration
        check_parser(
            "let a = 5, b, c = 6;",
            &[Expr::new(ExprDef::LetDecl(vec![
                (
                    String::from("a"),
                    Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
                ),
                (String::from("b"), None),
                (
                    String::from("c"),
                    Some(Expr::new(ExprDef::Const(Const::Num(6.0)))),
                ),
            ]))],
        );

        // Check `const` declaration
        check_parser(
            "const a = 5;",
            &[Expr::new(ExprDef::ConstDecl(vec![(
                String::from("a"),
                Expr::new(ExprDef::Const(Const::Num(5.0))),
            )]))],
        );

        // Check `const` declaration with no spaces
        check_parser(
            "const a=5;",
            &[Expr::new(ExprDef::ConstDecl(vec![(
                String::from("a"),
                Expr::new(ExprDef::Const(Const::Num(5.0))),
            )]))],
        );

        // Check empty `const` declaration
        check_invalid("const a;");

        // Check multiple `const` declaration
        check_parser(
            "const a = 5, c = 6;",
            &[Expr::new(ExprDef::ConstDecl(vec![
                (
                    String::from("a"),
                    Expr::new(ExprDef::Const(Const::Num(5.0))),
                ),
                (
                    String::from("c"),
                    Expr::new(ExprDef::Const(Const::Num(6.0))),
                ),
            ]))],
        );
    }

    #[test]
    fn check_operations() {
        use crate::syntax::ast::{constant::Const, op::BinOp};

        fn create_bin_op(op: BinOp, exp1: Expr, exp2: Expr) -> Expr {
            Expr::new(ExprDef::BinOp(op, Box::new(exp1), Box::new(exp2)))
        }

        // Check numeric operations
        check_parser(
            "a + b",
            &[create_bin_op(
                BinOp::Num(NumOp::Add),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a+1",
            &[create_bin_op(
                BinOp::Num(NumOp::Add),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Const(Const::Num(1.0))),
            )],
        );
        check_parser(
            "a - b",
            &[create_bin_op(
                BinOp::Num(NumOp::Sub),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a-1",
            &[create_bin_op(
                BinOp::Num(NumOp::Sub),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Const(Const::Num(1.0))),
            )],
        );
        check_parser(
            "a / b",
            &[create_bin_op(
                BinOp::Num(NumOp::Div),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a/2",
            &[create_bin_op(
                BinOp::Num(NumOp::Div),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Const(Const::Num(2.0))),
            )],
        );
        check_parser(
            "a * b",
            &[create_bin_op(
                BinOp::Num(NumOp::Mul),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a*2",
            &[create_bin_op(
                BinOp::Num(NumOp::Mul),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Const(Const::Num(2.0))),
            )],
        );
        check_parser(
            "a ** b",
            &[create_bin_op(
                BinOp::Num(NumOp::Pow),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a**2",
            &[create_bin_op(
                BinOp::Num(NumOp::Pow),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Const(Const::Num(2.0))),
            )],
        );
        check_parser(
            "a % b",
            &[create_bin_op(
                BinOp::Num(NumOp::Mod),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a%2",
            &[create_bin_op(
                BinOp::Num(NumOp::Mod),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Const(Const::Num(2.0))),
            )],
        );

        // Check complex numeric operations
        check_parser(
            "a + d*(b-3)+1",
            &[create_bin_op(
                BinOp::Num(NumOp::Add),
                Expr::new(ExprDef::Local(String::from("a"))),
                create_bin_op(
                    BinOp::Num(NumOp::Add),
                    // FIXME: shouldn't the last addition be on the right?
                    Expr::new(ExprDef::Const(Const::Num(1.0))),
                    create_bin_op(
                        BinOp::Num(NumOp::Mul),
                        Expr::new(ExprDef::Local(String::from("d"))),
                        create_bin_op(
                            BinOp::Num(NumOp::Sub),
                            Expr::new(ExprDef::Local(String::from("b"))),
                            Expr::new(ExprDef::Const(Const::Num(3.0))),
                        ),
                    ),
                ),
            )],
        );

        // Check bitwise operations
        check_parser(
            "a & b",
            &[create_bin_op(
                BinOp::Bit(BitOp::And),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a&b",
            &[create_bin_op(
                BinOp::Bit(BitOp::And),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );

        check_parser(
            "a | b",
            &[create_bin_op(
                BinOp::Bit(BitOp::Or),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a|b",
            &[create_bin_op(
                BinOp::Bit(BitOp::Or),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );

        check_parser(
            "a ^ b",
            &[create_bin_op(
                BinOp::Bit(BitOp::Xor),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a^b",
            &[create_bin_op(
                BinOp::Bit(BitOp::Xor),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );

        check_parser(
            "a << b",
            &[create_bin_op(
                BinOp::Bit(BitOp::Shl),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a<<b",
            &[create_bin_op(
                BinOp::Bit(BitOp::Shl),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );

        check_parser(
            "a >> b",
            &[create_bin_op(
                BinOp::Bit(BitOp::Shr),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a>>b",
            &[create_bin_op(
                BinOp::Bit(BitOp::Shr),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );

        // Check assign ops
        check_parser(
            "a += b",
            &[create_bin_op(
                BinOp::Assign(AssignOp::Add),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a -= b",
            &[create_bin_op(
                BinOp::Assign(AssignOp::Sub),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a *= b",
            &[create_bin_op(
                BinOp::Assign(AssignOp::Mul),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a **= b",
            &[create_bin_op(
                BinOp::Assign(AssignOp::Pow),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a /= b",
            &[create_bin_op(
                BinOp::Assign(AssignOp::Div),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a %= b",
            &[create_bin_op(
                BinOp::Assign(AssignOp::Mod),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a &= b",
            &[create_bin_op(
                BinOp::Assign(AssignOp::And),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a |= b",
            &[create_bin_op(
                BinOp::Assign(AssignOp::Or),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a ^= b",
            &[create_bin_op(
                BinOp::Assign(AssignOp::Xor),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a <<= b",
            &[create_bin_op(
                BinOp::Assign(AssignOp::Shl),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a >>= b",
            &[create_bin_op(
                BinOp::Assign(AssignOp::Shr),
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            )],
        );
        check_parser(
            "a %= 10 / 2",
            &[create_bin_op(
                BinOp::Assign(AssignOp::Mod),
                Expr::new(ExprDef::Local(String::from("a"))),
                create_bin_op(
                    BinOp::Num(NumOp::Div),
                    Expr::new(ExprDef::Const(Const::Num(10.0))),
                    Expr::new(ExprDef::Const(Const::Num(2.0))),
                ),
            )],
        );
    }

    #[test]
    fn check_function_declarations() {
        check_parser(
            "function foo(a) { return a; }",
            &[Expr::new(ExprDef::FunctionDecl(
                Some(String::from("foo")),
                vec![Expr::new(ExprDef::Local(String::from("a")))],
                Box::new(Expr::new(ExprDef::Block(vec![Expr::new(ExprDef::Return(
                    Some(Box::new(Expr::new(ExprDef::Local(String::from("a"))))),
                ))]))),
            ))],
        );

        check_parser(
            "function (a, ...b) {}",
            &[Expr::new(ExprDef::FunctionDecl(
                None,
                vec![
                    Expr::new(ExprDef::Local(String::from("a"))),
                    Expr::new(ExprDef::UnaryOp(
                        UnaryOp::Spread,
                        Box::new(Expr::new(ExprDef::Local(String::from("b")))),
                    )),
                ],
                Box::new(Expr::new(ExprDef::ObjectDecl(Box::new(BTreeMap::new())))),
            ))],
        );

        check_parser(
            "(...a) => {}",
            &[Expr::new(ExprDef::ArrowFunctionDecl(
                vec![Expr::new(ExprDef::UnaryOp(
                    UnaryOp::Spread,
                    Box::new(Expr::new(ExprDef::Local(String::from("a")))),
                ))],
                Box::new(Expr::new(ExprDef::ObjectDecl(Box::new(BTreeMap::new())))),
            ))],
        );

        check_parser(
            "(a, b, ...c) => {}",
            &[Expr::new(ExprDef::ArrowFunctionDecl(
                vec![
                    Expr::new(ExprDef::Local(String::from("a"))),
                    Expr::new(ExprDef::Local(String::from("b"))),
                    Expr::new(ExprDef::UnaryOp(
                        UnaryOp::Spread,
                        Box::new(Expr::new(ExprDef::Local(String::from("c")))),
                    )),
                ],
                Box::new(Expr::new(ExprDef::ObjectDecl(Box::new(BTreeMap::new())))),
            ))],
        );

        check_parser(
            "(a, b) => { return a + b; }",
            &[Expr::new(ExprDef::ArrowFunctionDecl(
                vec![
                    Expr::new(ExprDef::Local(String::from("a"))),
                    Expr::new(ExprDef::Local(String::from("b"))),
                ],
                Box::new(Expr::new(ExprDef::Block(vec![Expr::new(ExprDef::Return(
                    Some(Box::new(create_bin_op(
                        BinOp::Num(NumOp::Add),
                        Expr::new(ExprDef::Local(String::from("a"))),
                        Expr::new(ExprDef::Local(String::from("b"))),
                    ))),
                ))]))),
            ))],
        );
    }
}
