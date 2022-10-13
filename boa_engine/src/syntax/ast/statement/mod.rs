//! This module implements the `Statement` node.

mod block;
mod r#if;
mod labelled;
mod r#return;
mod throw;

pub mod iteration;
pub mod switch;
pub mod r#try;

use self::iteration::{for_loop::ForLoopInitializer, IterableLoopInitializer};
pub use self::{
    block::Block,
    iteration::{Break, Continue, DoWhileLoop, ForInLoop, ForLoop, ForOfLoop, WhileLoop},
    labelled::{Labelled, LabelledItem},
    r#if::If,
    r#return::Return,
    r#try::{Catch, Finally, Try},
    switch::{Case, Switch},
    throw::Throw,
};

use boa_interner::{Interner, ToInternedString};
use rustc_hash::FxHashSet;

use super::{
    declaration::{Binding, VarDeclaration},
    expression::{Expression, Identifier},
    ContainsSymbol,
};

#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    /// See [`Block`].
    Block(Block),

    /// See [`VarDeclaration`]
    Var(VarDeclaration),

    /// A empty node.
    ///
    /// Empty statement do nothing, just return undefined.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-EmptyStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/Empty
    Empty,

    /// See [`Expression`].
    Expression(Expression),

    /// See [`If`].
    If(If),

    /// See [`DoWhileLoop`].
    DoWhileLoop(DoWhileLoop),

    /// See [`WhileLoop`].
    WhileLoop(WhileLoop),

    /// See [`ForLoop`].
    ForLoop(ForLoop),

    /// See [`ForInLoop`].
    ForInLoop(ForInLoop),

    /// See [`ForOfLoop`].
    ForOfLoop(ForOfLoop),

    /// See[`Switch`].
    Switch(Switch),

    /// See [`Continue`].
    Continue(Continue),

    /// See [`Break`].
    Break(Break),

    /// See [`Return`].
    Return(Return),

    // TODO: Possibly add `with` statements.
    /// See [`Labelled`].
    Labelled(Labelled),

    /// See [`Throw`].
    Throw(Throw),

    /// See [`Try`].
    Try(Try),
}

impl Statement {
    /// Creates a string of the value of the node with the given indentation. For example, an
    /// indent level of 2 would produce this:
    ///
    /// ```js
    ///         function hello() {
    ///             console.log("hello");
    ///         }
    ///         hello();
    ///         a = 2;
    /// ```
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = match *self {
            Self::Block(_) => String::new(),
            _ => "    ".repeat(indentation),
        };

        buf.push_str(&self.to_no_indent_string(interner, indentation));

