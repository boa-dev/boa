//! Definitions of various **Syntax-Directed Operations** used in the [spec].
//!
//! [spec]: https://tc39.es/ecma262/#sec-syntax-directed-operations

use core::ops::ControlFlow;
use std::convert::Infallible;

use boa_interner::{Interner, Sym};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    declaration::{ExportDeclaration, ImportDeclaration, VarDeclaration},
    expression::{access::SuperPropertyAccess, Await, Identifier, SuperCall, Yield},
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunction, AsyncGenerator, Class, ClassElement,
        Function, Generator, PrivateName,
    },
    property::{MethodDefinition, PropertyDefinition},
    statement::LabelledItem,
    try_break,
    visitor::{NodeRef, VisitWith, Visitor, VisitorMut},
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
    /// The BindingIdentifier "eval" or "arguments".
    EvalOrArguments,
}

/// Returns `true` if the node contains the given symbol.
///
/// This is equivalent to the [`Contains`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
#[must_use]
pub fn contains<N>(node: &N, symbol: ContainsSymbol) -> bool
where
    N: VisitWith,
{
    /// Visitor used by the function to search for a specific symbol in a node.
    #[derive(Debug, Clone, Copy)]
    struct ContainsVisitor(ContainsSymbol);

    impl<'ast> Visitor<'ast> for ContainsVisitor {
        type BreakTy = ();

        fn visit_identifier(&mut self, node: &'ast Identifier) -> ControlFlow<Self::BreakTy> {
            if self.0 == ContainsSymbol::EvalOrArguments
                && (node.sym() == Sym::EVAL || node.sym() == Sym::ARGUMENTS)
            {
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }

        fn visit_function(&mut self, _: &'ast Function) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_async_function(&mut self, _: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_generator(&mut self, _: &'ast Generator) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_async_generator(&mut self, _: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

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
        fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
            match node {
                ClassElement::MethodDefinition(name, _)
                | ClassElement::StaticMethodDefinition(name, _)
                | ClassElement::FieldDefinition(name, _)
                | ClassElement::StaticFieldDefinition(name, _) => name.visit_with(self),
                _ => ControlFlow::Continue(()),
            }
        }

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

        fn visit_super_property_access(
            &mut self,
            node: &'ast SuperPropertyAccess,
        ) -> ControlFlow<Self::BreakTy> {
            if [ContainsSymbol::SuperProperty, ContainsSymbol::Super].contains(&self.0) {
                return ControlFlow::Break(());
            }
            node.visit_with(self)
        }

        fn visit_super_call(&mut self, node: &'ast SuperCall) -> ControlFlow<Self::BreakTy> {
            if [ContainsSymbol::SuperCall, ContainsSymbol::Super].contains(&self.0) {
                return ControlFlow::Break(());
            }
            node.visit_with(self)
        }

        fn visit_yield(&mut self, node: &'ast Yield) -> ControlFlow<Self::BreakTy> {
            if self.0 == ContainsSymbol::YieldExpression {
                return ControlFlow::Break(());
            }

            node.visit_with(self)
        }

        fn visit_await(&mut self, node: &'ast Await) -> ControlFlow<Self::BreakTy> {
            if self.0 == ContainsSymbol::AwaitExpression {
                return ControlFlow::Break(());
            }

            node.visit_with(self)
        }

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

        fn visit_identifier(&mut self, node: &'ast Identifier) -> ControlFlow<Self::BreakTy> {
            if node.sym() == Sym::ARGUMENTS {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }

        fn visit_function(&mut self, _: &'ast Function) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_async_function(&mut self, _: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_generator(&mut self, _: &'ast Generator) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_async_generator(&mut self, _: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
            match node {
                ClassElement::MethodDefinition(name, _)
                | ClassElement::StaticMethodDefinition(name, _) => return name.visit_with(self),
                _ => {}
            }
            node.visit_with(self)
        }

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
pub(crate) trait IdentList {
    fn add(&mut self, value: Sym, function: bool);
}

impl IdentList for Vec<Sym> {
    fn add(&mut self, value: Sym, _function: bool) {
        self.push(value);
    }
}

impl IdentList for Vec<Identifier> {
    fn add(&mut self, value: Sym, _function: bool) {
        self.push(Identifier::new(value));
    }
}

impl IdentList for Vec<(Identifier, bool)> {
    fn add(&mut self, value: Sym, function: bool) {
        self.push((Identifier::new(value), function));
    }
}

impl IdentList for FxHashSet<Identifier> {
    fn add(&mut self, value: Sym, _function: bool) {
        self.insert(Identifier::new(value));
    }
}

/// The [`Visitor`] used to obtain the bound names of a node.
#[derive(Debug)]
pub(crate) struct BoundNamesVisitor<'a, T: IdentList>(pub(crate) &'a mut T);

impl<'ast, T: IdentList> Visitor<'ast> for BoundNamesVisitor<'_, T> {
    type BreakTy = Infallible;

    fn visit_identifier(&mut self, node: &'ast Identifier) -> ControlFlow<Self::BreakTy> {
        self.0.add(node.sym(), false);
        ControlFlow::Continue(())
    }

    fn visit_expression(&mut self, _: &'ast Expression) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_function(&mut self, node: &'ast Function) -> ControlFlow<Self::BreakTy> {
        if let Some(ident) = node.name() {
            self.0.add(ident.sym(), true);
        }
        ControlFlow::Continue(())
    }

    fn visit_generator(&mut self, node: &'ast Generator) -> ControlFlow<Self::BreakTy> {
        if let Some(ident) = node.name() {
            self.0.add(ident.sym(), false);
        }
        ControlFlow::Continue(())
    }

    fn visit_async_function(&mut self, node: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
        if let Some(ident) = node.name() {
            self.0.add(ident.sym(), false);
        }
        ControlFlow::Continue(())
    }

    fn visit_async_generator(&mut self, node: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
        if let Some(ident) = node.name() {
            self.0.add(ident.sym(), false);
        }
        ControlFlow::Continue(())
    }

    fn visit_class(&mut self, node: &'ast Class) -> ControlFlow<Self::BreakTy> {
        if let Some(ident) = node.name() {
            self.0.add(ident.sym(), false);
        }
        ControlFlow::Continue(())
    }

    fn visit_export_declaration(
        &mut self,
        node: &'ast ExportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            ExportDeclaration::VarStatement(var) => try_break!(self.visit_var_declaration(var)),
            ExportDeclaration::Declaration(decl) => try_break!(self.visit_declaration(decl)),
            ExportDeclaration::DefaultFunction(f) => {
                self.0
                    .add(f.name().map_or(Sym::DEFAULT_EXPORT, Identifier::sym), true);
            }
            ExportDeclaration::DefaultGenerator(g) => {
                self.0
                    .add(g.name().map_or(Sym::DEFAULT_EXPORT, Identifier::sym), false);
            }
            ExportDeclaration::DefaultAsyncFunction(af) => {
                self.0.add(
                    af.name().map_or(Sym::DEFAULT_EXPORT, Identifier::sym),
                    false,
                );
            }
            ExportDeclaration::DefaultAsyncGenerator(ag) => {
                self.0.add(
                    ag.name().map_or(Sym::DEFAULT_EXPORT, Identifier::sym),
                    false,
                );
            }
            ExportDeclaration::DefaultClassDeclaration(cl) => {
                self.0.add(
                    cl.name().map_or(Sym::DEFAULT_EXPORT, Identifier::sym),
                    false,
                );
            }
            ExportDeclaration::DefaultAssignmentExpression(_) => {
                self.0.add(Sym::DEFAULT_EXPORT, false);
            }
            ExportDeclaration::ReExport { .. } | ExportDeclaration::List(_) => {}
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

    fn visit_expression(&mut self, _: &'ast Expression) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }
    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        if let Statement::Labelled(labelled) = node {
            return self.visit_labelled(labelled);
        }
        ControlFlow::Continue(())
    }
    fn visit_declaration(&mut self, node: &'ast Declaration) -> ControlFlow<Self::BreakTy> {
        BoundNamesVisitor(self.0).visit_declaration(node)
    }
    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        match node {
            LabelledItem::Function(f) => BoundNamesVisitor(self.0).visit_function(f),
            LabelledItem::Statement(_) => ControlFlow::Continue(()),
        }
    }
    fn visit_function(&mut self, node: &'ast Function) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    fn visit_async_function(&mut self, node: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    fn visit_generator(&mut self, node: &'ast Generator) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    fn visit_async_generator(&mut self, node: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    fn visit_arrow_function(&mut self, node: &'ast ArrowFunction) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    fn visit_async_arrow_function(
        &mut self,
        node: &'ast AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.body(), self.0);
        ControlFlow::Continue(())
    }
    fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
        if let ClassElement::StaticBlock(stmts) = node {
            top_level_lexicals(stmts, self.0);
        }
        ControlFlow::Continue(())
    }
    fn visit_import_declaration(
        &mut self,
        node: &'ast ImportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        BoundNamesVisitor(self.0).visit_import_declaration(node)
    }
    fn visit_export_declaration(
        &mut self,
        node: &'ast ExportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        if matches!(node, ExportDeclaration::VarStatement(_)) {
            return ControlFlow::Continue(());
        }
        BoundNamesVisitor(self.0).visit_export_declaration(node)
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

    fn visit_expression(&mut self, _: &'ast Expression) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_declaration(&mut self, _: &'ast Declaration) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_var_declaration(&mut self, node: &'ast VarDeclaration) -> ControlFlow<Self::BreakTy> {
        BoundNamesVisitor(self.0).visit_var_declaration(node)
    }

    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        match node {
            LabelledItem::Function(_) => ControlFlow::Continue(()),
            LabelledItem::Statement(stmt) => stmt.visit_with(self),
        }
    }

    fn visit_function(&mut self, node: &'ast Function) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }

    fn visit_async_function(&mut self, node: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }

    fn visit_generator(&mut self, node: &'ast Generator) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }

    fn visit_async_generator(&mut self, node: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }

    fn visit_arrow_function(&mut self, node: &'ast ArrowFunction) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }

    fn visit_async_arrow_function(
        &mut self,
        node: &'ast AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.body(), self.0);
        ControlFlow::Continue(())
    }

    fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
        if let ClassElement::StaticBlock(stmts) = node {
            top_level_vars(stmts, self.0);
        }
        node.visit_with(self)
    }

    fn visit_import_declaration(
        &mut self,
        _: &'ast ImportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_export_declaration(
        &mut self,
        node: &'ast ExportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            ExportDeclaration::VarStatement(var) => {
                BoundNamesVisitor(self.0).visit_var_declaration(var)
            }
            _ => ControlFlow::Continue(()),
        }
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
pub fn var_declared_names<'a, N>(node: &'a N) -> FxHashSet<Identifier>
where
    &'a N: Into<NodeRef<'a>>,
{
    let mut names = FxHashSet::default();
    VarDeclaredNamesVisitor(&mut names).visit(node.into());
    names
}

/// Utility function that collects the top level lexicals of a statement list into `names`.
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

/// Resolves the private names of a class and all of the contained classes and private identifiers.
pub fn class_private_name_resolver(node: &mut Class, top_level_class_index: usize) -> bool {
    /// Visitor used by the function to search for an identifier with the name `arguments`.
    #[derive(Debug, Clone)]
    struct ClassPrivateNameResolver {
        private_environments_stack: Vec<FxHashMap<Sym, PrivateName>>,
        top_level_class_index: usize,
    }

    impl<'ast> VisitorMut<'ast> for ClassPrivateNameResolver {
        type BreakTy = ();

        #[inline]
        fn visit_class_mut(&mut self, node: &'ast mut Class) -> ControlFlow<Self::BreakTy> {
            let mut names = FxHashMap::default();

            for element in node.elements.iter_mut() {
                match element {
                    ClassElement::PrivateMethodDefinition(name, _)
                    | ClassElement::PrivateStaticMethodDefinition(name, _)
                    | ClassElement::PrivateFieldDefinition(name, _)
                    | ClassElement::PrivateStaticFieldDefinition(name, _) => {
                        name.indices = (
                            self.top_level_class_index,
                            self.private_environments_stack.len(),
                        );
                        names.insert(name.description(), *name);
                    }
                    _ => {}
                }
            }

            self.private_environments_stack.push(names);

            for element in node.elements.iter_mut() {
                match element {
                    ClassElement::MethodDefinition(name, method)
                    | ClassElement::StaticMethodDefinition(name, method) => {
                        try_break!(self.visit_property_name_mut(name));
                        try_break!(self.visit_method_definition_mut(method));
                    }
                    ClassElement::PrivateMethodDefinition(_, method)
                    | ClassElement::PrivateStaticMethodDefinition(_, method) => {
                        try_break!(self.visit_method_definition_mut(method));
                    }
                    ClassElement::FieldDefinition(name, expression)
                    | ClassElement::StaticFieldDefinition(name, expression) => {
                        try_break!(self.visit_property_name_mut(name));
                        if let Some(expression) = expression {
                            try_break!(self.visit_expression_mut(expression));
                        }
                    }
                    ClassElement::PrivateFieldDefinition(_, expression)
                    | ClassElement::PrivateStaticFieldDefinition(_, expression) => {
                        if let Some(expression) = expression {
                            try_break!(self.visit_expression_mut(expression));
                        }
                    }
                    ClassElement::StaticBlock(statement_list) => {
                        try_break!(self.visit_statement_list_mut(statement_list));
                    }
                }
            }

            if let Some(function) = &mut node.constructor {
                try_break!(self.visit_function_mut(function));
            }

            self.private_environments_stack.pop();

            ControlFlow::Continue(())
        }

        #[inline]
        fn visit_private_name_mut(
            &mut self,
            node: &'ast mut PrivateName,
        ) -> ControlFlow<Self::BreakTy> {
            let mut found = false;

            for environment in self.private_environments_stack.iter().rev() {
                if let Some(n) = environment.get(&node.description()) {
                    found = true;
                    node.indices = n.indices;
                    break;
                }
            }

            if found {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        }
    }

    let mut visitor = ClassPrivateNameResolver {
        private_environments_stack: Vec::new(),
        top_level_class_index,
    };

    visitor.visit_class_mut(node).is_continue()
}

/// This function checks multiple syntax errors conditions for labels, `break` and `continue`.
///
/// The following syntax errors are checked:
/// - [`ContainsDuplicateLabels`][ContainsDuplicateLabels]
/// - [`ContainsUndefinedBreakTarget`][ContainsUndefinedBreakTarget]
/// - [`ContainsUndefinedContinueTarget`][ContainsUndefinedContinueTarget]
/// - Early errors for [`BreakStatement`][BreakStatement]
/// - Early errors for [`ContinueStatement`][ContinueStatement]
///
/// [ContainsDuplicateLabels]: https://tc39.es/ecma262/#sec-static-semantics-containsduplicatelabels
/// [ContainsUndefinedBreakTarget]: https://tc39.es/ecma262/#sec-static-semantics-containsundefinedbreaktarget
/// [ContainsUndefinedContinueTarget]: https://tc39.es/ecma262/#sec-static-semantics-containsundefinedcontinuetarget
/// [BreakStatement]: https://tc39.es/ecma262/#sec-break-statement-static-semantics-early-errors
/// [ContinueStatement]: https://tc39.es/ecma262/#sec-continue-statement-static-semantics-early-errors
#[must_use]
pub fn check_labels<N>(node: &N, interner: &Interner) -> Option<String>
where
    N: VisitWith,
{
    enum CheckLabelsError {
        DuplicateLabel(Sym),
        UndefinedBreakTarget(Sym),
        UndefinedContinueTarget(Sym),
        IllegalBreakStatement,
        IllegalContinueStatement,
    }

    #[derive(Debug, Clone)]
    struct CheckLabelsResolver {
        labels: FxHashSet<Sym>,
        continue_iteration_labels: FxHashSet<Sym>,
        continue_labels: Option<FxHashSet<Sym>>,
        iteration: bool,
        switch: bool,
    }

    impl<'ast> Visitor<'ast> for CheckLabelsResolver {
        type BreakTy = CheckLabelsError;

        fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
            match node {
                Statement::Block(node) => self.visit_block(node),
                Statement::Var(_)
                | Statement::Empty
                | Statement::Expression(_)
                | Statement::Return(_)
                | Statement::Throw(_) => ControlFlow::Continue(()),
                Statement::If(node) => self.visit_if(node),
                Statement::DoWhileLoop(node) => self.visit_do_while_loop(node),
                Statement::WhileLoop(node) => self.visit_while_loop(node),
                Statement::ForLoop(node) => self.visit_for_loop(node),
                Statement::ForInLoop(node) => self.visit_for_in_loop(node),
                Statement::ForOfLoop(node) => self.visit_for_of_loop(node),
                Statement::Switch(node) => self.visit_switch(node),
                Statement::Labelled(node) => self.visit_labelled(node),
                Statement::Try(node) => self.visit_try(node),
                Statement::Continue(node) => self.visit_continue(node),
                Statement::Break(node) => self.visit_break(node),
            }
        }

        fn visit_block(
            &mut self,
            node: &'ast crate::statement::Block,
        ) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.take();
            try_break!(self.visit_statement_list(node.statement_list()));
            self.continue_labels = continue_labels;
            ControlFlow::Continue(())
        }

        fn visit_break(
            &mut self,
            node: &'ast crate::statement::Break,
        ) -> ControlFlow<Self::BreakTy> {
            if let Some(label) = node.label() {
                if !self.labels.contains(&label) {
                    return ControlFlow::Break(CheckLabelsError::UndefinedBreakTarget(label));
                }
            } else if !self.iteration && !self.switch {
                return ControlFlow::Break(CheckLabelsError::IllegalBreakStatement);
            }
            ControlFlow::Continue(())
        }

        fn visit_continue(
            &mut self,
            node: &'ast crate::statement::Continue,
        ) -> ControlFlow<Self::BreakTy> {
            if !self.iteration {
                return ControlFlow::Break(CheckLabelsError::IllegalContinueStatement);
            }

            if let Some(label) = node.label() {
                if !self.continue_iteration_labels.contains(&label) {
                    return ControlFlow::Break(CheckLabelsError::UndefinedContinueTarget(label));
                }
            }
            ControlFlow::Continue(())
        }

        fn visit_do_while_loop(
            &mut self,
            node: &'ast crate::statement::DoWhileLoop,
        ) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.take();
            let continue_iteration_labels = self.continue_iteration_labels.clone();
            if let Some(continue_labels) = &continue_labels {
                self.continue_iteration_labels.extend(continue_labels);
            }
            let iteration = self.iteration;
            self.iteration = true;
            try_break!(self.visit_statement(node.body()));
            self.continue_iteration_labels = continue_iteration_labels;
            self.continue_labels = continue_labels;
            self.iteration = iteration;
            ControlFlow::Continue(())
        }

        fn visit_while_loop(
            &mut self,
            node: &'ast crate::statement::WhileLoop,
        ) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.take();
            let continue_iteration_labels = self.continue_iteration_labels.clone();
            if let Some(continue_labels) = &continue_labels {
                self.continue_iteration_labels.extend(continue_labels);
            }
            let iteration = self.iteration;
            self.iteration = true;
            try_break!(self.visit_statement(node.body()));
            self.continue_iteration_labels = continue_iteration_labels;
            self.continue_labels = continue_labels;
            self.iteration = iteration;
            ControlFlow::Continue(())
        }

        fn visit_for_loop(
            &mut self,
            node: &'ast crate::statement::ForLoop,
        ) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.take();
            let continue_iteration_labels = self.continue_iteration_labels.clone();
            if let Some(continue_labels) = &continue_labels {
                self.continue_iteration_labels.extend(continue_labels);
            }
            let iteration = self.iteration;
            self.iteration = true;
            try_break!(self.visit_statement(node.body()));
            self.continue_iteration_labels = continue_iteration_labels;
            self.continue_labels = continue_labels;
            self.iteration = iteration;
            ControlFlow::Continue(())
        }

        fn visit_for_in_loop(
            &mut self,
            node: &'ast crate::statement::ForInLoop,
        ) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.take();
            let continue_iteration_labels = self.continue_iteration_labels.clone();
            if let Some(continue_labels) = &continue_labels {
                self.continue_iteration_labels.extend(continue_labels);
            }
            let iteration = self.iteration;
            self.iteration = true;
            try_break!(self.visit_statement(node.body()));
            self.continue_iteration_labels = continue_iteration_labels;
            self.continue_labels = continue_labels;
            self.iteration = iteration;
            ControlFlow::Continue(())
        }

        fn visit_for_of_loop(
            &mut self,
            node: &'ast crate::statement::ForOfLoop,
        ) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.take();
            let continue_iteration_labels = self.continue_iteration_labels.clone();
            if let Some(continue_labels) = &continue_labels {
                self.continue_iteration_labels.extend(continue_labels);
            }
            let iteration = self.iteration;
            self.iteration = true;
            try_break!(self.visit_statement(node.body()));
            self.continue_iteration_labels = continue_iteration_labels;
            self.continue_labels = continue_labels;
            self.iteration = iteration;
            ControlFlow::Continue(())
        }

        fn visit_statement_list_item(
            &mut self,
            node: &'ast StatementListItem,
        ) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.take();
            if let StatementListItem::Statement(stmt) = node {
                try_break!(self.visit_statement(stmt));
            }
            self.continue_labels = continue_labels;
            ControlFlow::Continue(())
        }

        fn visit_if(&mut self, node: &'ast crate::statement::If) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.take();
            try_break!(self.visit_statement(node.body()));
            if let Some(stmt) = node.else_node() {
                try_break!(self.visit_statement(stmt));
            }
            self.continue_labels = continue_labels;
            ControlFlow::Continue(())
        }

        fn visit_switch(
            &mut self,
            node: &'ast crate::statement::Switch,
        ) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.take();
            let switch = self.switch;
            self.switch = true;
            for case in node.cases() {
                try_break!(self.visit_statement_list(case.body()));
            }
            if let Some(default) = node.default() {
                try_break!(self.visit_statement_list(default));
            }
            self.continue_labels = continue_labels;
            self.switch = switch;
            ControlFlow::Continue(())
        }

        fn visit_labelled(
            &mut self,
            node: &'ast crate::statement::Labelled,
        ) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.clone();
            if let Some(continue_labels) = &mut self.continue_labels {
                continue_labels.insert(node.label());
            } else {
                let mut continue_labels = FxHashSet::default();
                continue_labels.insert(node.label());
                self.continue_labels = Some(continue_labels);
            }

            if !self.labels.insert(node.label()) {
                return ControlFlow::Break(CheckLabelsError::DuplicateLabel(node.label()));
            }
            try_break!(self.visit_labelled_item(node.item()));
            self.labels.remove(&node.label());
            self.continue_labels = continue_labels;
            ControlFlow::Continue(())
        }

        fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
            match node {
                LabelledItem::Statement(stmt) => self.visit_statement(stmt),
                LabelledItem::Function(_) => ControlFlow::Continue(()),
            }
        }

        fn visit_try(&mut self, node: &'ast crate::statement::Try) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.take();
            try_break!(self.visit_block(node.block()));
            if let Some(catch) = node.catch() {
                try_break!(self.visit_block(catch.block()));
            }
            if let Some(finally) = node.finally() {
                try_break!(self.visit_block(finally.block()));
            }
            self.continue_labels = continue_labels;
            ControlFlow::Continue(())
        }

        fn visit_module_item_list(
            &mut self,
            node: &'ast crate::ModuleItemList,
        ) -> ControlFlow<Self::BreakTy> {
            let continue_labels = self.continue_labels.take();
            for item in node.items() {
                try_break!(self.visit_module_item(item));
            }
            self.continue_labels = continue_labels;
            ControlFlow::Continue(())
        }

        fn visit_module_item(
            &mut self,
            node: &'ast crate::ModuleItem,
        ) -> ControlFlow<Self::BreakTy> {
            match node {
                crate::ModuleItem::ImportDeclaration(_)
                | crate::ModuleItem::ExportDeclaration(_) => ControlFlow::Continue(()),
                crate::ModuleItem::StatementListItem(node) => self.visit_statement_list_item(node),
            }
        }
    }

    let mut visitor = CheckLabelsResolver {
        labels: FxHashSet::default(),
        continue_iteration_labels: FxHashSet::default(),
        continue_labels: None,
        iteration: false,
        switch: false,
    };

    if let ControlFlow::Break(error) = node.visit_with(&mut visitor) {
        let msg = match error {
            CheckLabelsError::DuplicateLabel(label) => {
                format!("duplicate label: {}", interner.resolve_expect(label))
            }
            CheckLabelsError::UndefinedBreakTarget(label) => {
                format!("undefined break target: {}", interner.resolve_expect(label))
            }
            CheckLabelsError::UndefinedContinueTarget(label) => format!(
                "undefined continue target: {}",
                interner.resolve_expect(label)
            ),
            CheckLabelsError::IllegalBreakStatement => "illegal break statement".into(),
            CheckLabelsError::IllegalContinueStatement => "illegal continue statement".into(),
        };

        Some(msg)
    } else {
        None
    }
}

/// Returns `true` if the given node contains a `CoverInitializedName`.
#[must_use]
pub fn contains_invalid_object_literal<N>(node: &N) -> bool
where
    N: VisitWith,
{
    #[derive(Debug, Clone)]
    struct ContainsInvalidObjectLiteral {}

    impl<'ast> Visitor<'ast> for ContainsInvalidObjectLiteral {
        type BreakTy = ();

        fn visit_object_literal(
            &mut self,
            node: &'ast crate::expression::literal::ObjectLiteral,
        ) -> ControlFlow<Self::BreakTy> {
            for pd in node.properties() {
                if let PropertyDefinition::CoverInitializedName(..) = pd {
                    return ControlFlow::Break(());
                }
                try_break!(self.visit_property_definition(pd));
            }
            ControlFlow::Continue(())
        }
    }

    let mut visitor = ContainsInvalidObjectLiteral {};

    node.visit_with(&mut visitor).is_break()
}
