use crate::syntax::ast::{
    expression::Expression,
    statement::{iteration::IterableLoopInitializer, Statement},
    ContainsSymbol,
};
use boa_interner::{Interner, ToIndentedString, ToInternedString};

/// A `for...in` loop statement, as defined by the [spec].
///
/// [`for...in`][forin] statements loop over all enumerable string properties of an object, including
/// inherited properties.
///
/// [forin]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...in
/// [spec]: https://tc39.es/ecma262/#prod-ForInOfStatement
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ForInLoop {
    initializer: IterableLoopInitializer,
    target: Expression,
    body: Box<Statement>,
}

impl ForInLoop {
    /// Creates a new `ForInLoop`.
    #[inline]
    pub fn new(initializer: IterableLoopInitializer, target: Expression, body: Statement) -> Self {
        Self {
            initializer,
            target,
            body: body.into(),
        }
    }

    /// Gets the initializer of the for...in loop.
    #[inline]
    pub fn initializer(&self) -> &IterableLoopInitializer {
        &self.initializer
    }

    /// Gets the target object of the for...in loop.
    #[inline]
    pub fn target(&self) -> &Expression {
        &self.target
    }

    /// Gets the body of the for...in loop.
    #[inline]
    pub fn body(&self) -> &Statement {
        &self.body
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.initializer.contains_arguments()
            || self.target.contains_arguments()
            || self.body.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.initializer.contains(symbol)
            || self.target.contains(symbol)
            || self.body.contains(symbol)
    }
}

impl ToIndentedString for ForInLoop {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = format!(
            "for ({} in {}) ",
            self.initializer.to_interned_string(interner),
            self.target.to_interned_string(interner)
        );
        buf.push_str(&self.body().to_indented_string(interner, indentation));

        buf
    }
}

impl From<ForInLoop> for Statement {
    #[inline]
    fn from(for_in: ForInLoop) -> Self {
        Self::ForInLoop(for_in)
    }
}
