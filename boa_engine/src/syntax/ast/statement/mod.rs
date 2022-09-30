//! This module implements the `Node` structure, which composes the AST.

mod block;
mod r#if;
mod r#return;
mod throw;

pub mod declaration;
pub mod iteration;
pub mod statement_list;
pub mod switch;
pub mod r#try;

pub use self::{
    block::Block,
    iteration::{Break, Continue, DoWhileLoop, ForInLoop, ForLoop, ForOfLoop, WhileLoop},
    r#if::If,
    r#return::Return,
    r#try::{Catch, Finally, Try},
    statement_list::StatementList,
    switch::{Case, Switch},
    throw::Throw,
};
use self::{
    declaration::{Binding, Declaration, DeclarationList},
    iteration::{for_loop::ForLoopInitializer, IterableLoopInitializer},
};

use boa_interner::{Interner, Sym, ToInternedString};
use rustc_hash::FxHashSet;
use std::cmp::Ordering;

use super::{
    expression::Expression,
    function::{AsyncFunction, AsyncGenerator, Class, ClassElement, Function, Generator},
    ContainsSymbol,
};

// TODO: This should be split into Expression and Statement.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    /// See [`Block`].
    Block(Block),

    /// See [`DeclarationList`]
    DeclarationList(DeclarationList),

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

    // TODO: extract labels into a `LabelledStatement`
    /// See [`Throw`].
    Throw(Throw),

    /// See [`Try`].
    Try(Try),

    /// See [`Function`]
    Function(Function),

    /// See [`Generator`]
    Generator(Generator),

    /// See [`AsyncFunction`]
    AsyncFunction(AsyncFunction),

    /// See [`AsyncGenerator`]
    AsyncGenerator(AsyncGenerator),

    /// See [`Class`]
    Class(Class),
}

impl Statement {
    /// Returns a node ordering based on the hoistability of each statement.
    pub(crate) fn hoistable_order(a: &Self, b: &Self) -> Ordering {
        match (a, b) {
            (Statement::Function(_), Statement::Function(_)) => Ordering::Equal,
            (_, Statement::Function(_)) => Ordering::Greater,
            (Statement::Function(_), _) => Ordering::Less,

            (_, _) => Ordering::Equal,
        }
    }

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
    fn to_no_indent_string(&self, interner: &Interner, indentation: usize) -> String {
        match *self {
            Self::Block(ref block) => block.to_indented_string(interner, indentation),
            Self::DeclarationList(ref list) => list.to_interned_string(interner),
            Self::Empty => ";".to_owned(),
            Self::Expression(ref expr) => expr.to_indented_string(interner, indentation),
            Self::If(ref if_smt) => if_smt.to_indented_string(interner, indentation),
            Self::DoWhileLoop(ref do_while) => do_while.to_indented_string(interner, indentation),
            Self::WhileLoop(ref while_loop) => while_loop.to_indented_string(interner, indentation),
            Self::ForLoop(ref for_loop) => for_loop.to_indented_string(interner, indentation),
            Self::ForInLoop(ref for_in) => for_in.to_indented_string(interner, indentation),
            Self::ForOfLoop(ref for_of) => for_of.to_indented_string(interner, indentation),
            Self::Switch(ref switch) => switch.to_indented_string(interner, indentation),
            Self::Continue(ref cont) => cont.to_interned_string(interner),
            Self::Break(ref break_smt) => break_smt.to_interned_string(interner),
            Self::Return(ref ret) => ret.to_interned_string(interner),
            Self::Throw(ref throw) => throw.to_interned_string(interner),
            Self::Try(ref try_catch) => try_catch.to_indented_string(interner, indentation),
            Self::Function(ref decl) => decl.to_indented_string(interner, indentation),
            Self::Generator(ref decl) => decl.to_interned_string(interner),
            Self::AsyncFunction(ref decl) => decl.to_indented_string(interner, indentation),
            Self::AsyncGenerator(ref decl) => decl.to_indented_string(interner, indentation),
            Self::Class(ref decl) => decl.to_indented_string(interner, indentation),
        }
    }

