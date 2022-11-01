//! Javascript Abstract Syntax Tree visitors.
//!
//! This module contains visitors which can be used to inspect or modify AST nodes. This allows for
//! fine-grained manipulation of ASTs for analysis, rewriting, or instrumentation.

/// `Try`-like conditional unwrapping of `ControlFlow`.
#[macro_export]
macro_rules! try_break {
    ($expr:expr) => {
        match $expr {
            core::ops::ControlFlow::Continue(c) => c,
            core::ops::ControlFlow::Break(b) => return core::ops::ControlFlow::Break(b),
        }
    };
}

use crate::syntax::ast::declaration::*;
use crate::syntax::ast::expression::*;
use crate::syntax::ast::function::*;
use crate::syntax::ast::pattern::*;
use crate::syntax::ast::statement::*;
use crate::syntax::ast::*;

/// Creates the default visit function implementation for a particular type
macro_rules! define_visit {
    ($fn_name:ident, $type_name:ident) => {
        #[doc = concat!("Visits a `", stringify!($type_name), "` with this visitor")]
        #[must_use]
        fn $fn_name(&mut self, node: &'ast $type_name) -> core::ops::ControlFlow<Self::BreakTy> {
            node.visit_with(self)
        }
    };
}

/// Creates the default mutable visit function implementation for a particular type
macro_rules! define_visit_mut {
    ($fn_name:ident, $type_name:ident) => {
        #[doc = concat!("Visits a `", stringify!($type_name), "` with this visitor, mutably")]
        #[must_use]
        fn $fn_name(
            &mut self,
            node: &'ast mut $type_name,
        ) -> core::ops::ControlFlow<Self::BreakTy> {
            node.visit_with_mut(self)
        }
    };
}

/// Represents an AST visitor.
///
/// This implementation is based largely on [chalk](https://github.com/rust-lang/chalk/blob/23d39c90ceb9242fbd4c43e9368e813e7c2179f7/chalk-ir/src/visit.rs)'s
/// visitor pattern.
#[allow(unused_variables)]
pub trait Visitor<'ast>: Sized {
    /// Type which will be propagated from the visitor if completing early.
    type BreakTy;

    define_visit!(visit_statement_list, StatementList);
    define_visit!(visit_statement_list_item, StatementListItem);
    define_visit!(visit_statement, Statement);
    define_visit!(visit_declaration, Declaration);
    define_visit!(visit_function, Function);
    define_visit!(visit_generator, Generator);
    define_visit!(visit_async_function, AsyncFunction);
    define_visit!(visit_async_generator, AsyncGenerator);
    define_visit!(visit_class, Class);
    define_visit!(visit_lexical_declaration, LexicalDeclaration);
    define_visit!(visit_block, Block);
    define_visit!(visit_var_declaration, VarDeclaration);
    define_visit!(visit_expression, Expression);
    define_visit!(visit_if, If);
    define_visit!(visit_do_while_loop, DoWhileLoop);
    define_visit!(visit_while_loop, WhileLoop);
    define_visit!(visit_for_loop, ForLoop);
    define_visit!(visit_for_in_loop, ForInLoop);
    define_visit!(visit_for_of_loop, ForOfLoop);
    define_visit!(visit_switch, Switch);
    define_visit!(visit_continue, Continue);
    define_visit!(visit_break, Break);
    define_visit!(visit_return, Return);
    define_visit!(visit_labelled, Labelled);
    define_visit!(visit_throw, Throw);
    define_visit!(visit_try, Try);
    define_visit!(visit_identifier, Identifier);
    define_visit!(visit_formal_parameter_list, FormalParameterList);
    define_visit!(visit_class_element, ClassElement);
    define_visit!(visit_variable_list, VariableList);
    define_visit!(visit_variable, Variable);
    define_visit!(visit_binding, Binding);
    define_visit!(visit_pattern, Pattern);
}

/// Represents an AST visitor which can modify AST content.
///
/// This implementation is based largely on [chalk](https://github.com/rust-lang/chalk/blob/23d39c90ceb9242fbd4c43e9368e813e7c2179f7/chalk-ir/src/visit.rs)'s
/// visitor pattern.
#[allow(unused_variables)]
pub trait VisitorMut<'ast>: Sized {
    /// Type which will be propagated from the visitor if completing early.
    type BreakTy;

    define_visit_mut!(visit_statement_list_mut, StatementList);
    define_visit_mut!(visit_statement_list_item_mut, StatementListItem);
    define_visit_mut!(visit_statement_mut, Statement);
    define_visit_mut!(visit_declaration_mut, Declaration);
    define_visit_mut!(visit_function_mut, Function);
    define_visit_mut!(visit_generator_mut, Generator);
    define_visit_mut!(visit_async_function_mut, AsyncFunction);
    define_visit_mut!(visit_async_generator_mut, AsyncGenerator);
    define_visit_mut!(visit_class_mut, Class);
    define_visit_mut!(visit_lexical_declaration_mut, LexicalDeclaration);
    define_visit_mut!(visit_block_mut, Block);
    define_visit_mut!(visit_var_declaration_mut, VarDeclaration);
    define_visit_mut!(visit_expression_mut, Expression);
    define_visit_mut!(visit_if_mut, If);
    define_visit_mut!(visit_do_while_loop_mut, DoWhileLoop);
    define_visit_mut!(visit_while_loop_mut, WhileLoop);
    define_visit_mut!(visit_for_loop_mut, ForLoop);
    define_visit_mut!(visit_for_in_loop_mut, ForInLoop);
    define_visit_mut!(visit_for_of_loop_mut, ForOfLoop);
    define_visit_mut!(visit_switch_mut, Switch);
    define_visit_mut!(visit_continue_mut, Continue);
    define_visit_mut!(visit_break_mut, Break);
    define_visit_mut!(visit_return_mut, Return);
    define_visit_mut!(visit_labelled_mut, Labelled);
    define_visit_mut!(visit_throw_mut, Throw);
    define_visit_mut!(visit_try_mut, Try);
    define_visit_mut!(visit_identifier_mut, Identifier);
    define_visit_mut!(visit_formal_parameter_list_mut, FormalParameterList);
    define_visit_mut!(visit_class_element_mut, ClassElement);
    define_visit_mut!(visit_variable_list_mut, VariableList);
    define_visit_mut!(visit_variable_mut, Variable);
    define_visit_mut!(visit_binding_mut, Binding);
    define_visit_mut!(visit_pattern_mut, Pattern);
}

/// Denotes that a type may be visited, providing a method which allows a visitor to traverse its
/// private fields.
pub trait VisitWith {
    /// Visit this node with the provided visitor.
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> core::ops::ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>;

    /// Visit this node with the provided visitor mutably, allowing the visitor to modify private
    /// fields.
    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> core::ops::ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>;
}
