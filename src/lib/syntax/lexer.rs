use syntax::ast::token::Token;

pub struct Lexer {
    // The list fo tokens generated so far
    pub tokens: Vec<Token>,
    // The current line number in the script
    line_number: u64,
    // the current column number in the script
    column_number: u64,
    // the reader
    buffer: String,
}
