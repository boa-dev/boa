//! Async Function Expression.

use crate::syntax::ast::join_nodes;
use crate::syntax::ast::statement::StatementList;
use crate::syntax::ast::{expression::Expression, Statement};
use boa_interner::{Interner, Sym, ToInternedString};

use super::FormalParameterList;

/// An async function expression is very similar to an async function declaration except used within
/// a wider expression (for example during an assignment).
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncFunctionExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/async_function
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncFunction {
    name: Option<Sym>,
    parameters: FormalParameterList,
    body: StatementList,
}

impl AsyncFunction {
    /// Creates a new function expression
    pub(in crate::syntax) fn new(
        name: Option<Sym>,
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
    pub fn name(&self) -> Option<Sym> {
        self.name
    }

    /// Gets the list of parameters of the function declaration.
    pub fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the function declaration.
    pub fn body(&self) -> &StatementList {
        &self.body
    }

    /// Implements the display formatting with indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = "async function".to_owned();
        if let Some(name) = self.name {
            buf.push_str(&format!(" {}", interner.resolve_expect(name)));
        }
        buf.push_str(&format!(
            "({}",
            join_nodes(interner, &self.parameters.parameters)
        ));
        if self.body().statements().is_empty() {
            buf.push_str(") {}");
        } else {
            buf.push_str(&format!(
                ") {{\n{}{}}}",
                self.body.to_indented_string(interner, indentation + 1),
                "    ".repeat(indentation)
            ));
        }
        buf
    }
}

impl ToInternedString for AsyncFunction {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<AsyncFunction> for Expression {
    fn from(expr: AsyncFunction) -> Self {
        Self::AsyncFunction(expr)
    }
}

impl From<AsyncFunction> for Statement {
    fn from(f: AsyncFunction) -> Self {
        Self::AsyncFunction(f)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fmt() {
        crate::syntax::ast::test_formatting(
            r#"
            async function async_func(a, b) {
                console.log(a);
            }
            async function async_func_2(a, b) {}
            pass_async_func(async function(a, b) {
                console.log("in async callback", a);
            });
            pass_async_func(async function(a, b) {});
            "#,
        );
    }
}
