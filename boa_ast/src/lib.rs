//! Boa's **`boa_ast`** crate implements an ECMAScript abstract syntax tree.
//!
//! # Crate Overview
//! **boa_ast** contains representations of [**Parse Nodes**][grammar] as defined by the ECMAScript
//! spec. Some `Parse Node`s are not represented by Boa's AST, because a lot of grammar productions
//! are only used to throw [**Early Errors**][early], and don't influence the evaluation of the AST
//! itself.
//!
//! Boa's AST is mainly split in three main components: [`Declaration`]s, [`Expression`]s and
//! [`Statement`]s, with [`StatementList`] being the primordial Parse Node that combines
//! all of them to create a proper AST.
//!
//! # About Boa
//! Boa is an open-source, experimental ECMAScript Engine written in Rust for lexing, parsing and executing ECMAScript/JavaScript. Currently, Boa
//! supports some of the [language][boa-conformance]. More information can be viewed at [Boa's website][boa-web].
//!
//! Try out the most recent release with Boa's live demo [playground][boa-playground].  
//!
//! # Boa Crates
//!  - **boa_ast** - Boa's ECMAScript Abstract Syntax Tree.
//!  - **boa_engine** - Boa's implementation of ECMAScript builtin objects and execution.
//!  - **boa_gc** - Boa's garbage collector.
//!  - **boa_interner** - Boa's string interner.
//!  - **boa_parser** - Boa's lexer and parser.
//!  - **boa_profiler** - Boa's code profiler.
//!  - **boa_unicode** - Boa's Unicode identifier.
//!  - **boa_icu_provider** - Boa's ICU4X data provider.
//!
//! [grammar]: https://tc39.es/ecma262/#sec-syntactic-grammar
//! [early]: https://tc39.es/ecma262/#sec-static-semantic-rules
//! [boa-conformance]: https://boa-dev.github.io/boa/test262/
//! [boa-web]: https://boa-dev.github.io/
//! [boa-playground]: https://boa-dev.github.io/boa/playground/

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(missing_docs, clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,

    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,

    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,

    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::option_if_let_else,
    clippy::use_self
)]

mod position;
mod punctuator;
mod statement_list;

pub mod declaration;
pub mod expression;
pub mod function;
pub mod keyword;
pub mod operations;
pub mod pattern;
pub mod property;
pub mod statement;
pub mod visitor;

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

/// Utility trait that adds a `UTF-16` escaped representation to every [`[u16]`][slice].
trait ToStringEscaped {
    /// Decodes `self` as an `UTF-16` encoded string, escaping any unpaired surrogates by its
    /// codepoint value.
    fn to_string_escaped(&self) -> String;
}

impl ToStringEscaped for [u16] {
    fn to_string_escaped(&self) -> String {
        char::decode_utf16(self.iter().copied())
            .map(|r| match r {
                Ok(c) => String::from(c),
                Err(e) => format!("\\u{:04X}", e.unpaired_surrogate()),
            })
            .collect()
    }
}
