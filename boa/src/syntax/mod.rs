//! Syntactical analysis, such as AST, Parsing and Lexing

/// The Javascript Abstract Syntax Tree
pub mod ast;
/// Lexical analysis (tokenizing/lexing).
pub mod lexer;
// Parses a sequence of tokens into expressions
pub mod parser;
