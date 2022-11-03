//! Syntactical analysis, such as Parsing and Lexing.
// syntax module has a lot of acronyms

pub mod lexer;
pub mod parser;

pub use lexer::Lexer;
pub use parser::Parser;
