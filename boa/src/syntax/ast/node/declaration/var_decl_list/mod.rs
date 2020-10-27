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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct VarDeclList {
    #[cfg_attr(feature = "serde", serde(flatten))]
    vars: Box<[VarDecl]>,
}

impl Executable for VarDeclList {
    fn run(&self, context: &mut Context) -> Result<Value> {
        for var in self.as_ref() {
            let val = match var.init() {
                Some(v) => v.run(context)?,
                None => Value::undefined(),
            };
            let environment = &mut context.realm_mut().environment;

            if environment.has_binding(var.name()) {
                if var.init().is_some() {
                    environment.set_mutable_binding(var.name(), val, true);
                }
            } else {
                environment.create_mutable_binding(
                    var.name().to_owned(),
                    false,
                    VariableScope::Function,
                );
                environment.initialize_binding(var.name(), val);
            }
        }
        Ok(Value::undefined())
    }
}

impl<T> From<T> for VarDeclList
where
    T: Into<Box<[VarDecl]>>,
{
    fn from(list: T) -> Self {
        Self { vars: list.into() }
    }
}

impl From<VarDecl> for VarDeclList {
    fn from(decl: VarDecl) -> Self {
        Self {
            vars: Box::new([decl]),
        }
    }
}

impl AsRef<[VarDecl]> for VarDeclList {
    fn as_ref(&self) -> &[VarDecl] {
        &self.vars
    }
}

impl fmt::Display for VarDeclList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.vars.is_empty() {
            write!(f, "var ")?;
            join_nodes(f, &self.vars)
        } else {
            Ok(())
        }
    }
}

impl From<VarDeclList> for Node {
    fn from(list: VarDeclList) -> Self {
        Self::VarDeclList(list)
    }
}

/// Individual variable declaration.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct VarDecl {
    name: Identifier,
    init: Option<Node>,
}

impl fmt::Display for VarDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.name, f)?;
        if let Some(ref init) = self.init {
            write!(f, " = {}", init)?;
        }
        Ok(())
    }
}

impl VarDecl {
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
