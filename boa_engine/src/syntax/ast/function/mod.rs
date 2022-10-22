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

pub(crate) use parameters::FormalParameterListFlags;

use crate::syntax::ast::{block_to_string, join_nodes, StatementList};
use boa_interner::{Interner, ToIndentedString};

use super::expression::{Expression, Identifier};
use super::{ContainsSymbol, Declaration};

/// The `function` expression defines a function with the specified parameters.
///
/// A function created with a function expression is a `Function` object and has all the
/// properties, methods and behavior of `Function`.
///
/// A function can also be created using a declaration (see function expression).
///
/// By default, functions return `undefined`. To return any other value, the function must have
/// a return statement that specifies the value to return.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-function
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
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
