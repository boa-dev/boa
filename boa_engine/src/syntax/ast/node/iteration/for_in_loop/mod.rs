use crate::syntax::ast::node::{iteration::IterableLoopInitializer, Node};
use boa_gc::{Finalize, Trace};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "fuzzer", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ForInLoop {
    init: Box<IterableLoopInitializer>,
    expr: Box<Node>,
    body: Box<Node>,
    label: Option<Sym>,
}

impl ForInLoop {
    pub fn new<I, B>(init: IterableLoopInitializer, expr: I, body: B) -> Self
    where
        I: Into<Node>,
        B: Into<Node>,
    {
        Self {
            init: Box::new(init),
            expr: Box::new(expr.into()),
            body: Box::new(body.into()),
            label: None,
        }
    }

    pub fn init(&self) -> &IterableLoopInitializer {
        &self.init
    }

    pub fn expr(&self) -> &Node {
        &self.expr
    }

    pub fn body(&self) -> &Node {
        &self.body
    }

    pub fn label(&self) -> Option<Sym> {
        self.label
    }

    pub fn set_label(&mut self, label: Sym) {
        self.label = Some(label);
    }

    #[cfg(feature = "fuzzer")]
    pub fn init_mut(&mut self) -> &mut IterableLoopInitializer {
        &mut self.init
    }

    #[cfg(feature = "fuzzer")]
    pub fn expr_mut(&mut self) -> &mut Node {
        &mut self.expr
    }

    #[cfg(feature = "fuzzer")]
    pub fn body_mut(&mut self) -> &mut Node {
        &mut self.body
    }

    #[cfg(feature = "fuzzer")]
    pub fn label_mut(&mut self) -> Option<&mut Sym> {
        self.label.as_mut()
    }

    /// Converts the "for in" loop to a string with the given indentation.
    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indentation: usize,
    ) -> String {
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
}

impl ToInternedString for ForInLoop {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<ForInLoop> for Node {
    fn from(for_in: ForInLoop) -> Self {
        Self::ForInLoop(for_in)
    }
}