    pub(crate) fn var_declared_names(&self, vars: &mut FxHashSet<Sym>) {
        match self {
            Statement::DeclarationList(DeclarationList::Var(list)) => {
                for decl in &**list {
                    vars.extend(decl.idents());
                }
            }
            Statement::Block(block) => {
                for node in block.statements() {
                    node.var_declared_names(vars);
                }
            }
            Statement::If(if_statement) => {
                if_statement.body().var_declared_names(vars);
                if let Some(node) = if_statement.else_node() {
                    node.var_declared_names(vars);
                }
            }
            Statement::DoWhileLoop(do_while_loop) => {
                do_while_loop.body().var_declared_names(vars);
            }
            Statement::WhileLoop(while_loop) => {
                while_loop.body().var_declared_names(vars);
            }
            Statement::ForLoop(for_loop) => {
                if let Some(ForLoopInitializer::DeclarationList(DeclarationList::Var(
                    declarations,
                ))) = for_loop.init()
                {
                    for declaration in declarations.iter() {
                        match declaration.binding() {
                            Binding::Identifier(ident) => {
                                vars.insert(ident.sym());
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
            Statement::ForInLoop(for_in_loop) => {
                if let IterableLoopInitializer::Var(bind) = for_in_loop.init() {
                    vars.extend(bind.idents());
                }
                for_in_loop.body().var_declared_names(vars);
            }
            Statement::ForOfLoop(for_of_loop) => {
                if let IterableLoopInitializer::Var(bind) = for_of_loop.init() {
                    vars.extend(bind.idents());
                }
                for_of_loop.body().var_declared_names(vars);
            }
            Statement::Switch(switch) => {
                for case in switch.cases() {
                    for node in case.body().statements() {
                        node.var_declared_names(vars);
                    }
                }
                if let Some(stmts) = switch.default() {
                    stmts.var_declared_names(vars);
                }
            }
            Statement::Try(try_statement) => {
                for node in try_statement.block().statements() {
                    node.var_declared_names(vars);
                }
                if let Some(catch) = try_statement.catch() {
                    for node in catch.block().statements() {
                        node.var_declared_names(vars);
                    }
                }
                if let Some(finally) = try_statement.finally() {
                    for node in finally.statements() {
                        node.var_declared_names(vars);
                    }
                }
            }
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
            Statement::Expression(expr) => expr.contains_arguments(),

            Statement::Block(block) => block.statements().iter().any(Statement::contains_arguments),
            Statement::DoWhileLoop(do_while_loop) => {
                do_while_loop.body().contains_arguments()
                    || do_while_loop.cond().contains_arguments()
            }
            Statement::ForLoop(for_loop) => {
                matches!(for_loop.init(), Some(expr) if expr.contains_arguments())
                    || matches!(for_loop.condition(), Some(expr) if expr.contains_arguments())
                    || matches!(for_loop.final_expr(), Some(expr) if expr.contains_arguments())
                    || for_loop.body().contains_arguments()
            }
            Statement::ForInLoop(for_in_loop) => {
                for_in_loop.init().contains_arguments()
                    || for_in_loop.expr().contains_arguments()
                    || for_in_loop.body().contains_arguments()
            }
            Statement::ForOfLoop(for_of_loop) => {
                for_of_loop.init().contains_arguments()
                    || for_of_loop.iterable().contains_arguments()
                    || for_of_loop.body().contains_arguments()
            }
            Statement::If(r#if) => {
                r#if.cond().contains_arguments()
                    || r#if.body().contains_arguments()
                    || matches!(r#if.else_node(), Some(node) if node.contains_arguments())
            }

            Statement::DeclarationList(decl_list) => decl_list
                .as_ref()
                .iter()
                .any(Declaration::contains_arguments),
            Statement::Return(r#return) => {
                matches!(r#return.expr(), Some(expr) if expr.contains_arguments())
            }
            Statement::Switch(switch) => {
                switch.val().contains_arguments()
                    || switch.cases().iter().any(Case::contains_arguments)
            }
            Statement::Throw(throw) => throw.expr().contains_arguments(),
            Statement::Try(r#try) => r#try.contains_arguments(),
            Statement::WhileLoop(while_loop) => {
                while_loop.condition().contains_arguments()
                    || while_loop.body().contains_arguments()
            }
            Statement::Class(class) => {
                matches!(class.super_ref(), Some(expr) if expr.contains_arguments())
                    || class
                        .elements()
                        .iter()
                        .any(ClassElement::contains_arguments)
            }
            _ => false,
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
            Statement::Expression(expr) => expr.contains(symbol),

            Statement::Block(block) => block.statements().iter().any(|stmt| stmt.contains(symbol)),
            Statement::DoWhileLoop(do_while_loop) => {
                do_while_loop.body().contains(symbol) || do_while_loop.cond().contains(symbol)
            }
            Statement::ForLoop(for_loop) => {
                matches!(for_loop.init(), Some(expr) if expr.contains(symbol))
                    || matches!(for_loop.condition(), Some(expr) if expr.contains(symbol))
                    || matches!(for_loop.final_expr(), Some(expr) if expr.contains(symbol))
                    || for_loop.body().contains(symbol)
            }
            Statement::ForInLoop(for_in_loop) => {
                for_in_loop.init().contains(symbol)
                    || for_in_loop.expr().contains(symbol)
                    || for_in_loop.body().contains(symbol)
            }
            Statement::ForOfLoop(for_of_loop) => {
                for_of_loop.init().contains(symbol)
                    || for_of_loop.iterable().contains(symbol)
                    || for_of_loop.body().contains(symbol)
            }
            Statement::If(r#if) => {
                r#if.cond().contains(symbol)
                    || r#if.body().contains(symbol)
                    || matches!(r#if.else_node(), Some(expr) if expr.contains(symbol))
            }

            Statement::DeclarationList(decl_list) => {
                decl_list.as_ref().iter().any(|decl| decl.contains(symbol))
            }
            Statement::Return(r#return) => {
                matches!(r#return.expr(), Some(expr) if expr.contains(symbol))
            }
            Statement::Switch(switch) => {
                switch.val().contains(symbol)
                    || switch.cases().iter().any(|case| case.contains(symbol))
            }
            Statement::Throw(throw) => throw.expr().contains(symbol),
            Statement::Try(r#try) => r#try.contains(symbol),
            Statement::WhileLoop(while_loop) => {
                while_loop.condition().contains(symbol) || while_loop.body().contains(symbol)
            }
            Statement::Class(class) => {
                matches!(class.super_ref(), Some(expr) if expr.contains(symbol))
                    || class.elements().iter().any(|elem| elem.contains(symbol))
            }
            _ => false,
        }
    }
}

impl ToInternedString for Statement {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}
