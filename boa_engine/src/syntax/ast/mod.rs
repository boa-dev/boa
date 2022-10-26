//! The Javascript Abstract Syntax Tree.
//!
//! This module contains representations of [**Parse Nodes**][grammar] as defined by the ECMAScript spec.
//! Some `Parse Node`s are not represented by Boa's AST, because a lot of grammar productions are
//! only used to throw [**Early Errors**][early], and don't influence the evaluation of the AST itself.
//!
//! Boa's AST is mainly split in three main components: [`Declaration`]s, [`Expression`]s and
//! [`Statement`]s, with [`StatementList`] being the primordial Parse Node that combines
//! all of them to create a proper AST.
//!
//! [grammar]: https://tc39.es/ecma262/#sec-syntactic-grammar
//! [early]: https://tc39.es/ecma262/#sec-static-semantic-rules

#![deny(missing_docs)]

mod position;
mod punctuator;
mod statement_list;

pub mod declaration;
pub mod expression;
pub mod function;
pub mod keyword;
pub mod pattern;
pub mod property;
pub mod statement;

use boa_interner::{Interner, ToIndentedString, ToInternedString};

pub use self::{
    declaration::Declaration,
    expression::Expression,
    keyword::Keyword,
    position::{Position, Span},
    punctuator::Punctuator,
    statement::Statement,
    statement_list::{StatementList, StatementListItem},
};

/// Represents all the possible symbols searched for by the [`Contains`][contains] operation.
///
/// [contains]: https://tc39.es/ecma262/#sec-syntax-directed-operations-contains
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ContainsSymbol {
    SuperProperty,
    SuperCall,
    YieldExpression,
    AwaitExpression,
    NewTarget,
    ClassBody,
    ClassHeritage,
    This,
    MethodDefinition,
}

/// Utility to join multiple Nodes into a single string.
fn join_nodes<N>(interner: &Interner, nodes: &[N]) -> String
where
    N: ToInternedString,
{
    let mut first = true;
    let mut buf = String::new();
    for e in nodes {
        if first {
            first = false;
        } else {
            buf.push_str(", ");
        }
        buf.push_str(&e.to_interned_string(interner));
    }
    buf
}

/// Displays the body of a block or statement list.
///
/// This includes the curly braces at the start and end. This will not indent the first brace,
/// but will indent the last brace.
fn block_to_string(body: &StatementList, interner: &Interner, indentation: usize) -> String {
    if body.statements().is_empty() {
        "{}".to_owned()
    } else {
        format!(
            "{{\n{}{}}}",
            body.to_indented_string(interner, indentation + 1),
            "    ".repeat(indentation)
        )
    }
}

/// This parses the given source code, and then makes sure that
/// the resulting `StatementList` is formatted in the same manner
/// as the source code. This is expected to have a preceding
/// newline.
///
/// This is a utility function for tests. It was made in case people
/// are using different indents in their source files. This fixes
/// any strings which may have been changed in a different indent
/// level.
#[cfg(test)]
fn test_formatting(source: &'static str) {
    use crate::{syntax::Parser, Context};

    // Remove preceding newline.
    let source = &source[1..];

    // Find out how much the code is indented
    let first_line = &source[..source.find('\n').unwrap()];
    let trimmed_first_line = first_line.trim();
    let characters_to_remove = first_line.len() - trimmed_first_line.len();

    let scenario = source
        .lines()
        .map(|l| &l[characters_to_remove..]) // Remove preceding whitespace from each line
        .collect::<Vec<&'static str>>()
        .join("\n");
    let mut context = Context::default();
    let result = Parser::new(scenario.as_bytes())
        .parse_all(&mut context)
        .expect("parsing failed")
        .to_interned_string(context.interner());
    if scenario != result {
        eprint!("========= Expected:\n{scenario}");
        eprint!("========= Got:\n{result}");
        // Might be helpful to find differing whitespace
        eprintln!("========= Expected: {scenario:?}");
        eprintln!("========= Got:      {result:?}");
        panic!("parsing test did not give the correct result (see above)");
    }
}
