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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ConstDeclList {
    #[cfg_attr(feature = "serde", serde(flatten))]
    list: Box<[ConstDecl]>,
}

impl Executable for ConstDeclList {
    fn run(&self, context: &mut Context) -> Result<Value> {
        for decl in self.as_ref() {
            let val = if let Some(init) = decl.init() {
                init.run(context)?
            } else {
                return context.throw_syntax_error("missing = in const declaration");
            };

            context.realm_mut().environment.create_immutable_binding(
                decl.name().to_owned(),
                false,
                VariableScope::Block,
            );

            context
                .realm_mut()
                .environment
                .initialize_binding(decl.name(), val);
        }
        Ok(Value::undefined())
    }
}

impl<T> From<T> for ConstDeclList
where
    T: Into<Box<[ConstDecl]>>,
{
    fn from(list: T) -> Self {
        Self { list: list.into() }
    }
}

impl From<ConstDecl> for ConstDeclList {
    fn from(decl: ConstDecl) -> Self {
        Self {
            list: Box::new([decl]),
        }
    }
}

impl AsRef<[ConstDecl]> for ConstDeclList {
    fn as_ref(&self) -> &[ConstDecl] {
        &self.list
    }
}

impl fmt::Display for ConstDeclList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.list.is_empty() {
            write!(f, "const ")?;
            join_nodes(f, &self.list)
        } else {
            Ok(())
        }
    }
}

impl From<ConstDeclList> for Node {
    fn from(list: ConstDeclList) -> Self {
        Self::ConstDeclList(list)
    }
}

/// Individual constant declaration.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ConstDecl {
    name: Identifier,
    init: Option<Node>,
}

impl fmt::Display for ConstDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.name, f)?;
        if let Some(ref init) = self.init {
            write!(f, " = {}", init)?;
        }
        Ok(())
    }
}

impl ConstDecl {
    /// Creates a new variable declaration.
    pub(in crate::syntax) fn new<N, I>(name: N, init: Option<I>) -> Self
    where
        N: Into<Identifier>,
        I: Into<Node>,
    {
        Self {
            name: name.into(),
            init: init.map(|n| n.into()),
        }
    }

    /// Gets the name of the variable.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Gets the initialization node for the variable, if any.
    pub fn init(&self) -> &Option<Node> {
        &self.init
    }
}
