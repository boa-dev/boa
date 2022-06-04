use crate::syntax::ast::node::{join_nodes, FormalParameterList, Node, StatementList};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// An arrow function expression is a syntactically compact alternative to a regular function
/// expression.
///
/// Arrow function expressions are ill suited as methods, and they cannot be used as
/// constructors. Arrow functions cannot be used as constructors and will throw an error when
/// used with new.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ArrowFunction
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ArrowFunctionDecl {
    name: Option<Sym>,
    params: FormalParameterList,
    body: StatementList,
}

impl ArrowFunctionDecl {
    /// Creates a new `ArrowFunctionDecl` AST node.
    pub(in crate::syntax) fn new<N, P, B>(name: N, params: P, body: B) -> Self
    where
        N: Into<Option<Sym>>,
        P: Into<FormalParameterList>,
        B: Into<StatementList>,
    {
        Self {
            name: name.into(),
            params: params.into(),
            body: body.into(),
        }
    }

    /// Gets the name of the function declaration.
    pub fn name(&self) -> Option<Sym> {
        self.name
    }

    /// Sets the name of the function declaration.
    pub fn set_name(&mut self, name: Option<Sym>) {
        self.name = name;
    }

    /// Gets the list of parameters of the arrow function.
    pub(crate) fn params(&self) -> &FormalParameterList {
        &self.params
    }

    /// Gets the body of the arrow function.
    pub(crate) fn body(&self) -> &StatementList {
        &self.body
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indentation: usize,
    ) -> String {
        let mut buf = format!("({}", join_nodes(interner, &self.params.parameters));
        if self.body().items().is_empty() {
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

impl ToInternedString for ArrowFunctionDecl {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<ArrowFunctionDecl> for Node {
    fn from(decl: ArrowFunctionDecl) -> Self {
        Self::ArrowFunctionDecl(decl)
    }
}
