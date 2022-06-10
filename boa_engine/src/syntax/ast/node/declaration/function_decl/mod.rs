use crate::syntax::ast::node::{join_nodes, FormalParameterList, Node, StatementList};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The `function` declaration (function statement) defines a function with the specified
/// parameters.
///
/// A function created with a function declaration is a `Function` object and has all the
/// properties, methods and behavior of `Function`.
///
/// A function can also be created using an expression (see [function expression][func_expr]).
///
/// By default, functions return `undefined`. To return any other value, the function must have
/// a return statement that specifies the value to return.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-function
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
/// [func_expr]: ../enum.Node.html#variant.FunctionExpr
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDecl {
    name: Sym,
    parameters: FormalParameterList,
    body: StatementList,
}

impl FunctionDecl {
    /// Creates a new function declaration.
    pub(in crate::syntax) fn new<P, B>(name: Sym, parameters: P, body: B) -> Self
    where
        P: Into<FormalParameterList>,
        B: Into<StatementList>,
    {
        Self {
            name,
            parameters: parameters.into(),
            body: body.into(),
        }
    }

    /// Gets the name of the function declaration.
    pub fn name(&self) -> Sym {
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
    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indentation: usize,
    ) -> String {
        let mut buf = format!(
            "function {}({}",
            interner.resolve_expect(self.name),
            join_nodes(interner, &self.parameters.parameters)
        );
        if self.body().items().is_empty() {
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

impl From<FunctionDecl> for Node {
    fn from(decl: FunctionDecl) -> Self {
        Self::FunctionDecl(decl)
    }
}

impl ToInternedString for FunctionDecl {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}
