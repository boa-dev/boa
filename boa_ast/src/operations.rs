//! Definitions of various **Syntax-Directed Operations** used in the [spec].
//!
//! [spec]: https://tc39.es/ecma262/#sec-syntax-directed-operations

use core::ops::ControlFlow;
use std::convert::Infallible;

use boa_interner::Sym;
use rustc_hash::FxHashSet;

use crate::{
    declaration::VarDeclaration,
    expression::{access::SuperPropertyAccess, Await, Identifier, SuperCall, Yield},
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunction, AsyncGenerator, Class, ClassElement,
        Function, Generator,
    },
    property::{MethodDefinition, PropertyDefinition},
    statement::LabelledItem,
    visitor::{NodeRef, VisitWith, Visitor},
    Declaration, Expression, Statement, StatementList, StatementListItem,
};

/// Represents all the possible symbols searched for by the [`Contains`][contains] operation.
///
/// [contains]: https://tc39.es/ecma262/#sec-syntax-directed-operations-contains
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContainsSymbol {
    /// A node with the `super` keyword (`super(args)` or `super.prop`).
    Super,
    /// A super property access (`super.prop`).
    SuperProperty,
    /// A super constructor call (`super(args)`).
    SuperCall,
    /// A yield expression (`yield 5`).
    YieldExpression,
    /// An await expression (`await 4`).
    AwaitExpression,
    /// The new target expression (`new.target`).
    NewTarget,
    /// The body of a class definition.
    ClassBody,
    /// The super class of a class definition.
    ClassHeritage,
    /// A this expression (`this`).
    This,
    /// A method definition.
    MethodDefinition,
}

