//! The [`Declaration`] Parse Node, as defined by the [spec].
//!
//! Javascript declarations include:
//! - [Lexical][lex] declarations (`let`, `const`).
//! - [Function][fun] declarations (`function`, `async function`).
//! - [Class][class] declarations.
//!
//! See [*Difference between statements and declarations*][diff] for an explanation on why `Declaration`s
//! and `Statement`s are distinct nodes.
//!
//! [spec]: https://tc39.es/ecma262/#prod-Declaration
//! [lex]: https://tc39.es/ecma262/#prod-LexicalDeclaration
//! [fun]: https://tc39.es/ecma262/#prod-HoistableDeclaration
//! [class]: https://tc39.es/ecma262/#prod-ClassDeclaration
//! [diff]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements#difference_between_statements_and_declarations

use super::function::{AsyncFunction, AsyncGenerator, Class, Function, Generator};
use boa_interner::{Interner, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;

mod variable;

use crate::visitor::{VisitWith, Visitor, VisitorMut};
pub use variable::*;

/// The `Declaration` Parse Node.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    /// See [`Function`]
    Function(Function),

    /// See [`Generator`]
    Generator(Generator),

    /// See [`AsyncFunction`]
    AsyncFunction(AsyncFunction),

    /// See [`AsyncGenerator`]
    AsyncGenerator(AsyncGenerator),

    /// See [`Class`]
    Class(Class),

    /// See [`LexicalDeclaration`]
    Lexical(LexicalDeclaration),
}

impl ToIndentedString for Declaration {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        match self {
            Declaration::Function(f) => f.to_indented_string(interner, indentation),
            Declaration::Generator(g) => g.to_indented_string(interner, indentation),
            Declaration::AsyncFunction(af) => af.to_indented_string(interner, indentation),
            Declaration::AsyncGenerator(ag) => ag.to_indented_string(interner, indentation),
            Declaration::Class(c) => c.to_indented_string(interner, indentation),
            Declaration::Lexical(l) => {
                let mut s = l.to_interned_string(interner);
                s.push(';');
                s
            }
        }
    }
}

impl VisitWith for Declaration {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            Declaration::Function(f) => visitor.visit_function(f),
            Declaration::Generator(g) => visitor.visit_generator(g),
            Declaration::AsyncFunction(af) => visitor.visit_async_function(af),
            Declaration::AsyncGenerator(ag) => visitor.visit_async_generator(ag),
            Declaration::Class(c) => visitor.visit_class(c),
            Declaration::Lexical(ld) => visitor.visit_lexical_declaration(ld),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            Declaration::Function(f) => visitor.visit_function_mut(f),
            Declaration::Generator(g) => visitor.visit_generator_mut(g),
            Declaration::AsyncFunction(af) => visitor.visit_async_function_mut(af),
            Declaration::AsyncGenerator(ag) => visitor.visit_async_generator_mut(ag),
            Declaration::Class(c) => visitor.visit_class_mut(c),
            Declaration::Lexical(ld) => visitor.visit_lexical_declaration_mut(ld),
        }
    }
}
