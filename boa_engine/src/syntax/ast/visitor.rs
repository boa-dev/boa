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

use crate::syntax::ast::{Declaration, Statement, StatementList, StatementListItem};

macro_rules! define_visit {
    ($fn_name:ident, $type_name:ident) => {
        #[doc = concat!("Visits a `", stringify!($type_name), "` with this visitor")]
        fn $fn_name(&mut self, node: &'ast $type_name) -> core::ops::ControlFlow<Self::BreakTy> {
            node.visit_with(self)
        }
    };
}

macro_rules! define_visit_mut {
    ($fn_name:ident, $type_name:ident) => {
        #[doc = concat!("Visits a `", stringify!($type_name), "` with this visitor, mutably")]
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
}

/// Denotes that a type may be visited, providing a method which allows a visitor to traverse its
/// private fields.
pub trait VisitWith<V> {
    /// Visit this node with the provided visitor.
    fn visit_with<'a>(&'a self, visitor: &mut V) -> core::ops::ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>;

    /// Visit this node with the provided visitor mutably, allowing the visitor to modify private
    /// fields.
    fn visit_with_mut<'a>(&'a mut self, visitor: &mut V) -> core::ops::ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>;
}
