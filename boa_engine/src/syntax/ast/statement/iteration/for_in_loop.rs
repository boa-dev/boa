use crate::syntax::ast::{
    expression::Expression,
    statement::{iteration::IterableLoopInitializer, Statement},
    ContainsSymbol,
};
use boa_interner::{Interner, Sym, ToInternedString};
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ForInLoop {
    init: IterableLoopInitializer,
    expr: Expression,
    body: Box<Statement>,
    label: Option<Sym>,
}

impl ForInLoop {
    pub fn new(init: IterableLoopInitializer, expr: Expression, body: Statement) -> Self {
        Self {
            init,
            expr,
            body: body.into(),
            label: None,
        }
    }

    pub fn init(&self) -> &IterableLoopInitializer {
        &self.init
    }

    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    pub fn body(&self) -> &Statement {
        &self.body
    }

    pub fn label(&self) -> Option<Sym> {
        self.label
    }

    pub fn set_label(&mut self, label: Sym) {
        self.label = Some(label);
    }

    /// Converts the "for in" loop to a string with the given indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = if let Some(label) = self.label {
            format!("{}: ", interner.resolve_expect(label))
        } else {
            String::new()
        };
        buf.push_str(&format!(
            "for ({} in {}) ",
            self.init.to_interned_string(interner),
            self.expr.to_interned_string(interner)
        ));
        buf.push_str(&self.body().to_indented_string(interner, indentation));

        buf
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.init.contains_arguments()
            || self.expr.contains_arguments()
            || self.body.contains_arguments()
            || matches!(self.label, Some(label) if label == Sym::ARGUMENTS)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.init.contains(symbol) || self.expr.contains(symbol) || self.body.contains(symbol)
    }
}

impl ToInternedString for ForInLoop {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<ForInLoop> for Statement {
    fn from(for_in: ForInLoop) -> Self {
        Self::ForInLoop(for_in)
    }
}
