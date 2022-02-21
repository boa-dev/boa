//! Syntactical analysis, such as Abstract Syntax Tree (AST), Parsing and Lexing
// syntax module has a lot of acronyms
#![allow(clippy::upper_case_acronyms)]

pub mod ast;
pub mod lexer;
pub mod parser;

pub use lexer::Lexer;
pub use parser::Parser;
