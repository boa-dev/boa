use arbitrary::{Arbitrary, Unstructured};
use boa_ast::{
    visitor::{VisitWith, VisitorMut},
    Expression, StatementList,
};
use boa_interner::{Interner, Sym, ToInternedString};
use std::{
    fmt::{Debug, Formatter},
    ops::ControlFlow,
};

/// Context for performing fuzzing. This structure contains both the generated AST as well as the
/// context used to resolve the symbols therein.
pub struct FuzzData<'arena> {
    pub interner: Interner,
    pub ast: StatementList<'arena>,
}

impl<'a, 'arena> Arbitrary<'a> for FuzzData<'arena>
where
    'a: 'arena,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut interner = Interner::with_capacity(8);
        let mut syms_available = Vec::with_capacity(8);
        for c in 'a'..='h' {
            syms_available.push(interner.get_or_intern(&*String::from(c)));
        }

        let mut ast = StatementList::arbitrary(u)?;

        struct FuzzReplacer<'s> {
            syms: &'s [Sym],
        }
        impl<'s, 'ast, 'arena> VisitorMut<'ast, 'arena> for FuzzReplacer<'s>
        where
            'arena: 'ast,
        {
            type BreakTy = arbitrary::Error;

            // TODO arbitrary strings literals?

            fn visit_expression_mut(
                &mut self,
                node: &'ast mut Expression<'arena>,
            ) -> ControlFlow<Self::BreakTy> {
                node.visit_with_mut(self)
            }

            fn visit_sym_mut(&mut self, node: &'ast mut Sym) -> ControlFlow<Self::BreakTy> {
                *node = self.syms[node.get() % self.syms.len()];
                ControlFlow::Continue(())
            }
        }

        let mut replacer = FuzzReplacer {
            syms: &syms_available,
        };
        if let ControlFlow::Break(e) = replacer.visit_statement_list_mut(&mut ast) {
            Err(e)
        } else {
            Ok(Self { interner, ast })
        }
    }
}

impl Debug for FuzzData<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuzzData")
            .field("ast", &self.ast)
            .finish_non_exhaustive()
    }
}

#[allow(dead_code)]
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
