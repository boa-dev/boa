use std::collections::btree_map::BTreeMap;
use syntax::ast::constant::Const;
use syntax::ast::expr::{Expr, ExprDef};
use syntax::ast::keyword::Keyword;
use syntax::ast::op::{BinOp, BitOp, CompOp, LogOp, NumOp, Operator, UnaryOp};
use syntax::ast::punc::Punctuator;
use syntax::ast::token::{Token, TokenData};

macro_rules! mk (
    ($this:expr, $def:expr) => (
        Expr::new($def, try!($this.get_token($this.pos)).pos, try!($this.get_token($this.pos)).pos)
    );
    ($this:expr, $def:expr, $first:expr) => (
        Expr::new($def, $first.pos, try!($this.get_token($this.pos)).pos)
    );
);

/// ParseError is an enum which represents errors encounted during parsing an expression
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
        println!("{}", exprs[0]);
        Ok(mk!(self, ExprDef::BlockExpr(exprs)))
    }

    fn get_token(&self, pos: usize) -> Result<Token, ParseError> {
        if pos < self.tokens.len() {
            Ok(self.tokens[pos].clone())
        } else {
            Err(ParseError::AbruptEnd)
        }
    }

    fn parse_struct(&mut self, keyword: Keyword) -> ParseResult {
        match keyword {
            Keyword::Throw => {
                let thrown = try!(self.parse());
                Ok(mk!(self, ExprDef::ThrowExpr(Box::new(thrown))))
            }
            Keyword::Var => {
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
                                "var statement",
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
                Ok(mk!(self, ExprDef::VarDeclExpr(vars)))
            }
            Keyword::Return => Ok(mk!(
                self,
                ExprDef::ReturnExpr(Some(Box::new(self.parse()?.clone())))
            )),
            Keyword::New => {
                let call = self.parse()?;
                match call.def {
                    ExprDef::CallExpr(ref func, ref args) => Ok(mk!(
                        self,
                        ExprDef::ConstructExpr(func.clone(), args.clone())
                    )),
                    _ => Err(ParseError::ExpectedExpr("constructor", call)),
                }
            }
            Keyword::TypeOf => Ok(mk!(self, ExprDef::TypeOfExpr(Box::new(self.parse()?)))),
            Keyword::If => {
                self.expect_punc(Punctuator::OpenParen, "if block")?;
                let cond = self.parse()?;
                self.expect_punc(Punctuator::CloseParen, "if block")?;
                let expr = self.parse()?;
                let next = self.get_token(self.pos + 1);
                Ok(mk!(
                    self,
                    ExprDef::IfExpr(
                        Box::new(cond),
                        Box::new(expr),
                        if next.is_ok() && next.unwrap().data == TokenData::Keyword(Keyword::Else) {
                            self.pos += 2;
                            Some(Box::new(self.parse()?))
                        } else {
                            None
                        }
                    )
                ))
            }
            Keyword::While => {
                self.expect_punc(Punctuator::OpenParen, "while condition")?;
                let cond = self.parse()?;
                self.expect_punc(Punctuator::CloseParen, "while condition")?;
                let expr = self.parse()?;
                Ok(mk!(
                    self,
                    ExprDef::WhileLoopExpr(Box::new(cond), Box::new(expr))
                ))
            }
            Keyword::Switch => {
                try!(self.expect_punc(Punctuator::OpenParen, "switch value"));
                let value = self.parse();
                try!(self.expect_punc(Punctuator::CloseParen, "switch value"));
                try!(self.expect_punc(Punctuator::OpenBlock, "switch block"));
                let mut cases = Vec::new();
                let mut default = None;
                while self.pos + 1 < self.tokens.len() {
                    let tok = self.get_token(self.pos)?;
                    self.pos += 1;
                    match tok.data {
                        TokenData::Keyword(Keyword::Case) => {
                            let cond = self.parse();
                            let mut block = Vec::new();
                            try!(self.expect_punc(Punctuator::Colon, "switch case"));
                            loop {
                                match try!(self.get_token(self.pos)).data {
                                    TokenData::Keyword(Keyword::Case)
                                    | TokenData::Keyword(Keyword::Default) => break,
                                    TokenData::Punctuator(Punctuator::CloseBlock) => break,
                                    _ => block.push(try!(self.parse())),
                                }
                            }
                            cases.push((cond.unwrap(), block));
                        }
                        TokenData::Keyword(Keyword::Default) => {
                            let mut block = Vec::new();
                            try!(self.expect_punc(Punctuator::Colon, "default switch case"));
                            loop {
                                match try!(self.get_token(self.pos)).data {
                                    TokenData::Keyword(Keyword::Case)
                                    | TokenData::Keyword(Keyword::Default) => break,
                                    TokenData::Punctuator(Punctuator::CloseBlock) => break,
                                    _ => block.push(try!(self.parse())),
                                }
                            }
                            default = Some(mk!(self, ExprDef::BlockExpr(block)));
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
                try!(self.expect_punc(Punctuator::CloseBlock, "switch block"));
                Ok(mk!(
                    self,
                    ExprDef::SwitchExpr(
                        Box::new(value.unwrap()),
                        cases,
                        match default {
                            Some(v) => Some(Box::new(v)),
                            None => None,
                        }
                    )
                ))
            }
            Keyword::Function => {
                // function [identifier] () { etc }
                let tk = try!(self.get_token(self.pos));
                let name = match tk.data {
                    TokenData::Identifier(ref name) => {
                        self.pos += 1;
                        Some(name.clone())
                    }
                    TokenData::Punctuator(Punctuator::OpenParen) => None,
                    _ => {
                        return Err(ParseError::Expected(
                            vec![TokenData::Identifier("identifier".to_string())],
                            tk.clone(),
                            "function name",
                        ))
                    }
                };
                // Now we have the function identifier we should have an open paren for arguments ( )
                self.expect_punc(Punctuator::OpenParen, "function")?;
                let mut args: Vec<String> = Vec::new();
                let mut tk = self.get_token(self.pos)?;
                while tk.data != TokenData::Punctuator(Punctuator::CloseParen) {
                    match tk.data {
                        TokenData::Identifier(ref id) => args.push(id.clone()),
                        _ => {
                            return Err(ParseError::Expected(
                                vec![TokenData::Identifier("identifier".to_string())],
                                tk.clone(),
                                "function arguments",
                            ))
                        }
                    }
                    self.pos += 1;
                    if try!(self.get_token(self.pos)).data
                        == TokenData::Punctuator(Punctuator::Comma)
                    {
                        self.pos += 1;
                    }
                    tk = self.get_token(self.pos)?;
                }
                self.pos += 1;
                let block = self.parse()?;
                Ok(mk!(
                    self,
                    ExprDef::FunctionDeclExpr(name, args, Box::new(block))
                ))
            }
            _ => Err(ParseError::UnexpectedKeyword(keyword)),
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
            TokenData::NullLiteral => mk!(self, ExprDef::ConstExpr(Const::Null)),
            TokenData::StringLiteral(text) => mk!(self, ExprDef::ConstExpr(Const::String(text))),
            TokenData::BooleanLiteral(val) => mk!(self, ExprDef::ConstExpr(Const::Bool(val))),
            TokenData::Identifier(ref s) if s == "undefined" => {
                mk!(self, ExprDef::ConstExpr(Const::Undefined))
            }
            TokenData::Identifier(s) => mk!(self, ExprDef::LocalExpr(s)),
            TokenData::Keyword(keyword) => try!(self.parse_struct(keyword)),
            TokenData::Punctuator(Punctuator::OpenParen) => {
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
                                                curr_tk.clone(),
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
            TokenData::Punctuator(Punctuator::OpenBracket) => {
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
            TokenData::Punctuator(Punctuator::Sub) => mk!(
                self,
                ExprDef::UnaryOpExpr(UnaryOp::Minus, Box::new(try!(self.parse())))
            ),
            TokenData::Punctuator(Punctuator::Add) => mk!(
                self,
                ExprDef::UnaryOpExpr(UnaryOp::Plus, Box::new(try!(self.parse())))
            ),
            TokenData::Punctuator(Punctuator::Not) => mk!(
                self,
                ExprDef::UnaryOpExpr(UnaryOp::Not, Box::new(try!(self.parse())))
            ),
            TokenData::Punctuator(Punctuator::Inc) => mk!(
                self,
                ExprDef::UnaryOpExpr(UnaryOp::IncrementPre, Box::new(try!(self.parse())))
            ),
            TokenData::Punctuator(Punctuator::Dec) => mk!(
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

    fn parse_next(&mut self, expr: Expr) -> ParseResult {
        let next = self.get_token(self.pos)?;
        let mut carry_on = true;
        let mut result = expr.clone();
        match next.data {
            TokenData::Punctuator(Punctuator::Dot) => {
                self.pos += 1;
                let tk = try!(self.get_token(self.pos));
                match tk.data {
                    TokenData::Identifier(ref s) => {
                        result = mk!(
                            self,
                            ExprDef::GetConstFieldExpr(Box::new(expr), s.to_string())
                        )
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
                let mut expect_comma_or_end = try!(self.get_token(self.pos + 1)).data
                    == TokenData::Punctuator(Punctuator::CloseParen);
                loop {
                    self.pos += 1;
                    let token = try!(self.get_token(self.pos));
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
                        let parsed = try!(self.parse());
                        self.pos -= 1;
                        args.push(parsed);
                        expect_comma_or_end = true;
                    }
                }
                result = mk!(self, ExprDef::CallExpr(Box::new(expr), args));
            }
            TokenData::Punctuator(Punctuator::Question) => {
                self.pos += 1;
                let if_e = try!(self.parse());
                try!(self.expect(TokenData::Punctuator(Punctuator::Colon), "if expression"));
                let else_e = try!(self.parse());
                result = mk!(
                    self,
                    ExprDef::IfExpr(Box::new(expr), Box::new(if_e), Some(Box::new(else_e)))
                );
            }
            TokenData::Punctuator(Punctuator::OpenBracket) => {
                self.pos += 1;
                let index = try!(self.parse());
                try!(self.expect(
                    TokenData::Punctuator(Punctuator::CloseBracket),
                    "array index"
                ));
                result = mk!(self, ExprDef::GetFieldExpr(Box::new(expr), Box::new(index)));
            }
            TokenData::Punctuator(Punctuator::Semicolon) | TokenData::Comment(_) => {
                self.pos += 1;
            }
            TokenData::Punctuator(Punctuator::Assign) => {
                self.pos += 1;
                let next = try!(self.parse());
                result = mk!(self, ExprDef::AssignExpr(Box::new(expr), Box::new(next)));
            }
            TokenData::Punctuator(Punctuator::Arrow) => {
                self.pos += 1;
                let mut args = Vec::with_capacity(1);
                match result.def {
                    ExprDef::LocalExpr(name) => args.push(name),
                    _ => return Err(ParseError::ExpectedExpr("identifier", result)),
                }
                let next = try!(self.parse());
                result = mk!(self, ExprDef::ArrowFunctionDeclExpr(args, Box::new(next)));
            }
            TokenData::Punctuator(Punctuator::Add) => {
                result = try!(self.binop(BinOp::Num(NumOp::Add), expr))
            }
            TokenData::Punctuator(Punctuator::Sub) => {
                result = try!(self.binop(BinOp::Num(NumOp::Sub), expr))
            }
            TokenData::Punctuator(Punctuator::Mul) => {
                result = try!(self.binop(BinOp::Num(NumOp::Mul), expr))
            }
            TokenData::Punctuator(Punctuator::Div) => {
                result = try!(self.binop(BinOp::Num(NumOp::Div), expr))
            }
            TokenData::Punctuator(Punctuator::Mod) => {
                result = try!(self.binop(BinOp::Num(NumOp::Mod), expr))
            }
            TokenData::Punctuator(Punctuator::BoolAnd) => {
                result = try!(self.binop(BinOp::Log(LogOp::And), expr))
            }
            TokenData::Punctuator(Punctuator::BoolOr) => {
                result = try!(self.binop(BinOp::Log(LogOp::Or), expr))
            }
            TokenData::Punctuator(Punctuator::And) => {
                result = try!(self.binop(BinOp::Bit(BitOp::And), expr))
            }
            TokenData::Punctuator(Punctuator::Or) => {
                result = try!(self.binop(BinOp::Bit(BitOp::Or), expr))
            }
            TokenData::Punctuator(Punctuator::Xor) => {
                result = try!(self.binop(BinOp::Bit(BitOp::Xor), expr))
            }
            TokenData::Punctuator(Punctuator::LeftSh) => {
                result = try!(self.binop(BinOp::Bit(BitOp::Shl), expr))
            }
            TokenData::Punctuator(Punctuator::RightSh) => {
                result = try!(self.binop(BinOp::Bit(BitOp::Shr), expr))
            }
            TokenData::Punctuator(Punctuator::Eq) => {
                result = try!(self.binop(BinOp::Comp(CompOp::Equal), expr))
            }
            TokenData::Punctuator(Punctuator::NotEq) => {
                result = try!(self.binop(BinOp::Comp(CompOp::NotEqual), expr))
            }
            TokenData::Punctuator(Punctuator::StrictEq) => {
                result = try!(self.binop(BinOp::Comp(CompOp::StrictEqual), expr))
            }
            TokenData::Punctuator(Punctuator::StrictNotEq) => {
                result = try!(self.binop(BinOp::Comp(CompOp::StrictNotEqual), expr))
            }
            TokenData::Punctuator(Punctuator::LessThan) => {
                result = try!(self.binop(BinOp::Comp(CompOp::LessThan), expr))
            }
            TokenData::Punctuator(Punctuator::LessThanOrEq) => {
                result = try!(self.binop(BinOp::Comp(CompOp::LessThanOrEqual), expr))
            }
            TokenData::Punctuator(Punctuator::GreaterThan) => {
                result = try!(self.binop(BinOp::Comp(CompOp::GreaterThan), expr))
            }
            TokenData::Punctuator(Punctuator::GreaterThanOrEq) => {
                result = try!(self.binop(BinOp::Comp(CompOp::GreaterThanOrEqual), expr))
            }
            TokenData::Punctuator(Punctuator::Inc) => {
                result = mk!(
                    self,
                    ExprDef::UnaryOpExpr(UnaryOp::IncrementPost, Box::new(try!(self.parse())))
                )
            }
            TokenData::Punctuator(Punctuator::Dec) => {
                result = mk!(
                    self,
                    ExprDef::UnaryOpExpr(UnaryOp::DecrementPost, Box::new(try!(self.parse())))
                )
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
        let next = try!(self.parse());
        Ok(match next.def {
            ExprDef::BinOpExpr(ref op2, ref a, ref b) => {
                let other_precedence = op2.get_precedence();
                if precedence < other_precedence || (precedence == other_precedence && !assoc) {
                    mk!(
                        self,
                        ExprDef::BinOpExpr(
                            op2.clone(),
                            b.clone(),
                            Box::new(mk!(
                                self,
                                ExprDef::BinOpExpr(op.clone(), Box::new(orig), a.clone())
                            ))
                        )
                    )
                } else {
                    mk!(
                        self,
                        ExprDef::BinOpExpr(op, Box::new(orig), Box::new(next.clone()))
                    )
                }
            }
            _ => mk!(self, ExprDef::BinOpExpr(op, Box::new(orig), Box::new(next))),
        })
    }

    /// Returns an error if the next symbol is not `tk`
    fn expect(&mut self, tk: TokenData, routine: &'static str) -> Result<(), ParseError> {
        self.pos += 1;
        let curr_tk = self.get_token(self.pos - 1)?;
        if curr_tk.data != tk {
            Err(ParseError::Expected(vec![tk], curr_tk, routine))
        } else {
            Ok(())
        }
    }

    /// Returns an error if the next symbol is not the punctuator `p`
    #[inline(always)]
    fn expect_punc(&mut self, p: Punctuator, routine: &'static str) -> Result<(), ParseError> {
        self.expect(TokenData::Punctuator(p), routine)
    }
}
