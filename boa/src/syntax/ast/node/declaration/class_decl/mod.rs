use crate::{
    builtins::function::FunctionFlags,
    environment::lexical_environment::VariableScope,
    exec::Executable,
    gc::{Finalize, Trace},
    object::{GcObject, Object, PROTOTYPE},
    syntax::{
        ast::node::{FunctionDecl, Node},
        parser::class::ClassField,
    },
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

    fields: Box<[ClassField]>,
    static_fields: Box<[ClassField]>,
}

impl ClassDecl {
    /// Creates a new class declaration.
    pub(in crate::syntax) fn new<N, C, F, SF>(
        name: N,
        constructor: C,
        fields: F,
        static_fields: SF,
    ) -> Self
    where
        N: Into<Box<str>>,
        C: Into<Option<FunctionDecl>>,
        F: Into<Box<[ClassField]>>,
        SF: Into<Box<[ClassField]>>,
    {
        Self {
            name: name.into(),
            constructor: constructor.into(),
            fields: fields.into(),
            static_fields: static_fields.into(),
        }
    }

    /// Gets the name of the class.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the constructor of this class.
    pub fn constructor(&self) -> &Option<FunctionDecl> {
        &self.constructor
    }

    /// Gets the list of all fields defined on this class. This includes all methods,
    /// fields, getters, and setters. Does not include the constructor.
    pub fn all_fields(&self) -> &[ClassField] {
        &self.fields
    }

    /// Gets the list of all static fields defined on this class. This includes all
    /// methods, fields, getters, and setters. Does not include the constructor.
    pub fn all_static_fields(&self) -> &[ClassField] {
        &self.static_fields
    }

    /// Returns an iterator that will loop through all methods on the class.
    pub fn methods(&self) -> impl Iterator<Item = &FunctionDecl> {
        self.fields
            .iter()
            .map(|v| match v {
                ClassField::Method(v) => Some(v),
                _ => None,
            })
            .flatten()
    }

    /// Returns an iterator that will loop through all methods on the class.
    pub fn static_methods(&self) -> impl Iterator<Item = &FunctionDecl> {
        self.static_fields
            .iter()
            .map(|v| match v {
                ClassField::Method(v) => Some(v),
                _ => None,
            })
            .flatten()
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        if self.fields.is_empty() && self.static_fields.is_empty() {
            return write!(f, "class {} {{}}", self.name);
        }
        let indent = "    ".repeat(indentation);
        writeln!(f, "class {} {{", self.name)?;
        for field in self.all_fields() {
            write!(f, "    {}", indent)?;
            match field {
                ClassField::Method(method) => {
                    method.display_no_function(f, indentation + 1)?;
                }
                ClassField::Field(name, value) => {
                    write!(f, "{} = {};", name, value)?;
                }
                ClassField::Getter(method) => {
                    write!(f, "get ")?;
                    method.display_no_function(f, indentation + 1)?;
                }
                ClassField::Setter(method) => {
                    write!(f, "set ")?;
                    method.display_no_function(f, indentation + 1)?;
                }
            }
            writeln!(f)?;
        }
        for field in self.all_static_fields() {
            write!(f, "    static {}", indent)?;
            match field {
                ClassField::Method(method) => {
                    method.display_no_function(f, indentation + 1)?;
                }
                ClassField::Field(name, value) => {
                    write!(f, "{} = {};", name, value)?;
                }
                ClassField::Getter(method) => {
                    write!(f, "get ")?;
                    method.display_no_function(f, indentation + 1)?;
                }
                ClassField::Setter(method) => {
                    write!(f, "set ")?;
                    method.display_no_function(f, indentation + 1)?;
                }
            }
            writeln!(f)?;
        }
        write!(f, "{}}}", indent)
    }
}

impl Executable for ClassDecl {
    fn run(&self, context: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("ClassDecl", "exec");
        let class = match &self.constructor {
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
        class.set_field("name", self.name(), false, context)?;

        // Setup non static things
        let proto = Value::Object(GcObject::new(Object::new()));
        for f in self.fields.iter() {
            match f {
                ClassField::Method(method) => {
                    let f = context.create_function(
                        method.parameters().to_vec(),
                        method.body().to_vec(),
                        FunctionFlags::CALLABLE | FunctionFlags::CONSTRUCTABLE,
                    )?;
                    proto.set_field(method.name(), f, false, context)?;
                }
                ClassField::Field(name, value) => {
                    proto.set_field(name.clone(), value.run(context)?, false, context)?;
                }
                _ => unimplemented!(),
            }
        }
        class.set_field(PROTOTYPE, proto, false, context)?;

        // Setup static things
        for method in self.static_methods() {
            let f = context.create_function(
                method.parameters().to_vec(),
                method.body().to_vec(),
                FunctionFlags::CALLABLE | FunctionFlags::CONSTRUCTABLE,
            )?;
            class.set_field(method.name(), f, false, context)?;
        }

        if context.has_binding(self.name()) {
            // TODO: Unclear if this is legal. In firefox, this produces a redeclaration error.
            context.set_mutable_binding(self.name(), class, true)?;
        } else {
            context.create_mutable_binding(
                self.name().to_owned(),
                false,
                VariableScope::Function,
            )?;

            context.initialize_binding(self.name(), class)?;
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
