//! Declaration nodes
use crate::{
    environment::lexical_environment::VariableScope,
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::node::{join_nodes, Identifier, Node, NodeKind},
    Context, Result, Value,
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

pub mod arrow_function_decl;
pub mod async_function_decl;
pub mod async_function_expr;
pub mod function_decl;
pub mod function_expr;

pub use self::{
    arrow_function_decl::ArrowFunctionDecl, async_function_decl::AsyncFunctionDecl,
    async_function_expr::AsyncFunctionExpr, function_decl::FunctionDecl,
    function_expr::FunctionExpr,
};

#[cfg(test)]
mod tests;

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum DeclarationList {
    /// The `const` statements are block-scoped, much like variables defined using the `let`
    /// keyword.
    ///
    /// This declaration creates a constant whose scope can be either global or local to the block
    /// in which it is declared. Global constants do not become properties of the window object,
    /// unlike var variables.
    ///
    /// An initializer for a constant is required. You must specify its value in the same statement
    /// in which it's declared. (This makes sense, given that it can't be changed later.)
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/const
    /// [identifier]: https://developer.mozilla.org/en-US/docs/Glossary/identifier
    /// [expression]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Expressions
    Const(Box<[Declaration]>),

    /// The `let` statement declares a block scope local variable, optionally initializing it to a
    /// value.
    ///
    ///
    /// `let` allows you to declare variables that are limited to a scope of a block statement, or
    /// expression on which it is used, unlike the `var` keyword, which defines a variable
    /// globally, or locally to an entire function regardless of block scope.
    ///
    /// Just like const the `let` does not create properties of the window object when declared
    /// globally (in the top-most scope).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/let
    Let(Box<[Declaration]>),

    /// The `var` statement declares a variable, optionally initializing it to a value.
    ///
    /// var declarations, wherever they occur, are processed before any code is executed. This is
    /// called hoisting, and is discussed further below.
    ///
    /// The scope of a variable declared with var is its current execution context, which is either
    /// the enclosing function or, for variables declared outside any function, global. If you
    /// re-declare a JavaScript variable, it will not lose its value.
    ///
    /// Assigning a value to an undeclared variable implicitly creates it as a global variable (it
    /// becomes a property of the global object) when the assignment is executed.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-VariableStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
    Var(Box<[Declaration]>),
}

impl Executable for DeclarationList {
    fn run(&self, context: &mut Context) -> Result<Value> {
        for decl in self.as_ref() {
            use DeclarationList::*;
            let val = match decl.init() {
                None if self.is_const() => {
                    return context.throw_syntax_error("missing = in const declaration")
                }
                Some(init) => init.run(context)?,
                None => Value::undefined(),
            };

            if self.is_var() && context.has_binding(decl.name()) {
                if decl.init().is_some() {
                    context.set_mutable_binding(decl.name(), val, true)?;
                }
                continue;
            }

            match &self {
                Const(_) => context.create_immutable_binding(
                    decl.name().to_owned(),
                    false,
                    VariableScope::Block,
                )?,
                Let(_) => context.create_mutable_binding(
                    decl.name().to_owned(),
                    false,
                    VariableScope::Block,
                )?,
                Var(_) => context.create_mutable_binding(
                    decl.name().to_owned(),
                    false,
                    VariableScope::Function,
                )?,
            }

            context.initialize_binding(decl.name(), val)?;
        }

        Ok(Value::undefined())
    }
}

impl DeclarationList {
    #[allow(dead_code)]
    pub(in crate::syntax) fn is_let(&self) -> bool {
        matches!(self, Self::Let(_))
    }
    pub(in crate::syntax) fn is_const(&self) -> bool {
        matches!(self, Self::Const(_))
    }
    pub(in crate::syntax) fn is_var(&self) -> bool {
        matches!(self, Self::Var(_))
    }
}

impl AsRef<[Declaration]> for DeclarationList {
    fn as_ref(&self) -> &[Declaration] {
        use DeclarationList::*;
        match self {
            Var(list) | Const(list) | Let(list) => list,
        }
    }
}

impl fmt::Display for DeclarationList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.as_ref().is_empty() {
            use DeclarationList::*;
            match &self {
                Let(_) => write!(f, "let ")?,
                Const(_) => write!(f, "const ")?,
                Var(_) => write!(f, "var ")?,
            }
            join_nodes(f, self.as_ref())
        } else {
            Ok(())
        }
    }
}

impl From<DeclarationList> for NodeKind {
    fn from(list: DeclarationList) -> Self {
        use DeclarationList::*;
        match &list {
            Let(_) => Self::LetDeclList(list),
            Const(_) => Self::ConstDeclList(list),
            Var(_) => Self::VarDeclList(list),
        }
    }
}

impl From<Declaration> for Box<[Declaration]> {
    fn from(d: Declaration) -> Self {
        Box::new([d])
    }
}

/// Individual declaration.
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Declaration {
    name: Identifier,
    init: Option<Node>,
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.name, f)?;
        if let Some(ref init) = self.init {
            write!(f, " = {}", init)?;
        }
        Ok(())
    }
}

impl Declaration {
    /// Creates a new variable declaration.
    pub(in crate::syntax) fn new<N, I>(name: N, init: I) -> Self
    where
        N: Into<Identifier>,
        I: Into<Option<Node>>,
    {
        Self {
            name: name.into(),
            init: init.into(),
        }
    }

    /// Gets the name of the variable.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Gets the initialization node for the variable, if any.
    pub fn init(&self) -> Option<&Node> {
        self.init.as_ref()
    }
}
