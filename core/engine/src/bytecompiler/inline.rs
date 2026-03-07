//! IIFE (Immediately Invoked Function Expression) inlining optimization.
//!
//! This module implements compile-time inlining of simple IIFE patterns like:
//!   - `((a, b) => a + b)(x, y)`
//!   - `(function(a, b) { return a + b; })(x, y)`
//!   - `(() => { sideEffect(); })()`
//!
//! Instead of creating a function object, pushing a call frame, and executing
//! the callee's bytecode in a separate frame, the callee's body is compiled
//! directly into the caller's bytecode with parameters mapped to registers
//! containing the argument values.

use std::ops::ControlFlow;

use super::ByteCompiler;
use boa_ast::{
    Expression, Statement, StatementListItem,
    declaration::Binding,
    function::{ArrowFunction, FunctionExpression},
    visitor::{VisitWith, Visitor},
};

#[cfg(test)]
mod tests;

/// Describes the shape of an inlinable function body.
enum InlinableBody<'a> {
    /// A single expression whose value is the result.
    /// e.g., `=> expr` or `{ return expr; }`
    SingleExpression(&'a Expression),

    /// A list of statements with no return — result is `undefined`.
    /// e.g., `{ stmt1; stmt2; }` or `=> { stmt1; stmt2; }`
    StatementsOnly(&'a [StatementListItem]),

    /// Statements followed by a trailing `return expr;`.
    /// e.g., `{ stmt1; stmt2; return expr; }`
    StatementsWithReturn {
        stmts: &'a [StatementListItem],
        return_expr: &'a Expression,
    },
}

impl ByteCompiler<'_> {
    /// Try to inline an IIFE arrow function call: `((params) => body)(args)`.
    ///
    /// Returns `true` if the call was successfully inlined, `false` if it
    /// should fall back to normal call compilation.
    pub(crate) fn try_inline_arrow_call(
        &mut self,
        arrow: &ArrowFunction,
        args: &[Expression],
        dst: &super::Register,
    ) -> bool {
        if !Self::is_arrow_inlinable(arrow) {
            return false;
        }

        let Some(body) = Self::classify_body(arrow.body().statement_list().statements()) else {
            return false;
        };

        let params: Vec<_> = arrow.parameters().as_ref().to_vec();

        self.inline_function_body(&params, args, &body, arrow.scopes(), dst);

        true
    }

    /// Try to inline an IIFE function expression call: `(function(params) { ... })(args)`.
    pub(crate) fn try_inline_function_call(
        &mut self,
        func: &FunctionExpression,
        args: &[Expression],
        dst: &super::Register,
    ) -> bool {
        if !Self::is_function_expr_inlinable(func) {
            return false;
        }

        let Some(body) = Self::classify_body(func.body().statement_list().statements()) else {
            return false;
        };

        let params: Vec<_> = func.parameters().as_ref().to_vec();

        self.inline_function_body(&params, args, &body, func.scopes(), dst);

        true
    }

    /// Check if an arrow function is eligible for inlining.
    fn is_arrow_inlinable(arrow: &ArrowFunction) -> bool {
        if arrow.contains_direct_eval() {
            return false;
        }

        let params = arrow.parameters();

        // Simple parameters only (no destructuring, no defaults, no rest)
        if !params.is_simple() {
            return false;
        }
        if params
            .as_ref()
            .iter()
            .any(boa_ast::function::FormalParameter::is_rest_param)
        {
            return false;
        }

        let scopes = arrow.scopes();

        // All bindings must be register-local (no environment needed)
        if !scopes.function_scope().all_bindings_local() {
            return false;
        }

        // No parameter expressions requiring separate scope
        if scopes.parameters_scope().is_some() {
            return false;
        }
        if scopes.parameters_eval_scope().is_some() {
            return false;
        }

        true
    }

    /// Check if a function expression is eligible for inlining.
    fn is_function_expr_inlinable(func: &FunctionExpression) -> bool {
        if func.contains_direct_eval() {
            return false;
        }

        // No binding identifier (no self-reference like `(function f() { f(); })()`)
        if func.has_binding_identifier() {
            return false;
        }

        let params = func.parameters();

        if !params.is_simple() {
            return false;
        }
        if params
            .as_ref()
            .iter()
            .any(boa_ast::function::FormalParameter::is_rest_param)
        {
            return false;
        }

        let scopes = func.scopes();

        if !scopes.function_scope().all_bindings_local() {
            return false;
        }
        if scopes.parameters_scope().is_some() {
            return false;
        }
        if scopes.parameters_eval_scope().is_some() {
            return false;
        }

        // Function expressions can require a function scope for `this`, `arguments`, etc.
        if scopes.requires_function_scope() {
            return false;
        }

        true
    }

    /// Classify a function body into an [`InlinableBody`] variant, or return
    /// `None` if the body cannot be inlined.
    ///
    /// The body is inlinable if it matches one of:
    /// - A single `return expr;` statement (expression bodies, simple returns)
    /// - Statements with no `return` anywhere (side-effect-only bodies)
    /// - Statements ending with a `return expr;`, where no other `return`
    ///   appears anywhere in the preceding statements
    fn classify_body(stmts: &[StatementListItem]) -> Option<InlinableBody<'_>> {
        if stmts.is_empty() {
            return Some(InlinableBody::StatementsOnly(stmts));
        }

        // Check if the last statement is `return expr;`
        let last_is_return = matches!(
            stmts.last(),
            Some(StatementListItem::Statement(stmt)) if matches!(stmt.as_ref(), Statement::Return(_))
        );

        if last_is_return {
            let StatementListItem::Statement(last_stmt) = &stmts[stmts.len() - 1] else {
                unreachable!();
            };
            let Statement::Return(ret) = last_stmt.as_ref() else {
                unreachable!();
            };

            let preceding = &stmts[..stmts.len() - 1];

            if stmts.len() == 1 {
                // Single `return expr;` — extract the expression directly.
                return ret.target().map(InlinableBody::SingleExpression);
            }

            // Multiple statements ending with return: ensure no `return` in preceding statements.
            if body_contains_return(preceding) {
                return None;
            }

            let Some(return_expr) = ret.target() else {
                // `return;` with no value — treat preceding as statements-only, result is undefined.
                return Some(InlinableBody::StatementsOnly(preceding));
            };

            return Some(InlinableBody::StatementsWithReturn {
                stmts: preceding,
                return_expr,
            });
        }

        // No return at the end. Ensure no `return` anywhere in the body.
        if body_contains_return(stmts) {
            return None;
        }

        Some(InlinableBody::StatementsOnly(stmts))
    }

    /// Compile a function body inline, mapping parameters to argument registers.
    fn inline_function_body(
        &mut self,
        params: &[boa_ast::function::FormalParameter],
        args: &[Expression],
        body: &InlinableBody<'_>,
        scopes: &boa_ast::scope::FunctionScopes,
        dst: &super::Register,
    ) {
        let function_scope = scopes.function_scope();
        // Step 1: Compile argument expressions into registers.
        let mut arg_registers = Vec::new();
        for arg in args {
            let reg = self.register_allocator.alloc();
            self.compile_expr(arg, &reg);
            arg_registers.push(reg);
        }

        // For missing args (fewer args than params), provide undefined.
        while arg_registers.len() < params.len() {
            let reg = self.register_allocator.alloc();
            self.bytecode.emit_push_undefined(reg.variable());
            arg_registers.push(reg);
        }

        // Step 2: Save current scope state.
        let saved_variable_scope = self.variable_scope.clone();
        let saved_lexical_scope = self.lexical_scope.clone();

        // Step 3: Set scope to the function's scope.
        // Since all_bindings_local() is true, no environment push is needed.
        self.variable_scope = function_scope.clone();
        self.lexical_scope = function_scope.clone();

        // Step 4: Map parameter names to argument registers.
        let mut saved_mappings = Vec::new();
        for (i, param) in params.iter().enumerate() {
            let Binding::Identifier(ident) = param.variable().binding() else {
                unreachable!("is_simple() guarantees all params are identifiers");
            };

            let name = self.resolve_identifier_expect(*ident);
            let binding = self.lexical_scope.get_identifier_reference(name);

            let old = self
                .local_binding_registers
                .insert(binding, arg_registers[i].index());
            saved_mappings.push(old);
        }

        // Step 5: Push the body's lexical scope if present (needed for const/let declarations).
        let body_scope = self.push_declarative_scope(scopes.lexical_scope());

        // Step 6: Compile the body.
        match body {
            InlinableBody::SingleExpression(expr) => {
                self.compile_expr(expr, dst);
            }
            InlinableBody::StatementsOnly(stmts) => {
                for item in *stmts {
                    self.compile_stmt_list_item(item, false, false);
                }
                self.bytecode.emit_push_undefined(dst.variable());
            }
            InlinableBody::StatementsWithReturn { stmts, return_expr } => {
                for item in *stmts {
                    self.compile_stmt_list_item(item, false, false);
                }
                self.compile_expr(return_expr, dst);
            }
        }

        // Step 7: Pop the body's lexical scope.
        self.pop_declarative_scope(body_scope);

        // Step 8: Restore parameter bindings.
        for (i, param) in params.iter().enumerate() {
            let Binding::Identifier(ident) = param.variable().binding() else {
                unreachable!();
            };

            let name = self.resolve_identifier_expect(*ident);
            let binding = self.lexical_scope.get_identifier_reference(name);

            if let Some(old_index) = saved_mappings[i] {
                self.local_binding_registers.insert(binding, old_index);
            } else {
                self.local_binding_registers.remove(&binding);
            }
        }

        // Step 9: Restore scopes.
        self.variable_scope = saved_variable_scope;
        self.lexical_scope = saved_lexical_scope;

        // Step 10: Deallocate argument registers in reverse order.
        for reg in arg_registers.into_iter().rev() {
            self.register_allocator.dealloc(reg);
        }
    }
}

