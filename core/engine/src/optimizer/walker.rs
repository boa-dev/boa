use super::PassAction;
use boa_ast::{
    Expression,
    visitor::{VisitWith, VisitorMut},
};
use std::{convert::Infallible, ops::ControlFlow};

/// The utility structure that traverses the AST.
pub(crate) struct Walker<'arena, F>
where
    F: FnMut(&mut Expression<'arena>) -> PassAction<Expression<'arena>>,
{
    /// The function to be applied to the node.
    f: F,

    /// Did a change happen while traversing.
    changed: bool,

    _phantom: std::marker::PhantomData<&'arena ()>,
}

impl<'arena, F> Walker<'arena, F>
where
    F: FnMut(&mut Expression<'arena>) -> PassAction<Expression<'arena>>,
{
    pub(crate) const fn new(f: F) -> Self {
        Self {
            f,
            changed: false,
            _phantom: std::marker::PhantomData,
        }
    }

    pub(crate) const fn changed(&self) -> bool {
        self.changed
    }

    /// Walk the AST in postorder.
    pub(crate) fn walk_expression_postorder(&mut self, expr: &mut Expression<'arena>) {
        let _ = self.visit_expression_mut(expr);
    }
}

impl<'ast, 'arena: 'ast, F> VisitorMut<'ast, 'arena> for Walker<'arena, F>
where
    F: FnMut(&mut Expression<'arena>) -> PassAction<Expression<'arena>>,
{
    type BreakTy = Infallible;

    /// Visits the tree in postorder.
    fn visit_expression_mut(
        &mut self,
        expr: &'ast mut Expression<'arena>,
    ) -> ControlFlow<Self::BreakTy> {
        expr.visit_with_mut(self)?;

        match (self.f)(expr) {
            PassAction::Keep => {}
            PassAction::Modified => self.changed = true,
            PassAction::Replace(new) => {
                *expr = new;
                self.changed = true;
            }
        }

        ControlFlow::Continue(())
    }
}
