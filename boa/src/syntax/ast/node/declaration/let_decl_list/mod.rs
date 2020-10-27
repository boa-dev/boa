use crate::{
    environment::lexical_environment::VariableScope,
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::node::{join_nodes, Identifier, Node},
    Context, Result, Value,
};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct LetDeclList {
    #[cfg_attr(feature = "serde", serde(flatten))]
    list: Box<[LetDecl]>,
}

impl Executable for LetDeclList {
    fn run(&self, context: &mut Context) -> Result<Value> {
        for var in self.as_ref() {
            let val = match var.init() {
                Some(v) => v.run(context)?,
                None => Value::undefined(),
            };
            context.realm_mut().environment.create_mutable_binding(
                var.name().to_owned(),
                false,
                VariableScope::Block,
            );
            context
                .realm_mut()
                .environment
                .initialize_binding(var.name(), val);
        }
        Ok(Value::undefined())
    }
}

impl<T> From<T> for LetDeclList
where
    T: Into<Box<[LetDecl]>>,
{
    fn from(list: T) -> Self {
        Self { list: list.into() }
    }
}

impl From<LetDecl> for LetDeclList {
    fn from(decl: LetDecl) -> Self {
        Self {
            list: Box::new([decl]),
        }
    }
}

impl AsRef<[LetDecl]> for LetDeclList {
    fn as_ref(&self) -> &[LetDecl] {
        &self.list
    }
}

impl fmt::Display for LetDeclList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.list.is_empty() {
            write!(f, "let ")?;
            join_nodes(f, &self.list)
        } else {
            Ok(())
        }
    }
}

impl From<LetDeclList> for Node {
    fn from(list: LetDeclList) -> Self {
        Self::LetDeclList(list)
    }
}

/// Individual constant declaration.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct LetDecl {
    name: Identifier,
    init: Option<Node>,
}

impl fmt::Display for LetDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.name, f)?;
        if let Some(ref init) = self.init {
            write!(f, " = {}", init)?;
        }
        Ok(())
    }
}

impl LetDecl {
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
