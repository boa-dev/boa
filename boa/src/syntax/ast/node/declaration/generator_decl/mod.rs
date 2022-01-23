use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::{join_nodes, FormalParameter, Node, StatementList},
};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The `function*` declaration (`function` keyword followed by an asterisk) defines a generator function,
/// which returns a `Generator` object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-GeneratorDeclaration
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function*
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct GeneratorDecl {
    name: Sym,
    parameters: Box<[FormalParameter]>,
    body: StatementList,
}

impl GeneratorDecl {
    /// Creates a new generator declaration.
    pub(in crate::syntax) fn new<P, B>(name: Sym, parameters: P, body: B) -> Self
    where
        P: Into<Box<[FormalParameter]>>,
        B: Into<StatementList>,
    {
        Self {
            name,
            parameters: parameters.into(),
            body: body.into(),
        }
    }

    /// Gets the name of the generator declaration.
    pub fn name(&self) -> Sym {
        self.name
    }

    /// Gets the list of parameters of the generator declaration.
    pub fn parameters(&self) -> &[FormalParameter] {
        &self.parameters
    }

    /// Gets the body of the generator declaration.
    pub fn body(&self) -> &[Node] {
        self.body.items()
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indentation: usize,
    ) -> String {
        let mut buf = format!(
            "function* {}({}",
            interner.resolve_expect(self.name),
            join_nodes(interner, &self.parameters)
        );
        if self.body().is_empty() {
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

impl From<GeneratorDecl> for Node {
    fn from(decl: GeneratorDecl) -> Self {
        Self::GeneratorDecl(decl)
    }
}

impl ToInternedString for GeneratorDecl {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}
