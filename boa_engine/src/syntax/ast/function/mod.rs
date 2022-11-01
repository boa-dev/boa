//! Functions and classes nodes, as defined by the [spec].
//!
//! [Functions][func] are mainly subprograms that can be called by external code to execute a sequence of
//! statements (the *body* of the function). Javascript functions fall in several categories:
//!
//! - [`Function`]s.
//! - [`ArrowFunction`]s.
//! - [`Generator`]s.
//! - [`AsyncFunction`]s.
//! - [`AsyncGenerator`]s.
//!
//! All of them can be declared in either [declaration][decl] form or [expression][expr] form,
//! except from `ArrowFunction`s, which can only be declared in expression form.
//!
//! This module also contains [`Class`]es, which are templates for creating objects. Classes
//! can also be declared in either declaration or expression form.
//!
//! [spec]: https://tc39.es/ecma262/#sec-ecmascript-language-functions-and-classes
//! [func]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions
//! [decl]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
//! [expr]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function

mod arrow_function;
mod async_function;
mod async_generator;
mod class;
mod generator;
mod parameters;

pub use arrow_function::ArrowFunction;
pub use async_function::AsyncFunction;
pub use async_generator::AsyncGenerator;
pub use class::{Class, ClassElement};
pub use generator::Generator;
pub use parameters::{FormalParameter, FormalParameterList};
use std::ops::ControlFlow;

pub(crate) use parameters::FormalParameterListFlags;

use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
use crate::syntax::ast::{block_to_string, join_nodes, StatementList};
use crate::try_break;
use boa_interner::{Interner, ToIndentedString};

use super::expression::{Expression, Identifier};
use super::{ContainsSymbol, Declaration};

/// A function definition, as defined by the [spec].
///
/// By default, functions return `undefined`. To return any other value, the function must have
/// a return statement that specifies the value to return.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-function-definitions
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    name: Option<Identifier>,
    parameters: FormalParameterList,
    body: StatementList,
}

impl Function {
    /// Creates a new function expression
    #[inline]
    pub(in crate::syntax) fn new(
        name: Option<Identifier>,
        parameters: FormalParameterList,
        body: StatementList,
    ) -> Self {
        Self {
            name,
            parameters,
            body,
        }
    }

    /// Gets the name of the function declaration.
    #[inline]
    pub fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Gets the list of parameters of the function declaration.
    #[inline]
    pub fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the function declaration.
    #[inline]
    pub fn body(&self) -> &StatementList {
        &self.body
    }
}

impl ToIndentedString for Function {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = "function".to_owned();
        if let Some(name) = self.name {
            buf.push_str(&format!(" {}", interner.resolve_expect(name.sym())));
        }
        buf.push_str(&format!(
            "({}) {}",
            join_nodes(interner, &self.parameters.parameters),
            block_to_string(&self.body, interner, indentation)
        ));

        buf
    }
}

impl From<Function> for Expression {
    #[inline]
    fn from(expr: Function) -> Self {
        Self::Function(expr)
    }
}

impl From<Function> for Declaration {
    #[inline]
    fn from(f: Function) -> Self {
        Self::Function(f)
    }
}

/// Helper function to check if a function contains a super call or super property access.
pub(crate) fn function_contains_super(
    body: &StatementList,
    parameters: &FormalParameterList,
) -> bool {
    for param in parameters.parameters.iter() {
        if param.variable().contains(ContainsSymbol::SuperCall)
            || param.variable().contains(ContainsSymbol::SuperProperty)
        {
            return true;
        }
    }
    for node in body.statements() {
        if node.contains(ContainsSymbol::SuperCall) || node.contains(ContainsSymbol::SuperProperty)
        {
            return true;
        }
    }
    false
}

/// Returns `true` if the function parameters or body contain a direct `super` call.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-hasdirectsuper
pub(crate) fn has_direct_super(body: &StatementList, parameters: &FormalParameterList) -> bool {
    for param in parameters.parameters.iter() {
        if param.variable().contains(ContainsSymbol::SuperCall) {
            return true;
        }
    }
    for node in body.statements() {
        if node.contains(ContainsSymbol::SuperCall) {
            return true;
        }
    }
    false
}

impl VisitWith for Function {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(ident) = &self.name {
            try_break!(visitor.visit_identifier(ident));
        }
        try_break!(visitor.visit_formal_parameter_list(&self.parameters));
        visitor.visit_statement_list(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(ident) = &mut self.name {
            try_break!(visitor.visit_identifier_mut(ident));
        }
        try_break!(visitor.visit_formal_parameter_list_mut(&mut self.parameters));
        visitor.visit_statement_list_mut(&mut self.body)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn duplicate_function_name() {
        let scenario = r#"
    function f () {}
    function f () {return 12;}
    f()
    "#;

        assert_eq!(&crate::exec(scenario), "12");
    }

    #[test]
    fn fmt() {
        crate::syntax::ast::test_formatting(
            r#"
        function func(a, b) {
            console.log(a);
        }
        function func_2(a, b) {}
        pass_func(function(a, b) {
            console.log("in callback", a);
        });
        pass_func(function(a, b) {});
        "#,
        );
    }
}