/// Returns `true` if any statement in the slice contains a `return` statement
/// at any nesting depth, without descending into nested function bodies.
fn body_contains_return(stmts: &[StatementListItem]) -> bool {
    struct ReturnFinder;

    impl<'ast> Visitor<'ast> for ReturnFinder {
        type BreakTy = ();

        fn visit_return(
            &mut self,
            _node: &'ast boa_ast::statement::Return,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Break(())
        }

        // Don't descend into nested functions — their `return` statements
        // are scoped to the nested function, not the one we're inlining.
        fn visit_function_expression(
            &mut self,
            _node: &'ast FunctionExpression,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_function_declaration(
            &mut self,
            _node: &'ast boa_ast::function::FunctionDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_arrow_function(
            &mut self,
            _node: &'ast ArrowFunction,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_async_arrow_function(
            &mut self,
            _node: &'ast boa_ast::function::AsyncArrowFunction,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_async_function_expression(
            &mut self,
            _node: &'ast boa_ast::function::AsyncFunctionExpression,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_async_function_declaration(
            &mut self,
            _node: &'ast boa_ast::function::AsyncFunctionDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_generator_expression(
            &mut self,
            _node: &'ast boa_ast::function::GeneratorExpression,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_generator_declaration(
            &mut self,
            _node: &'ast boa_ast::function::GeneratorDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_async_generator_expression(
            &mut self,
            _node: &'ast boa_ast::function::AsyncGeneratorExpression,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_async_generator_declaration(
            &mut self,
            _node: &'ast boa_ast::function::AsyncGeneratorDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
    }

    let mut finder = ReturnFinder;
    for stmt in stmts {
        if stmt.visit_with(&mut finder).is_break() {
            return true;
        }
    }
    false
}
