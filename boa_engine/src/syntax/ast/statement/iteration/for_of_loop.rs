use crate::syntax::ast::{
    expression::Expression,
    statement::{iteration::IterableLoopInitializer, Statement},
    ContainsSymbol,
};
use boa_interner::{Interner, ToIndentedString, ToInternedString};

/// A `for...of` loop statement, as defined by the [spec].
///
/// [`for..of`][forof] statements loop over a sequence of values obtained from an iterable object (Array,
/// String, Map, generators).
///
/// This type combines `for..of` and [`for await...of`][forawait] statements in a single structure,
/// since `for await...of` is essentially the same statement but with async iterable objects
/// as the source of iteration.
///
/// [forof]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...of
/// [spec]: https://tc39.es/ecma262/#prod-ForInOfStatement
/// [forawait]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for-await...of
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ForOfLoop {
    init: IterableLoopInitializer,
    iterable: Expression,
    body: Box<Statement>,
    r#await: bool,
}

impl ForOfLoop {
    /// Creates a new "for of" loop AST node.
    #[inline]
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

    /// Gets the initializer of the for...of loop.
    #[inline]
    pub fn init(&self) -> &IterableLoopInitializer {
        &self.init
    }

    /// Gets the iterable expression of the for...of loop.
    #[inline]
    pub fn iterable(&self) -> &Expression {
        &self.iterable
    }

    /// Gets the body to execute in the for...of loop.
    #[inline]
    pub fn body(&self) -> &Statement {
        &self.body
    }

    /// Returns true if this "for...of" loop is an "for await...of" loop.
    #[inline]
    pub(crate) fn r#await(&self) -> bool {
        self.r#await
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

impl ToIndentedString for ForOfLoop {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "for ({} of {}) {}",
            self.init.to_interned_string(interner),
            self.iterable.to_interned_string(interner),
            self.body().to_indented_string(interner, indentation)
        )
    }
}

impl From<ForOfLoop> for Statement {
    #[inline]
    fn from(for_of: ForOfLoop) -> Self {
        Self::ForOfLoop(for_of)
    }
}
