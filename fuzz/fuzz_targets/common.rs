use boa_ast::{
    visitor::{VisitWith, VisitorMut},
    Expression, StatementList,
};
use boa_interner::{Interner, Sym, ToInternedString};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use std::fmt::{Debug, Formatter};
use std::ops::ControlFlow;

/// Context for performing fuzzing. This structure contains both the generated AST as well as the
/// context used to resolve the symbols therein.
pub struct FuzzData {
    pub interner: Interner,
    pub ast: StatementList,
}

impl<'a> Arbitrary<'a> for FuzzData {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut interner = Interner::with_capacity(8);
        let mut syms_available = Vec::with_capacity(8);
        for c in 'a'..='h' {
            syms_available.push(interner.get_or_intern(&*String::from(c)));
        }

        let mut ast = StatementList::arbitrary(u)?;

        struct FuzzReplacer<'a, 's, 'u> {
            syms: &'s [Sym],
            u: &'u mut Unstructured<'a>,
        }
        impl<'a, 's, 'u, 'ast> VisitorMut<'ast> for FuzzReplacer<'a, 's, 'u> {
            type BreakTy = arbitrary::Error;

            // TODO arbitrary strings literals?

            fn visit_expression_mut(
                &mut self,
                node: &'ast mut Expression,
            ) -> ControlFlow<Self::BreakTy> {
                if matches!(node, Expression::FormalParameterList(_)) {
                    match self.u.arbitrary() {
                        Ok(id) => *node = Expression::Identifier(id),
                        Err(e) => return ControlFlow::Break(e),
                    }
                }
                node.visit_with_mut(self)
            }

            fn visit_sym_mut(&mut self, node: &'ast mut Sym) -> ControlFlow<Self::BreakTy> {
                *node = self.syms[node.get() % self.syms.len()];
                ControlFlow::Continue(())
            }
        }

        let mut replacer = FuzzReplacer {
            syms: &syms_available,
            u,
        };
        if let ControlFlow::Break(e) = replacer.visit_statement_list_mut(&mut ast) {
            Err(e)
        } else {
            Ok(Self { interner, ast })
        }
    }
}

impl Debug for FuzzData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuzzData")
            .field("ast", &self.ast)
            .finish_non_exhaustive()
    }
}

pub struct FuzzSource {
    pub interner: Interner,
    pub source: String,
}

impl<'a> Arbitrary<'a> for FuzzSource {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let data = FuzzData::arbitrary(u)?;
        let source = data.ast.to_interned_string(&data.interner);
        Ok(Self {
            interner: data.interner,
            source,
        })
    }
}

impl Debug for FuzzSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Fuzzed source:\n{}", self.source))
    }
}