        buf
    }

    /// Implements the display formatting with indentation.
    ///
    /// This will not prefix the value with any indentation. If you want to prefix this with proper
    /// indents, use [`to_indented_string()`](Self::to_indented_string).
    pub(super) fn to_no_indent_string(&self, interner: &Interner, indentation: usize) -> String {
        match self {
            Self::Block(block) => block.to_indented_string(interner, indentation),
            Self::Var(var) => var.to_interned_string(interner),
            Self::Empty => ";".to_owned(),
            Self::Expression(expr) => expr.to_indented_string(interner, indentation),
            Self::If(if_smt) => if_smt.to_indented_string(interner, indentation),
            Self::DoWhileLoop(do_while) => do_while.to_indented_string(interner, indentation),
            Self::WhileLoop(while_loop) => while_loop.to_indented_string(interner, indentation),
            Self::ForLoop(for_loop) => for_loop.to_indented_string(interner, indentation),
            Self::ForInLoop(for_in) => for_in.to_indented_string(interner, indentation),
            Self::ForOfLoop(for_of) => for_of.to_indented_string(interner, indentation),
            Self::Switch(switch) => switch.to_indented_string(interner, indentation),
            Self::Continue(cont) => cont.to_interned_string(interner),
            Self::Break(break_smt) => break_smt.to_interned_string(interner),
            Self::Return(ret) => ret.to_interned_string(interner),
            Self::Labelled(labelled) => labelled.to_interned_string(interner),
            Self::Throw(throw) => throw.to_interned_string(interner),
            Self::Try(try_catch) => try_catch.to_indented_string(interner, indentation),
        }
    }

    pub(crate) fn var_declared_names(&self, vars: &mut FxHashSet<Identifier>) {
        match self {
            Self::Var(VarDeclaration(list)) => {
                for decl in list.as_ref() {
                    vars.extend(decl.idents());
                }
            }
            Self::Block(block) => {
                for node in block.statement_list().statements() {
                    node.var_declared_names(vars);
                }
            }
            Self::If(if_statement) => {
                if_statement.body().var_declared_names(vars);
                if let Some(node) = if_statement.else_node() {
                    node.var_declared_names(vars);
                }
            }
            Self::DoWhileLoop(do_while_loop) => {
                do_while_loop.body().var_declared_names(vars);
            }
            Self::WhileLoop(while_loop) => {
                while_loop.body().var_declared_names(vars);
            }
            Self::ForLoop(for_loop) => {
                if let Some(ForLoopInitializer::Var(VarDeclaration(list))) = for_loop.init() {
                    for variable in list.as_ref() {
                        match variable.binding() {
                            Binding::Identifier(ident) => {
                                vars.insert(*ident);
                            }
                            Binding::Pattern(pattern) => {
                                for ident in pattern.idents() {
                                    vars.insert(ident);
                                }
                            }
                        }
                    }
                }
                for_loop.body().var_declared_names(vars);
            }
            Self::ForInLoop(for_in_loop) => {
                if let IterableLoopInitializer::Var(bind) = for_in_loop.init() {
                    vars.extend(bind.idents());
                }
                for_in_loop.body().var_declared_names(vars);
            }
            Self::ForOfLoop(for_of_loop) => {
                if let IterableLoopInitializer::Var(bind) = for_of_loop.init() {
                    vars.extend(bind.idents());
                }
                for_of_loop.body().var_declared_names(vars);
            }
            Self::Switch(switch) => {
                for case in switch.cases() {
                    for node in case.body().statements() {
                        node.var_declared_names(vars);
                    }
                }
                if let Some(stmts) = switch.default() {
                    stmts.var_declared_names(vars);
                }
            }
            Self::Try(try_statement) => {
                for node in try_statement.block().statement_list().statements() {
                    node.var_declared_names(vars);
                }
                if let Some(catch) = try_statement.catch() {
                    for node in catch.block().statement_list().statements() {
                        node.var_declared_names(vars);
                    }
                }
                if let Some(finally) = try_statement.finally() {
                    for node in finally.statement_list().statements() {
                        node.var_declared_names(vars);
                    }
                }
            }
            Self::Labelled(labelled) => match labelled.item() {
                LabelledItem::Function(_) => {}
                LabelledItem::Statement(stmt) => stmt.var_declared_names(vars),
            },
            _ => {}
        }
    }

    /// Returns true if the node contains a identifier reference named 'arguments'.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
    // TODO: replace with a visitor
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            Self::Empty => false,
            Self::Block(block) => block.contains_arguments(),
            Self::Var(var) => var.contains_arguments(),
            Self::Expression(expr) => expr.contains_arguments(),
            Self::If(r#if) => r#if.contains_arguments(),
            Self::DoWhileLoop(dowhile) => dowhile.contains_arguments(),
            Self::WhileLoop(whileloop) => whileloop.contains_arguments(),
            Self::ForLoop(forloop) => forloop.contains_arguments(),
            Self::ForInLoop(forin) => forin.contains_arguments(),
            Self::ForOfLoop(forof) => forof.contains_arguments(),
            Self::Switch(switch) => switch.contains_arguments(),
            Self::Continue(r#continue) => r#continue.contains_arguments(),
            Self::Break(r#break) => r#break.contains_arguments(),
            Self::Return(r#return) => r#return.contains_arguments(),
            Self::Labelled(labelled) => labelled.contains_arguments(),
            Self::Throw(throw) => throw.contains_arguments(),
            Self::Try(r#try) => r#try.contains_arguments(),
        }
    }

    /// Returns `true` if the node contains the given token.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
    // TODO: replace with a visitor
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            Self::Empty | Self::Continue(_) | Self::Break(_) => false,
            Self::Block(block) => block.contains(symbol),
            Self::Var(var) => var.contains(symbol),
            Self::Expression(expr) => expr.contains(symbol),
            Self::If(r#if) => r#if.contains(symbol),
            Self::DoWhileLoop(dowhile) => dowhile.contains(symbol),
            Self::WhileLoop(whileloop) => whileloop.contains(symbol),
            Self::ForLoop(forloop) => forloop.contains(symbol),
            Self::ForInLoop(forin) => forin.contains(symbol),
            Self::ForOfLoop(forof) => forof.contains(symbol),
            Self::Switch(switch) => switch.contains(symbol),
            Self::Return(r#return) => r#return.contains(symbol),
            Self::Labelled(labelled) => labelled.contains(symbol),
            Self::Throw(throw) => throw.contains(symbol),
            Self::Try(r#try) => r#try.contains(symbol),
        }
    }
}

impl ToInternedString for Statement {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}
