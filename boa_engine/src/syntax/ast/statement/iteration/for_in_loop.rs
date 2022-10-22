use crate::syntax::ast::{
    expression::Expression,
    statement::{iteration::IterableLoopInitializer, Statement},
    ContainsSymbol,
};
use boa_interner::{Interner, ToIndentedString, ToInternedString};

#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ForInLoop {
    init: IterableLoopInitializer,
    expr: Expression,
    body: Box<Statement>,
}

impl ForInLoop {
    /// Creates a new `ForInLoop`.
    #[inline]
    pub fn new(init: IterableLoopInitializer, expr: Expression, body: Statement) -> Self {
        Self {
            init,
            expr,
            body: body.into(),
        }
    }

    /// Gets the initializer of the for...in loop.
    #[inline]
    pub fn init(&self) -> &IterableLoopInitializer {
        &self.init
    }

    /// Gets the for...in loop expression.
    #[inline]
    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    /// Gets the body of the for...in loop.
    #[inline]
    pub fn body(&self) -> &Statement {
        &self.body
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.init.contains_arguments()
            || self.expr.contains_arguments()
            || self.body.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.init.contains(symbol) || self.expr.contains(symbol) || self.body.contains(symbol)
    }
}

impl ToIndentedString for ForInLoop {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = format!(
            "for ({} in {}) ",
            self.init.to_interned_string(interner),
            self.expr.to_interned_string(interner)
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
