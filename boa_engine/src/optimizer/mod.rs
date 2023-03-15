//! Implements optimizations.

pub(crate) mod pass;
pub(crate) mod walker;

use self::{pass::ConstantFolding, walker::Walker};
use crate::Context;
use bitflags::bitflags;
use boa_ast::{visitor::VisitorMut, Expression, StatementList};
use std::{fmt, ops::ControlFlow};

bitflags! {
    /// Optimizer options.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct OptimizerOptions: u8 {
        /// Print statistics to `stdout`.
        const STATISTICS = 0b0000_0001;

        /// Apply contant folding optimization.
        const CONSTANT_FOLDING = 0b0000_0010;

        /// Apply all optimizations.
        const OPTIMIZE_ALL = Self::CONSTANT_FOLDING.bits();
    }
}

/// The action to be performed after an optimization step.
#[derive(Debug)]
pub(crate) enum PassAction<T> {
    /// Keep the node, do nothing.
    Keep,

    /// The node was modified inplace.
    Modified,

    /// Replace the node.
    Replace(T),
}

/// Contains statistics about the optimizer execution.
#[derive(Debug, Default, Clone, Copy)]
pub struct OptimizerStatistics {
    /// How many times was the optimization run in total.
    pub constant_folding_run_count: usize,

    /// How many passes did the optimization run in total.
    pub constant_folding_pass_count: usize,
}

impl fmt::Display for OptimizerStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Optimizer {{")?;
        writeln!(
            f,
            "    constant folding: {} run(s), {} pass(es) ({} mutating, {} checking)",
            self.constant_folding_run_count,
            self.constant_folding_pass_count,
            self.constant_folding_pass_count
                .saturating_sub(self.constant_folding_run_count),
            self.constant_folding_run_count
        )?;
        writeln!(f, "}}")?;
        Ok(())
    }
}

/// This represents an AST optimizer.
#[derive(Debug)]
pub(crate) struct Optimizer<'context, 'host> {
    statistics: OptimizerStatistics,
    context: &'context mut Context<'host>,
}

impl<'context, 'host> Optimizer<'context, 'host> {
    /// Create a optimizer.
    pub(crate) fn new(context: &'context mut Context<'host>) -> Self {
        Self {
            statistics: OptimizerStatistics::default(),
            context,
        }
    }

    /// Run the constant folding optimization on an expression.
    fn run_constant_folding_pass(&mut self, expr: &mut Expression) -> bool {
        self.statistics.constant_folding_run_count += 1;

        let mut has_changes = false;
        loop {
            self.statistics.constant_folding_pass_count += 1;
            let mut walker = Walker::new(|expr| -> PassAction<Expression> {
                ConstantFolding::fold_expression(expr, self.context)
            });
            // NOTE: postoder traversal is optimal for constant folding,
            // since it evaluates the tree bottom-up.
            walker.walk_expression_postorder(expr);
            if !walker.changed() {
                break;
            }
            has_changes = true;
        }
        has_changes
    }

    fn run_all(&mut self, expr: &mut Expression) {
        if self
            .context
            .optimizer_options()
            .contains(OptimizerOptions::CONSTANT_FOLDING)
        {
            self.run_constant_folding_pass(expr);
        }
    }

    /// Apply optimizations inplace.
    pub(crate) fn apply(&mut self, statement_list: &mut StatementList) -> OptimizerStatistics {
        self.visit_statement_list_mut(statement_list);
        if self
            .context
            .optimizer_options()
            .contains(OptimizerOptions::STATISTICS)
        {
            println!("{}", self.statistics);
        }
        self.statistics
    }
}

impl<'ast> VisitorMut<'ast> for Optimizer<'_, '_> {
    type BreakTy = ();

    fn visit_expression_mut(&mut self, node: &'ast mut Expression) -> ControlFlow<Self::BreakTy> {
        self.run_all(node);
        ControlFlow::Continue(())
    }
}
