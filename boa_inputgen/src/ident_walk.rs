use boa_engine::syntax::ast::node::visit::Visitor;
use boa_engine::syntax::ast::node::StatementList;
use boa_interner::Sym;

struct SymReplacer<'a> {
    syms: &'a [Sym],
}

impl<'a, 'ast> Visitor<'ast> for SymReplacer<'a> {
    fn visit_sym_mut(&mut self, sym: &'ast mut Sym) {
        *sym = self.syms[sym.as_raw().get() % self.syms.len()];
    }
}

pub(crate) fn replace_syms(syms: &[Sym], sample: &mut StatementList) {
    let mut replacer = SymReplacer { syms };
    replacer.visit_statement_list_mut(sample);
}