/// Returns `true` if the node contains the given symbol.
///
/// This is equivalent to the [`Contains`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
#[must_use]
#[inline]
pub fn contains<N>(node: &N, symbol: ContainsSymbol) -> bool
where
    N: VisitWith,
{
    /// Visitor used by the function to search for a specific symbol in a node.
    #[derive(Debug, Clone, Copy)]
    struct ContainsVisitor(ContainsSymbol);

    impl<'ast> Visitor<'ast> for ContainsVisitor {
        type BreakTy = ();

        #[inline]
        fn visit_function(&mut self, _: &'ast Function) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        #[inline]
        fn visit_async_function(&mut self, _: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        #[inline]
        fn visit_generator(&mut self, _: &'ast Generator) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        #[inline]
        fn visit_async_generator(&mut self, _: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        #[inline]
        fn visit_class(&mut self, node: &'ast Class) -> ControlFlow<Self::BreakTy> {
            if !node.elements().is_empty() && self.0 == ContainsSymbol::ClassBody {
                return ControlFlow::Break(());
            }

            if node.super_ref().is_some() && self.0 == ContainsSymbol::ClassHeritage {
                return ControlFlow::Break(());
            }

            node.visit_with(self)
        }

        // `ComputedPropertyContains`: https://tc39.es/ecma262/#sec-static-semantics-computedpropertycontains
        #[inline]
        fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
            match node {
                ClassElement::MethodDefinition(name, _)
                | ClassElement::StaticMethodDefinition(name, _)
                | ClassElement::FieldDefinition(name, _)
                | ClassElement::StaticFieldDefinition(name, _) => name.visit_with(self),
                _ => ControlFlow::Continue(()),
            }
        }

        #[inline]
        fn visit_property_definition(
            &mut self,
            node: &'ast PropertyDefinition,
        ) -> ControlFlow<Self::BreakTy> {
            if let PropertyDefinition::MethodDefinition(name, _) = node {
                if self.0 == ContainsSymbol::MethodDefinition {
                    return ControlFlow::Break(());
                }
                return name.visit_with(self);
            }

            node.visit_with(self)
        }

        #[inline]
        fn visit_arrow_function(
            &mut self,
            node: &'ast ArrowFunction,
        ) -> ControlFlow<Self::BreakTy> {
            if ![
                ContainsSymbol::NewTarget,
                ContainsSymbol::SuperProperty,
                ContainsSymbol::SuperCall,
                ContainsSymbol::Super,
                ContainsSymbol::This,
            ]
            .contains(&self.0)
            {
                return ControlFlow::Continue(());
            }

            node.visit_with(self)
        }

        #[inline]
        fn visit_async_arrow_function(
            &mut self,
            node: &'ast AsyncArrowFunction,
        ) -> ControlFlow<Self::BreakTy> {
            if ![
                ContainsSymbol::NewTarget,
                ContainsSymbol::SuperProperty,
                ContainsSymbol::SuperCall,
                ContainsSymbol::Super,
                ContainsSymbol::This,
            ]
            .contains(&self.0)
            {
                return ControlFlow::Continue(());
            }

            node.visit_with(self)
        }

        #[inline]
        fn visit_super_property_access(
            &mut self,
            node: &'ast SuperPropertyAccess,
        ) -> ControlFlow<Self::BreakTy> {
            if [ContainsSymbol::SuperProperty, ContainsSymbol::Super].contains(&self.0) {
                return ControlFlow::Break(());
            }
            node.visit_with(self)
        }

        #[inline]
        fn visit_super_call(&mut self, node: &'ast SuperCall) -> ControlFlow<Self::BreakTy> {
            if [ContainsSymbol::SuperCall, ContainsSymbol::Super].contains(&self.0) {
                return ControlFlow::Break(());
            }
            node.visit_with(self)
        }

        #[inline]
        fn visit_yield(&mut self, node: &'ast Yield) -> ControlFlow<Self::BreakTy> {
            if self.0 == ContainsSymbol::YieldExpression {
                return ControlFlow::Break(());
            }

            node.visit_with(self)
        }

        #[inline]
        fn visit_await(&mut self, node: &'ast Await) -> ControlFlow<Self::BreakTy> {
            if self.0 == ContainsSymbol::AwaitExpression {
                return ControlFlow::Break(());
            }

            node.visit_with(self)
        }

        #[inline]
        fn visit_expression(&mut self, node: &'ast Expression) -> ControlFlow<Self::BreakTy> {
            if node == &Expression::This && self.0 == ContainsSymbol::This {
                return ControlFlow::Break(());
            }
            if node == &Expression::NewTarget && self.0 == ContainsSymbol::NewTarget {
                return ControlFlow::Break(());
            }
            node.visit_with(self)
        }
    }

    node.visit_with(&mut ContainsVisitor(symbol)).is_break()
}

/// Returns true if the node contains an identifier reference with name `arguments`.
///
/// This is equivalent to the [`ContainsArguments`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
#[must_use]
pub fn contains_arguments<N>(node: &N) -> bool
where
    N: VisitWith,
{
    /// Visitor used by the function to search for an identifier with the name `arguments`.
    #[derive(Debug, Clone, Copy)]
    struct ContainsArgsVisitor;

    impl<'ast> Visitor<'ast> for ContainsArgsVisitor {
        type BreakTy = ();

        #[inline]
        fn visit_identifier(&mut self, node: &'ast Identifier) -> ControlFlow<Self::BreakTy> {
            if node.sym() == Sym::ARGUMENTS {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }

        #[inline]
        fn visit_function(&mut self, _: &'ast Function) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        #[inline]
        fn visit_async_function(&mut self, _: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        #[inline]
        fn visit_generator(&mut self, _: &'ast Generator) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        #[inline]
        fn visit_async_generator(&mut self, _: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        #[inline]
        fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
            match node {
                ClassElement::MethodDefinition(name, _)
                | ClassElement::StaticMethodDefinition(name, _) => return name.visit_with(self),
                _ => {}
            }
            node.visit_with(self)
        }

        #[inline]
        fn visit_property_definition(
            &mut self,
            node: &'ast PropertyDefinition,
        ) -> ControlFlow<Self::BreakTy> {
            if let PropertyDefinition::MethodDefinition(name, _) = node {
                name.visit_with(self)
            } else {
                node.visit_with(self)
            }
        }
    }
    node.visit_with(&mut ContainsArgsVisitor).is_break()
}

/// Returns `true` if `method` has a super call in its parameters or body.
///
/// This is equivalent to the [`HasDirectSuper`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-hasdirectsuper
#[must_use]
#[inline]
pub fn has_direct_super(method: &MethodDefinition) -> bool {
    match method {
        MethodDefinition::Get(f) | MethodDefinition::Set(f) | MethodDefinition::Ordinary(f) => {
            contains(f, ContainsSymbol::SuperCall)
        }
        MethodDefinition::Generator(f) => contains(f, ContainsSymbol::SuperCall),
        MethodDefinition::AsyncGenerator(f) => contains(f, ContainsSymbol::SuperCall),
        MethodDefinition::Async(f) => contains(f, ContainsSymbol::SuperCall),
    }
}

/// A container that [`BoundNamesVisitor`] can use to push the found identifiers.
trait IdentList {
    fn add(&mut self, value: Identifier, function: bool);
}

impl IdentList for Vec<Identifier> {
    #[inline]
    fn add(&mut self, value: Identifier, _function: bool) {
        self.push(value);
    }
}

impl IdentList for Vec<(Identifier, bool)> {
    #[inline]
    fn add(&mut self, value: Identifier, function: bool) {
        self.push((value, function));
    }
}

impl IdentList for FxHashSet<Identifier> {
    #[inline]
    fn add(&mut self, value: Identifier, _function: bool) {
        self.insert(value);
    }
}

/// The [`Visitor`] used to obtain the bound names of a node.
#[derive(Debug)]
struct BoundNamesVisitor<'a, T: IdentList>(&'a mut T);

impl<'ast, T: IdentList> Visitor<'ast> for BoundNamesVisitor<'_, T> {
    type BreakTy = Infallible;

    #[inline]
    fn visit_identifier(&mut self, node: &'ast Identifier) -> ControlFlow<Self::BreakTy> {
        self.0.add(*node, false);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_expression(&mut self, _: &'ast Expression) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }
    // TODO: add "*default" for module default functions without name
    #[inline]
    fn visit_function(&mut self, node: &'ast Function) -> ControlFlow<Self::BreakTy> {
        if let Some(ident) = node.name() {
            self.0.add(ident, true);
        }
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_generator(&mut self, node: &'ast Generator) -> ControlFlow<Self::BreakTy> {
        if let Some(ident) = node.name() {
            self.0.add(ident, false);
        }
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_async_function(&mut self, node: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
        if let Some(ident) = node.name() {
            self.0.add(ident, false);
        }
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_async_generator(&mut self, node: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
        if let Some(ident) = node.name() {
            self.0.add(ident, false);
        }
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_class(&mut self, node: &'ast Class) -> ControlFlow<Self::BreakTy> {
        if let Some(ident) = node.name() {
            self.0.add(ident, false);
        }
        ControlFlow::Continue(())
    }
}

/// Returns a list with the bound names of an AST node, which may contain duplicates.
///
/// This is equivalent to the [`BoundNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-boundnames
#[must_use]
#[inline]
pub fn bound_names<'a, N>(node: &'a N) -> Vec<Identifier>
where
    &'a N: Into<NodeRef<'a>>,
{
    let mut names = Vec::new();
    BoundNamesVisitor(&mut names).visit(node.into());

    names
}

/// The [`Visitor`] used to obtain the lexically declared names of a node.
#[derive(Debug)]
struct LexicallyDeclaredNamesVisitor<'a, T: IdentList>(&'a mut T);

impl<'ast, T: IdentList> Visitor<'ast> for LexicallyDeclaredNamesVisitor<'_, T> {
    type BreakTy = Infallible;
    #[inline]
    fn visit_expression(&mut self, _: &'ast Expression) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        if let Statement::Labelled(labelled) = node {
            return self.visit_labelled(labelled);
        }
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_declaration(&mut self, node: &'ast Declaration) -> ControlFlow<Self::BreakTy> {
        BoundNamesVisitor(self.0).visit_declaration(node)
    }
    #[inline]
    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        match node {
            LabelledItem::Function(f) => BoundNamesVisitor(self.0).visit_function(f),
            LabelledItem::Statement(_) => ControlFlow::Continue(()),
        }
    }
    #[inline]
    fn visit_function(&mut self, node: &'ast Function) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_async_function(&mut self, node: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_generator(&mut self, node: &'ast Generator) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_async_generator(&mut self, node: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_arrow_function(&mut self, node: &'ast ArrowFunction) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_async_arrow_function(
        &mut self,
        node: &'ast AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
        if let ClassElement::StaticBlock(stmts) = node {
            top_level_lexicals(stmts, self.0);
        }
        ControlFlow::Continue(())
    }

    // TODO:  ScriptBody : StatementList
    // 1. Return TopLevelLexicallyDeclaredNames of StatementList.
    // But we don't have that node yet. In the meantime, use `top_level_lexically_declared_names` directly.
}

/// Returns a list with the lexical bindings of a node, which may contain duplicates.
///
/// This is equivalent to the [`LexicallyDeclaredNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-lexicallydeclarednames
#[must_use]
#[inline]
pub fn lexically_declared_names<'a, N>(node: &'a N) -> Vec<Identifier>
where
    &'a N: Into<NodeRef<'a>>,
{
    let mut names = Vec::new();
    LexicallyDeclaredNamesVisitor(&mut names).visit(node.into());
    names
}

/// Returns a list with the lexical bindings of a node, which may contain duplicates.
///
/// If a declared name originates from a function declaration it is flagged as `true` in the returned
/// list. (See [B.3.2.4 Changes to Block Static Semantics: Early Errors])
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-lexicallydeclarednames
/// [changes]: https://tc39.es/ecma262/#sec-block-duplicates-allowed-static-semantics
#[must_use]
#[inline]
pub fn lexically_declared_names_legacy<'a, N>(node: &'a N) -> Vec<(Identifier, bool)>
where
    &'a N: Into<NodeRef<'a>>,
{
    let mut names = Vec::new();
    LexicallyDeclaredNamesVisitor(&mut names).visit(node.into());
    names
}

/// The [`Visitor`] used to obtain the var declared names of a node.
#[derive(Debug)]
struct VarDeclaredNamesVisitor<'a>(&'a mut FxHashSet<Identifier>);

impl<'ast> Visitor<'ast> for VarDeclaredNamesVisitor<'_> {
    type BreakTy = Infallible;
    #[inline]
    fn visit_expression(&mut self, _: &'ast Expression) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_declaration(&mut self, _: &'ast Declaration) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_var_declaration(&mut self, node: &'ast VarDeclaration) -> ControlFlow<Self::BreakTy> {
        BoundNamesVisitor(self.0).visit_var_declaration(node)
    }
    #[inline]
    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        match node {
            LabelledItem::Function(_) => ControlFlow::Continue(()),
            LabelledItem::Statement(stmt) => stmt.visit_with(self),
        }
    }
    #[inline]
    fn visit_function(&mut self, node: &'ast Function) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_async_function(&mut self, node: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_generator(&mut self, node: &'ast Generator) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_async_generator(&mut self, node: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_arrow_function(&mut self, node: &'ast ArrowFunction) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_async_arrow_function(
        &mut self,
        node: &'ast AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }
    #[inline]
    fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
        if let ClassElement::StaticBlock(stmts) = node {
            top_level_vars(stmts, self.0);
        }
        node.visit_with(self)
    }

    // TODO:  ScriptBody : StatementList
    // 1. Return TopLevelVarDeclaredNames of StatementList.
    // But we don't have that node yet. In the meantime, use `top_level_var_declared_names` directly.
}

/// Returns a set with the var bindings of a node, with no duplicates.
///
/// This is equivalent to the [`VarDeclaredNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-vardeclarednames
#[must_use]
#[inline]
pub fn var_declared_names<'a, N>(node: &'a N) -> FxHashSet<Identifier>
where
    &'a N: Into<NodeRef<'a>>,
{
    let mut names = FxHashSet::default();
    VarDeclaredNamesVisitor(&mut names).visit(node.into());
    names
}

/// Utility function that collects the top level lexicals of a statement list into `names`.
#[inline]
fn top_level_lexicals<T: IdentList>(stmts: &StatementList, names: &mut T) {
    for stmt in stmts.statements() {
        if let StatementListItem::Declaration(decl) = stmt {
            match decl {
                // Note
                // At the top level of a function, or script, function declarations are treated like
                // var declarations rather than like lexical declarations.
                Declaration::Function(_)
                | Declaration::Generator(_)
                | Declaration::AsyncFunction(_)
                | Declaration::AsyncGenerator(_) => {}
                Declaration::Class(class) => {
                    BoundNamesVisitor(names).visit_class(class);
                }
                Declaration::Lexical(decl) => {
                    BoundNamesVisitor(names).visit_lexical_declaration(decl);
                }
            }
        }
    }
}

/// Returns a list with the lexical bindings of a top-level statement list, which may contain duplicates.
///
/// This is equivalent to the [`TopLevelLexicallyDeclaredNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-toplevellexicallydeclarednames
#[must_use]
#[inline]
pub fn top_level_lexically_declared_names(stmts: &StatementList) -> Vec<Identifier> {
    let mut names = Vec::new();
    top_level_lexicals(stmts, &mut names);
    names
}

/// Utility function that collects the top level vars of a statement list into `names`.
#[inline]
fn top_level_vars(stmts: &StatementList, names: &mut FxHashSet<Identifier>) {
    for stmt in stmts.statements() {
        match stmt {
            StatementListItem::Declaration(decl) => {
                match decl {
                    // Note
                    // At the top level of a function, or script, function declarations are treated like
                    // var declarations rather than like lexical declarations.
                    Declaration::Function(f) => {
                        BoundNamesVisitor(names).visit_function(f);
                    }
                    Declaration::Generator(f) => {
                        BoundNamesVisitor(names).visit_generator(f);
                    }
                    Declaration::AsyncFunction(f) => {
                        BoundNamesVisitor(names).visit_async_function(f);
                    }
                    Declaration::AsyncGenerator(f) => {
                        BoundNamesVisitor(names).visit_async_generator(f);
                    }
                    Declaration::Class(_) | Declaration::Lexical(_) => {}
                }
            }
            StatementListItem::Statement(stmt) => {
                let mut stmt = Some(stmt);
                while let Some(Statement::Labelled(labelled)) = stmt {
                    match labelled.item() {
                        LabelledItem::Function(f) => {
                            BoundNamesVisitor(names).visit_function(f);
                            stmt = None;
                        }
                        LabelledItem::Statement(s) => stmt = Some(s),
                    }
                }
                if let Some(stmt) = stmt {
                    VarDeclaredNamesVisitor(names).visit(stmt);
                }
            }
        }
    }
}

/// Returns a list with the var bindings of a top-level statement list, with no duplicates.
///
/// This is equivalent to the [`TopLevelVarDeclaredNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-toplevelvardeclarednames
#[must_use]
#[inline]
pub fn top_level_var_declared_names(stmts: &StatementList) -> FxHashSet<Identifier> {
    let mut names = FxHashSet::default();
    top_level_vars(stmts, &mut names);
    names
}
