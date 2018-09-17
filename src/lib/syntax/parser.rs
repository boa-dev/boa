use std::collections::btree_map::BTreeMap;
use syntax::ast::constant::Const;
use syntax::ast::expr::{Expr, ExprDef};
use syntax::ast::keyword::Keyword;
use syntax::ast::op::UnaryOp;
use syntax::ast::punc::Punctuator;
use syntax::ast::token::{Token, TokenData};

macro_rules! mk (
    ($this:expr, $def:expr) => (
        Expr::new($def, try!($this.get_token($this.pos - 1)).pos, try!($this.get_token($this.pos  - 1)).pos)
    );
    ($this:expr, $def:expr, $first:expr) => (
        Expr::new($def, $first.pos, try!($this.get_token($this.pos - 1)).pos)
    );
);

/// An error encounted during parsing an expression
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

pub type ParseResult = Result<Expr, ParseError>;

pub struct Parser {
    /// The tokens being input
    tokens: Vec<Token>,
    /// The current position within the tokens
    pos: usize,
}

impl Parser {
    /// Create a new parser, using `tokens` as input
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,
            pos: 0,
        }
    }

    /// Parse all expressions in the token array
    pub fn parse_all(&mut self) -> ParseResult {
        let mut exprs = Vec::new();
        while self.pos < self.tokens.len() {
            let result = try!(self.parse());
            exprs.push(result);
        }
        Ok(mk!(self, ExprDef::BlockExpr(exprs)))
    }

    fn get_token(&self, pos: usize) -> Result<Token, ParseError> {
        if pos < self.tokens.len() {
            Ok(self.tokens[pos].clone())
        } else {
            Err(ParseError::AbruptEnd)
        }
    }

    /// Parse a single expression
    pub fn parse(&mut self) -> ParseResult {
        if self.pos > self.tokens.len() {
            return Err(ParseError::AbruptEnd);
        }
        let token = try!(self.get_token(self.pos));
        self.pos += 1;
        let expr: Expr = match token.data {
            TokenData::Punctuator(Punctuator::Semicolon) | TokenData::Comment(_)
                if self.pos < self.tokens.len() =>
            {
                try!(self.parse())
            }
            TokenData::Punctuator(Punctuator::Semicolon) | TokenData::Comment(_) => {
                mk!(self, ExprDef::ConstExpr(Const::Undefined))
            }
            TokenData::NumericLiteral(num) => mk!(self, ExprDef::ConstExpr(Const::Num(num))),
            TNullLiteral => mk!(self, ExprDef::ConstExpr(Const::Null)),
            TokenData::StringLiteral(text) => mk!(self, ExprDef::ConstExpr(Const::String(text))),
            TokenData::BooleanLiteral(val) => mk!(self, ExprDef::ConstExpr(Const::Bool(val))),
            TokenData::Identifier(ref s) if s == "undefined" => {
                mk!(self, ExprDef::ConstExpr(Const::Undefined))
            }
            TokenData::Identifier(s) => mk!(self, ExprDef::LocalExpr(s)),
            TokenData::Keyword(keyword) => try!(self.parse_struct(keyword)),
            TokenData::Punctuator(POpenParen) => {
                match try!(self.get_token(self.pos)).data {
                    TokenData::Punctuator(Punctuator::CloseParen)
                        if try!(self.get_token(self.pos + 1)).data
                            == TokenData::Punctuator(Punctuator::Arrow) =>
                    {
                        self.pos += 2;
                        let expr = try!(self.parse());
                        mk!(
                            self,
                            ExprDef::ArrowFunctionDeclExpr(Vec::new(), Box::new(expr)),
                            token
                        )
                    }
                    _ => {
                        let next = try!(self.parse());
                        let next_tok = try!(self.get_token(self.pos));
                        self.pos += 1;
                        match next_tok.data {
                            TokenData::Punctuator(Punctuator::CloseParen) => next,
                            TokenData::Punctuator(Punctuator::Comma) => {
                                // at this point it's probably gonna be an arrow function
                                let mut args = vec![
                                    match next.def {
                                        ExprDef::LocalExpr(name) => name,
                                        _ => "".to_string(),
                                    },
                                    match try!(self.get_token(self.pos)).data {
                                        TokenData::Identifier(ref id) => id.clone(),
                                        _ => "".to_string(),
                                    },
                                ];
                                let mut expect_ident = true;
                                loop {
                                    self.pos += 1;
                                    let curr_tk = try!(self.get_token(self.pos));
                                    match curr_tk.data {
                                        TokenData::Identifier(ref id) if expect_ident => {
                                            args.push(id.clone());
                                            expect_ident = false;
                                        }
                                        TokenData::Punctuator(Punctuator::Comma) => {
                                            expect_ident = true;
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
                                                ],
                                                curr_tk,
                                                "arrow function",
                                            ))
                                        }
                                    }
                                }
                                try!(self.expect(
                                    TokenData::Punctuator(Punctuator::Arrow),
                                    "arrow function"
                                ));
                                let expr = try!(self.parse());
                                mk!(
                                    self,
                                    ExprDef::ArrowFunctionDeclExpr(args, Box::new(expr)),
                                    token
                                )
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
            TokenData::Punctuator(POpenBracket) => {
                let mut array: Vec<Expr> = Vec::new();
                let mut expect_comma_or_end = try!(self.get_token(self.pos)).data
                    == TokenData::Punctuator(Punctuator::CloseBracket);
                loop {
                    let token = try!(self.get_token(self.pos));
                    if token.data == TokenData::Punctuator(Punctuator::CloseBracket)
                        && expect_comma_or_end
                    {
                        self.pos += 1;
                        break;
                    } else if token.data == TokenData::Punctuator(Punctuator::Comma)
                        && expect_comma_or_end
                    {
                        expect_comma_or_end = false;
                    } else if token.data == TokenData::Punctuator(Punctuator::Comma)
                        && !expect_comma_or_end
                    {
                        array.push(mk!(self, ExprDef::ConstExpr(Const::Null)));
                        expect_comma_or_end = false;
                    } else if expect_comma_or_end {
                        return Err(ParseError::Expected(
                            vec![
                                TokenData::Punctuator(Punctuator::Comma),
                                TokenData::Punctuator(Punctuator::CloseBracket),
                            ],
                            token.clone(),
                            "array declaration",
                        ));
                    } else {
                        let parsed = try!(self.parse());
                        self.pos -= 1;
                        array.push(parsed);
                        expect_comma_or_end = true;
                    }
                    self.pos += 1;
                }
                mk!(self, ExprDef::ArrayDeclExpr(array), token)
            }
            TokenData::Punctuator(Punctuator::OpenBlock)
                if try!(self.get_token(self.pos)).data
                    == TokenData::Punctuator(Punctuator::CloseBlock) =>
            {
                self.pos += 1;
                mk!(
                    self,
                    ExprDef::ObjectDeclExpr(Box::new(BTreeMap::new())),
                    token
                )
            }
            TokenData::Punctuator(Punctuator::OpenBlock)
                if try!(self.get_token(self.pos + 1)).data
                    == TokenData::Punctuator(Punctuator::Colon) =>
            {
                let mut map = Box::new(BTreeMap::new());
                while try!(self.get_token(self.pos - 1)).data
                    == TokenData::Punctuator(Punctuator::Comma)
                    || map.len() == 0
                {
                    let tk = try!(self.get_token(self.pos));
                    let name = match tk.data {
                        TokenData::Identifier(ref id) => id.clone(),
                        TokenData::StringLiteral(ref str) => str.clone(),
                        _ => {
                            return Err(vec![
                                TokenData::Identifier("identifier".to_string()),
                                TokenData::StringLiteral("string".to_string()),
                                tk,
                                "object declaration",
                            ])
                        }
                    };
                    self.pos += 1;
                    try!(self.expect(
                        TokenData::Punctuator(Punctuator::Colon),
                        "object declaration"
                    ));
                    let value = try!(self.parse());
                    map.insert(name, value);
                    self.pos += 1;
                }
                mk!(self, ExprDef::ObjectDeclExpr(map), token)
            }
            TokenData::Punctuator(Punctuator::OpenBlock) => {
                let mut exprs = Vec::new();
                loop {
                    if try!(self.get_token(self.pos)).data
                        == TokenData::Punctuator(Punctuator::CloseBlock)
                    {
                        break;
                    } else {
                        exprs.push(try!(self.parse()));
                    }
                }
                self.pos += 1;
                mk!(self, ExprDef::BlockExpr(exprs), token)
            }
            TokenData::Punctuator(PSub) => mk!(
                self,
                ExprDef::UnaryOpExpr(UnaryOp::Minus, Box::new(try!(self.parse())))
            ),
            TokenData::Punctuator(PAdd) => mk!(
                self,
                ExprDef::UnaryOpExpr(UnaryOp::Plus, Box::new(try!(self.parse())))
            ),
            TokenData::Punctuator(PNot) => mk!(
                self,
                ExprDef::UnaryOpExpr(UnaryOp::Not, Box::new(try!(self.parse())))
            ),
            TokenData::Punctuator(PInc) => mk!(
                self,
                ExprDef::UnaryOpExpr(UnaryOp::IncrementPre, Box::new(try!(self.parse())))
            ),
            TokenData::Punctuator(PDec) => mk!(
                self,
                ExprDef::UnaryOpExpr(UnaryOp::DecrementPre, Box::new(try!(self.parse())))
            ),
            _ => return Err(ParseError::Expected(Vec::new(), token.clone(), "script")),
        };
        if self.pos >= self.tokens.len() {
            Ok(expr)
        } else {
            self.parse_next(expr)
        }
    }
}
