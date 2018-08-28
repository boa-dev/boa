use syntax::ast::expr::Expr;
use syntax::ast::keyword::Keyword;
use syntax::ast::token::{Token, TokenData};

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
    pos: u64,
}

impl Parser {
    /// Create a new parser, using `tokens` as input
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,
            pos: 0,
        }
    }
}
