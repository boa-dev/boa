use crate::{
    builtins::function::FunctionFlags,
    environment::lexical_environment::VariableScope,
    exec::Executable,
    gc::{Finalize, Trace},
    object::{GcObject, Object, PROTOTYPE},
    syntax::ast::node::{FunctionDecl, Node},
    BoaProfiler, Context, Result, Value,
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// The `class` declaration defines a class with the specified methods, fields, and optional constructor.
///
/// Classes can be used to create objects, which can also be created through literals (using `{}`).
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ClassDecl {
    name: Box<str>,
    constructor: Option<FunctionDecl>,

    methods: Box<[FunctionDecl]>,
    static_methods: Box<[FunctionDecl]>,
}

impl ClassDecl {
    /// Creates a new class declaration.
    pub(in crate::syntax) fn new<N, C, M, SM>(
        name: N,
        constructor: C,
        methods: M,
        static_methods: SM,
    ) -> Self
    where
        N: Into<Box<str>>,
        C: Into<Option<FunctionDecl>>,
        M: Into<Box<[FunctionDecl]>>,
        SM: Into<Box<[FunctionDecl]>>,
    {
        Self {
            name: name.into(),
            constructor: constructor.into(),
            methods: methods.into(),
            static_methods: static_methods.into(),
        }
    }

    /// Gets the name of the class.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the list of functions defined on this class. Does not include the constructor.
    pub fn methods(&self) -> &[FunctionDecl] {
        &self.methods
    }

    /// Gets the list of statuc functions defined on this class. Does not include the constructor.
    pub fn static_methods(&self) -> &[FunctionDecl] {
        &self.static_methods
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        // TODO: Implement display for class
        write!(f, "class {{}}")
    }
}

impl Executable for ClassDecl {
    fn run(&self, context: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("ClassDecl", "exec");
        let constructor = match &self.constructor {
            Some(c) => context.create_function(
                c.parameters().to_vec(),
                c.body().to_vec(),
                FunctionFlags::CALLABLE | FunctionFlags::CONSTRUCTABLE,
            )?,
            None => context.create_function(
                vec![],
                vec![],
                FunctionFlags::CALLABLE | FunctionFlags::CONSTRUCTABLE,
            )?,
        };

        // Set the name and assign it in the current environment
        constructor.set_field("name", self.name(), context)?;

        let proto = Value::Object(GcObject::new(Object::new()));
        for method in self.methods() {
            proto.set_field(method.name(), method, context)?;
        }
        constructor.set_field(PROTOTYPE, proto, context)?;

        if context.has_binding(self.name()) {
            context.set_mutable_binding(self.name(), constructor, true)?;
        } else {
            context.create_mutable_binding(
                self.name().to_owned(),
                false,
                VariableScope::Function,
            )?;

            context.initialize_binding(self.name(), constructor)?;
        }
        Ok(Value::undefined())
    }
}

impl From<ClassDecl> for Node {
    fn from(decl: ClassDecl) -> Self {
        Self::ClassDecl(decl)
    }
}

impl fmt::Display for ClassDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}
