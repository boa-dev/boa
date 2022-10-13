use crate::syntax::ast::{
    expression::Expression,
    statement::{iteration::IterableLoopInitializer, Statement},
    ContainsSymbol,
};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ForOfLoop {
    init: IterableLoopInitializer,
    iterable: Expression,
    body: Box<Statement>,
    r#await: bool,
}

impl ForOfLoop {
    /// Creates a new "for of" loop AST node.
    pub fn new(
        init: IterableLoopInitializer,
        iterable: Expression,
        body: Statement,
        r#await: bool,
    ) -> Self {
        Self {
            init,
            iterable,
            body: body.into(),
            r#await,
        }
    }

    pub fn init(&self) -> &IterableLoopInitializer {
        &self.init
    }

    pub fn iterable(&self) -> &Expression {
        &self.iterable
    }

    pub fn body(&self) -> &Statement {
        &self.body
    }

    /// Returns true if this "for...of" loop is an "for await...of" loop.
    pub(crate) fn r#await(&self) -> bool {
        self.r#await
    }

    /// Converts the "for of" loop to a string with the given indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "for ({} of {}) {}",
            self.init.to_interned_string(interner),
            self.iterable.to_interned_string(interner),
            self.body().to_indented_string(interner, indentation)
        )
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.init.contains_arguments()
            || self.iterable.contains_arguments()
            || self.body.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.init.contains(symbol) || self.iterable.contains(symbol) || self.body.contains(symbol)
    }
}

impl ToInternedString for ForOfLoop {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<ForOfLoop> for Statement {
    fn from(for_of: ForOfLoop) -> Self {
        Self::ForOfLoop(for_of)
    }
}
