use crate::syntax::ast::{
    expression::{Expression, Identifier},
    join_nodes, ContainsSymbol, StatementList,
};
use boa_interner::{Interner, ToIndentedString};

use super::FormalParameterList;

/// An arrow function expression, as defined by the [spec].
///
/// An [arrow function][mdn] expression is a syntactically compact alternative to a regular function
/// expression. Arrow function expressions are ill suited as methods, and they cannot be used as
/// constructors. Arrow functions cannot be used as constructors and will throw an error when
/// used with new.
///
/// [spec]: https://tc39.es/ecma262/#prod-ArrowFunction
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ArrowFunction {
    name: Option<Identifier>,
    parameters: FormalParameterList,
    body: StatementList,
}

impl ArrowFunction {
    /// Creates a new `ArrowFunctionDecl` AST Expression.
    #[inline]
    pub(in crate::syntax) fn new(
        name: Option<Identifier>,
        params: FormalParameterList,
        body: StatementList,
    ) -> Self {
        Self {
            name,
            parameters: params,
            body,
        }
    }

    /// Gets the name of the function declaration.
    #[inline]
    pub fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Sets the name of the function declaration.
    #[inline]
    pub fn set_name(&mut self, name: Option<Identifier>) {
        self.name = name;
    }

    /// Gets the list of parameters of the arrow function.
    #[inline]
    pub(crate) fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the arrow function.
    #[inline]
    pub(crate) fn body(&self) -> &StatementList {
        &self.body
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.parameters.contains_arguments() || self.body.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        if ![
            ContainsSymbol::NewTarget,
            ContainsSymbol::SuperProperty,
            ContainsSymbol::SuperCall,
            ContainsSymbol::This,
        ]
        .contains(&symbol)
        {
            return false;
        }
        self.parameters.contains(symbol) || self.body.contains(symbol)
    }
}

impl ToIndentedString for ArrowFunction {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = format!("({}", join_nodes(interner, &self.parameters.parameters));
        if self.body().statements().is_empty() {
            buf.push_str(") => {}");
        } else {
            buf.push_str(&format!(
                ") => {{\n{}{}}}",
                self.body.to_indented_string(interner, indentation + 1),
                "    ".repeat(indentation)
            ));
        }
        buf
    }
}

impl From<ArrowFunction> for Expression {
    fn from(decl: ArrowFunction) -> Self {
        Self::ArrowFunction(decl)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fmt() {
        crate::syntax::ast::test_formatting(
            r#"
        let arrow_func = (a, b) => {
            console.log("in multi statement arrow");
            console.log(b);
        };
        let arrow_func_2 = (a, b) => {};
        "#,
        );
    }
}
