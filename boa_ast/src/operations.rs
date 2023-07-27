//! Definitions of various **Syntax-Directed Operations** used in the [spec].
//!
//! [spec]: https://tc39.es/ecma262/#sec-syntax-directed-operations

use core::ops::ControlFlow;
use std::convert::Infallible;

use boa_interner::{Interner, Sym};
use rustc_hash::FxHashSet;

use crate::{
    declaration::{
        Binding, ExportDeclaration, ImportDeclaration, LexicalDeclaration, VarDeclaration, Variable,
    },
    expression::{
        access::{PrivatePropertyAccess, SuperPropertyAccess},
        operator::BinaryInPrivate,
        Await, Identifier, OptionalOperationKind, SuperCall, Yield,
    },
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunction, AsyncGenerator, Class, ClassElement,
        Function, Generator,
    },
    property::{MethodDefinition, PropertyDefinition},
    statement::{
        iteration::{ForLoopInitializer, IterableLoopInitializer},
        LabelledItem,
    },
    try_break,
    visitor::{NodeRef, VisitWith, Visitor},
    Declaration, Expression, ModuleItem, Script, Statement, StatementList, StatementListItem,
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

    fn visit_script(&mut self, node: &'ast Script) -> ControlFlow<Self::BreakTy> {
        top_level_lexicals(node.statements(), self.0);
        ControlFlow::Continue(())
    }

    fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
        match node {
            // ModuleItem : ImportDeclaration
            ModuleItem::ImportDeclaration(import) => {
                // 1. Return the BoundNames of ImportDeclaration.
                BoundNamesVisitor(self.0).visit_import_declaration(import)
            }

            // ModuleItem : ExportDeclaration
            ModuleItem::ExportDeclaration(export) => {
                // 1. If ExportDeclaration is export VariableStatement, return a new empty List.
                if matches!(export, ExportDeclaration::VarStatement(_)) {
                    ControlFlow::Continue(())
                } else {
                    // 2. Return the BoundNames of ExportDeclaration.
                    BoundNamesVisitor(self.0).visit_export_declaration(export)
                }
            }

            // ModuleItem : StatementListItem
            ModuleItem::StatementListItem(item) => {
                // 1. Return LexicallyDeclaredNames of StatementListItem.
                self.visit_statement_list_item(item)
            }
        }
    }

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
        self.visit_script(node.body())
    }

    fn visit_async_function(&mut self, node: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
        self.visit_script(node.body())
    }

    fn visit_generator(&mut self, node: &'ast Generator) -> ControlFlow<Self::BreakTy> {
        self.visit_script(node.body())
    }

    fn visit_async_generator(&mut self, node: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
        self.visit_script(node.body())
    }

    fn visit_arrow_function(&mut self, node: &'ast ArrowFunction) -> ControlFlow<Self::BreakTy> {
        self.visit_script(node.body())
    }

    fn visit_async_arrow_function(
        &mut self,
        node: &'ast AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_script(node.body())
    }

    fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
        if let ClassElement::StaticBlock(body) = node {
            self.visit_script(body);
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

    fn visit_script(&mut self, node: &'ast Script) -> ControlFlow<Self::BreakTy> {
        top_level_vars(node.statements(), self.0);
        ControlFlow::Continue(())
    }

    fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
        match node {
            // ModuleItem : ImportDeclaration
            ModuleItem::ImportDeclaration(_) => {
                // 1. Return a new empty List.
                ControlFlow::Continue(())
            }

            // ModuleItem : ExportDeclaration
            ModuleItem::ExportDeclaration(export) => {
                // 1. If ExportDeclaration is export VariableStatement, return BoundNames of ExportDeclaration.
                if let ExportDeclaration::VarStatement(var) = export {
                    BoundNamesVisitor(self.0).visit_var_declaration(var)
                } else {
                    // 2. Return a new empty List.
                    ControlFlow::Continue(())
                }
            }

            ModuleItem::StatementListItem(item) => self.visit_statement_list_item(item),
        }
    }

    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        match node {
            Statement::Empty
            | Statement::Expression(_)
            | Statement::Continue(_)
            | Statement::Break(_)
            | Statement::Return(_)
            | Statement::Throw(_) => ControlFlow::Continue(()),
            Statement::Block(node) => self.visit(node),
            Statement::Var(node) => self.visit(node),
            Statement::If(node) => self.visit(node),
            Statement::DoWhileLoop(node) => self.visit(node),
            Statement::WhileLoop(node) => self.visit(node),
            Statement::ForLoop(node) => self.visit(node),
            Statement::ForInLoop(node) => self.visit(node),
            Statement::ForOfLoop(node) => self.visit(node),
            Statement::Switch(node) => self.visit(node),
            Statement::Labelled(node) => self.visit(node),
            Statement::Try(node) => self.visit(node),
            Statement::With(node) => self.visit(node),
        }
    }

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            StatementListItem::Statement(stmt) => self.visit_statement(stmt),
            StatementListItem::Declaration(_) => ControlFlow::Continue(()),
        }
    }

    fn visit_variable(&mut self, node: &'ast Variable) -> ControlFlow<Self::BreakTy> {
        BoundNamesVisitor(self.0).visit_variable(node)
    }

    fn visit_if(&mut self, node: &'ast crate::statement::If) -> ControlFlow<Self::BreakTy> {
        if let Some(node) = node.else_node() {
            self.visit(node);
        }
        self.visit(node.body())
    }

    fn visit_do_while_loop(
        &mut self,
        node: &'ast crate::statement::DoWhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())
    }

    fn visit_while_loop(
        &mut self,
        node: &'ast crate::statement::WhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())
    }

    fn visit_for_loop(
        &mut self,
        node: &'ast crate::statement::ForLoop,
    ) -> ControlFlow<Self::BreakTy> {
        if let Some(ForLoopInitializer::Var(node)) = node.init() {
            BoundNamesVisitor(self.0).visit_var_declaration(node);
        }
        self.visit(node.body())
    }

    fn visit_for_in_loop(
        &mut self,
        node: &'ast crate::statement::ForInLoop,
    ) -> ControlFlow<Self::BreakTy> {
        if let IterableLoopInitializer::Var(node) = node.initializer() {
            BoundNamesVisitor(self.0).visit_variable(node);
        }
        self.visit(node.body())
    }

    fn visit_for_of_loop(
        &mut self,
        node: &'ast crate::statement::ForOfLoop,
    ) -> ControlFlow<Self::BreakTy> {
        if let IterableLoopInitializer::Var(node) = node.initializer() {
            BoundNamesVisitor(self.0).visit_variable(node);
        }
        self.visit(node.body())
    }

    fn visit_with(&mut self, node: &'ast crate::statement::With) -> ControlFlow<Self::BreakTy> {
        self.visit(node.statement())
    }

    fn visit_switch(&mut self, node: &'ast crate::statement::Switch) -> ControlFlow<Self::BreakTy> {
        for case in node.cases() {
            self.visit(case);
        }
        if let Some(node) = node.default() {
            self.visit(node);
        }
        ControlFlow::Continue(())
    }

    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        match node {
            LabelledItem::Function(_) => ControlFlow::Continue(()),
            LabelledItem::Statement(stmt) => self.visit(stmt),
        }
    }

    fn visit_try(&mut self, node: &'ast crate::statement::Try) -> ControlFlow<Self::BreakTy> {
        if let Some(node) = node.finally() {
            self.visit(node);
        }
        if let Some(node) = node.catch() {
            self.visit(node.block());
        }
        self.visit(node.block())
    }

    fn visit_function(&mut self, node: &'ast Function) -> ControlFlow<Self::BreakTy> {
        self.visit_script(node.body())
    }

    fn visit_async_function(&mut self, node: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
        self.visit_script(node.body())
    }

    fn visit_generator(&mut self, node: &'ast Generator) -> ControlFlow<Self::BreakTy> {
        self.visit_script(node.body())
    }

    fn visit_async_generator(&mut self, node: &'ast AsyncGenerator) -> ControlFlow<Self::BreakTy> {
        self.visit_script(node.body())
    }

    fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
        if let ClassElement::StaticBlock(body) = node {
            self.visit_script(body);
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
///
/// This is equivalent to the [`TopLevelLexicallyDeclaredNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-toplevellexicallydeclarednames
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

/// Utility function that collects the top level vars of a statement list into `names`.
///
/// This is equivalent to the [`TopLevelVarDeclaredNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-toplevelvardeclarednames
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

/// Returns `true` if all private identifiers in a node are valid.
///
/// This is equivalent to the [`AllPrivateIdentifiersValid`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-allprivateidentifiersvalid
#[must_use]
#[inline]
pub fn all_private_identifiers_valid<'a, N>(node: &'a N, private_names: Vec<Sym>) -> bool
where
    &'a N: Into<NodeRef<'a>>,
{
    AllPrivateIdentifiersValidVisitor(private_names)
        .visit(node.into())
        .is_continue()
}

struct AllPrivateIdentifiersValidVisitor(Vec<Sym>);

impl<'ast> Visitor<'ast> for AllPrivateIdentifiersValidVisitor {
    type BreakTy = ();

    fn visit_class(&mut self, node: &'ast Class) -> ControlFlow<Self::BreakTy> {
        if let Some(node) = node.super_ref() {
            try_break!(self.visit(node));
        }

        let mut names = self.0.clone();
        for element in node.elements() {
            match element {
                ClassElement::PrivateMethodDefinition(name, _)
                | ClassElement::PrivateStaticMethodDefinition(name, _)
                | ClassElement::PrivateFieldDefinition(name, _)
                | ClassElement::PrivateStaticFieldDefinition(name, _) => {
                    names.push(name.description());
                }
                _ => {}
            }
        }

        let mut visitor = Self(names);

        if let Some(node) = node.constructor() {
            try_break!(visitor.visit(node));
        }

        for element in node.elements() {
            match element {
                ClassElement::MethodDefinition(name, method)
                | ClassElement::StaticMethodDefinition(name, method) => {
                    try_break!(visitor.visit(name));
                    try_break!(visitor.visit(method));
                }
                ClassElement::FieldDefinition(name, expression)
                | ClassElement::StaticFieldDefinition(name, expression) => {
                    try_break!(visitor.visit(name));
                    if let Some(expression) = expression {
                        try_break!(visitor.visit(expression));
                    }
                }
                ClassElement::PrivateMethodDefinition(_, method)
                | ClassElement::PrivateStaticMethodDefinition(_, method) => {
                    try_break!(visitor.visit(method));
                }
                ClassElement::PrivateFieldDefinition(_, expression)
                | ClassElement::PrivateStaticFieldDefinition(_, expression) => {
                    if let Some(expression) = expression {
                        try_break!(visitor.visit(expression));
                    }
                }
                ClassElement::StaticBlock(statement_list) => {
                    try_break!(visitor.visit(statement_list));
                }
            }
        }

        ControlFlow::Continue(())
    }

    fn visit_private_property_access(
        &mut self,
        node: &'ast PrivatePropertyAccess,
    ) -> ControlFlow<Self::BreakTy> {
        if self.0.contains(&node.field().description()) {
            self.visit(node.target())
        } else {
            ControlFlow::Break(())
        }
    }

    fn visit_binary_in_private(
        &mut self,
        node: &'ast BinaryInPrivate,
    ) -> ControlFlow<Self::BreakTy> {
        if self.0.contains(&node.lhs().description()) {
            self.visit(node.rhs())
        } else {
            ControlFlow::Break(())
        }
    }

    fn visit_optional_operation_kind(
        &mut self,
        node: &'ast OptionalOperationKind,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            OptionalOperationKind::SimplePropertyAccess { field } => {
                self.visit_property_access_field(field)
            }
            OptionalOperationKind::PrivatePropertyAccess { field } => {
                if self.0.contains(&field.description()) {
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(())
                }
            }
            OptionalOperationKind::Call { args } => {
                for arg in args.iter() {
                    try_break!(self.visit_expression(arg));
                }
                ControlFlow::Continue(())
            }
        }
    }
}

/// Errors that can occur when checking labels.
#[derive(Debug, Clone, Copy)]
pub enum CheckLabelsError {
    /// A label was used multiple times.
    DuplicateLabel(Sym),

    /// A `break` statement was used with a label that was not defined.
    UndefinedBreakTarget(Sym),

    /// A `continue` statement was used with a label that was not defined.
    UndefinedContinueTarget(Sym),

    /// A `break` statement was used in a non-looping context.
    IllegalBreakStatement,

    /// A `continue` statement was used in a non-looping context.
    IllegalContinueStatement,
}

impl CheckLabelsError {
    /// Returns an error message based on the error.
    #[must_use]
    pub fn message(&self, interner: &Interner) -> String {
        match self {
            Self::DuplicateLabel(label) => {
                format!("duplicate label: {}", interner.resolve_expect(*label))
            }
            Self::UndefinedBreakTarget(label) => {
                format!(
                    "undefined break target: {}",
                    interner.resolve_expect(*label)
                )
            }
            Self::UndefinedContinueTarget(label) => format!(
                "undefined continue target: {}",
                interner.resolve_expect(*label)
            ),
            Self::IllegalBreakStatement => "illegal break statement".into(),
            Self::IllegalContinueStatement => "illegal continue statement".into(),
        }
    }
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
///
/// # Errors
///
/// This function returns an error for the first syntax error that is found.
pub fn check_labels<N>(node: &N) -> Result<(), CheckLabelsError>
where
    N: VisitWith,
{
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
                Statement::With(with) => self.visit_with(with),
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

        fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
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
        Err(error)
    } else {
        Ok(())
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

/// The type of a lexically scoped declaration.
#[derive(Copy, Clone, Debug)]
pub enum LexicallyScopedDeclaration<'a> {
    /// See [`LexicalDeclaration`]
    LexicalDeclaration(&'a LexicalDeclaration),

    /// See [`Function`]
    Function(&'a Function),

    /// See [`Generator`]
    Generator(&'a Generator),

    /// See [`AsyncFunction`]
    AsyncFunction(&'a AsyncFunction),

    /// See [`AsyncGenerator`]
    AsyncGenerator(&'a AsyncGenerator),

    /// See [`Class`]
    Class(&'a Class),

    /// A default assignment expression as an export declaration.
    ///
    /// Only valid inside module exports.
    AssignmentExpression(&'a Expression),
}

impl LexicallyScopedDeclaration<'_> {
    /// Return the bound names of the declaration.
    #[must_use]
    pub fn bound_names(&self) -> Vec<Identifier> {
        match *self {
            Self::LexicalDeclaration(v) => bound_names(v),
            Self::Function(f) => bound_names(f),
            Self::Generator(g) => bound_names(g),
            Self::AsyncFunction(f) => bound_names(f),
            Self::AsyncGenerator(g) => bound_names(g),
            Self::Class(cl) => bound_names(cl),
            Self::AssignmentExpression(expr) => bound_names(expr),
        }
    }
}

impl<'ast> From<&'ast Declaration> for LexicallyScopedDeclaration<'ast> {
    fn from(value: &'ast Declaration) -> LexicallyScopedDeclaration<'ast> {
        match value {
            Declaration::Function(f) => Self::Function(f),
            Declaration::Generator(g) => Self::Generator(g),
            Declaration::AsyncFunction(af) => Self::AsyncFunction(af),
            Declaration::AsyncGenerator(ag) => Self::AsyncGenerator(ag),
            Declaration::Class(c) => Self::Class(c),
            Declaration::Lexical(lex) => Self::LexicalDeclaration(lex),
        }
    }
}

/// Returns a list of lexically scoped declarations of the given node.
///
/// This is equivalent to the [`LexicallyScopedDeclarations`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-lexicallyscopeddeclarations
#[must_use]
pub fn lexically_scoped_declarations<'a, N>(node: &'a N) -> Vec<LexicallyScopedDeclaration<'a>>
where
    &'a N: Into<NodeRef<'a>>,
{
    let mut declarations = Vec::new();
    LexicallyScopedDeclarationsVisitor(&mut declarations).visit(node.into());
    declarations
}

/// The [`Visitor`] used to obtain the lexically scoped declarations of a node.
#[derive(Debug)]
struct LexicallyScopedDeclarationsVisitor<'a, 'ast>(&'a mut Vec<LexicallyScopedDeclaration<'ast>>);

impl<'ast> Visitor<'ast> for LexicallyScopedDeclarationsVisitor<'_, 'ast> {
    type BreakTy = Infallible;

    // ScriptBody : StatementList
    fn visit_script(&mut self, node: &'ast Script) -> ControlFlow<Self::BreakTy> {
        // 1. Return TopLevelLexicallyScopedDeclarations of StatementList.
        TopLevelLexicallyScopedDeclarationsVisitor(self.0).visit_statement_list(node.statements())
    }

    fn visit_export_declaration(
        &mut self,
        node: &'ast ExportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        let decl = match node {
            // ExportDeclaration :
            // export ExportFromClause FromClause ;
            // export NamedExports ;
            // export VariableStatement
            ExportDeclaration::ReExport { .. }
            | ExportDeclaration::List(_)
            | ExportDeclaration::VarStatement(_) => {
                //     1. Return a new empty List.
                return ControlFlow::Continue(());
            }

            // ExportDeclaration : export Declaration
            ExportDeclaration::Declaration(decl) => {
                // 1. Return a List whose sole element is DeclarationPart of Declaration.
                decl.into()
            }

            // ExportDeclaration : export default HoistableDeclaration
            // 1. Return a List whose sole element is DeclarationPart of HoistableDeclaration.
            ExportDeclaration::DefaultFunction(f) => LexicallyScopedDeclaration::Function(f),
            ExportDeclaration::DefaultGenerator(g) => LexicallyScopedDeclaration::Generator(g),
            ExportDeclaration::DefaultAsyncFunction(af) => {
                LexicallyScopedDeclaration::AsyncFunction(af)
            }
            ExportDeclaration::DefaultAsyncGenerator(ag) => {
                LexicallyScopedDeclaration::AsyncGenerator(ag)
            }

            // ExportDeclaration : export default ClassDeclaration
            ExportDeclaration::DefaultClassDeclaration(c) => {
                // 1. Return a List whose sole element is ClassDeclaration.
                LexicallyScopedDeclaration::Class(c)
            }

            // ExportDeclaration : export default AssignmentExpression ;
            ExportDeclaration::DefaultAssignmentExpression(expr) => {
                // 1. Return a List whose sole element is this ExportDeclaration.
                LexicallyScopedDeclaration::AssignmentExpression(expr)
            }
        };

        self.0.push(decl);

        ControlFlow::Continue(())
    }

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            // StatementListItem : Statement
            StatementListItem::Statement(Statement::Labelled(labelled)) => {
                // 1. If Statement is Statement : LabelledStatement , return LexicallyScopedDeclarations of LabelledStatement.
                self.visit_labelled(labelled)
            }
            StatementListItem::Statement(_) => {
                // 2. Return a new empty List.
                ControlFlow::Continue(())
            }

            // StatementListItem : Declaration
            StatementListItem::Declaration(declaration) => {
                // 1. Return a List whose sole element is DeclarationPart of Declaration.
                self.0.push(declaration.into());
                ControlFlow::Continue(())
            }
        }
    }

    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        match node {
            // LabelledItem : FunctionDeclaration
            LabelledItem::Function(f) => {
                // 1. Return « FunctionDeclaration ».
                self.0.push(LexicallyScopedDeclaration::Function(f));
            }

            // LabelledItem : Statement
            LabelledItem::Statement(_) => {
                // 1. Return a new empty List.
            }
        }
        ControlFlow::Continue(())
    }

    fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
        match node {
            ModuleItem::StatementListItem(item) => self.visit_statement_list_item(item),
            ModuleItem::ExportDeclaration(export) => self.visit_export_declaration(export),

            // ModuleItem : ImportDeclaration
            ModuleItem::ImportDeclaration(_) => {
                // 1. Return a new empty List.
                ControlFlow::Continue(())
            }
        }
    }
}
/// The [`Visitor`] used to obtain the top level lexically scoped declarations of a node.
///
/// This is equivalent to the [`TopLevelLexicallyScopedDeclarations`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-toplevellexicallyscopeddeclarations
#[derive(Debug)]
struct TopLevelLexicallyScopedDeclarationsVisitor<'a, 'ast>(
    &'a mut Vec<LexicallyScopedDeclaration<'ast>>,
);

impl<'ast> Visitor<'ast> for TopLevelLexicallyScopedDeclarationsVisitor<'_, 'ast> {
    type BreakTy = Infallible;

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            // StatementListItem : Declaration
            StatementListItem::Declaration(d) => match d {
                // 1. If Declaration is Declaration : HoistableDeclaration , then
                Declaration::Function(_)
                | Declaration::Generator(_)
                | Declaration::AsyncFunction(_)
                | Declaration::AsyncGenerator(_) => {
                    // a. Return a new empty List.
                }

                // 2. Return « Declaration ».
                Declaration::Class(cl) => {
                    self.0.push(LexicallyScopedDeclaration::Class(cl));
                }
                Declaration::Lexical(lex) => {
                    self.0
                        .push(LexicallyScopedDeclaration::LexicalDeclaration(lex));
                }
            },

            // StatementListItem : Statement
            StatementListItem::Statement(_) => {
                // 1. Return a new empty List.
            }
        }

        ControlFlow::Continue(())
    }
}

/// The type of a var scoped declaration.
#[derive(Clone, Debug)]
pub enum VarScopedDeclaration {
    /// See [`VarDeclaration`]
    VariableDeclaration(Variable),

    /// See [`Function`]
    Function(Function),

    /// See [`Generator`]
    Generator(Generator),

    /// See [`AsyncFunction`]
    AsyncFunction(AsyncFunction),

    /// See [`AsyncGenerator`]
    AsyncGenerator(AsyncGenerator),
}

impl VarScopedDeclaration {
    /// Return the bound names of the declaration.
    #[must_use]
    pub fn bound_names(&self) -> Vec<Identifier> {
        match self {
            Self::VariableDeclaration(v) => bound_names(v),
            Self::Function(f) => bound_names(f),
            Self::Generator(g) => bound_names(g),
            Self::AsyncFunction(f) => bound_names(f),
            Self::AsyncGenerator(g) => bound_names(g),
        }
    }
}

/// Returns a list of var scoped declarations of the given node.
///
/// This is equivalent to the [`VarScopedDeclarations`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-varscopeddeclarations
#[must_use]
pub fn var_scoped_declarations<'a, N>(node: &'a N) -> Vec<VarScopedDeclaration>
where
    &'a N: Into<NodeRef<'a>>,
{
    let mut declarations = Vec::new();
    VarScopedDeclarationsVisitor(&mut declarations).visit(node.into());
    declarations
}

/// The [`Visitor`] used to obtain the var scoped declarations of a node.
#[derive(Debug)]
struct VarScopedDeclarationsVisitor<'a>(&'a mut Vec<VarScopedDeclaration>);

impl<'ast> Visitor<'ast> for VarScopedDeclarationsVisitor<'_> {
    type BreakTy = Infallible;

    // ScriptBody : StatementList
    fn visit_script(&mut self, node: &'ast Script) -> ControlFlow<Self::BreakTy> {
        // 1. Return TopLevelVarScopedDeclarations of StatementList.
        TopLevelVarScopedDeclarationsVisitor(self.0).visit_statement_list(node.statements())
    }

    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        match node {
            Statement::Block(s) => self.visit(s),
            Statement::Var(s) => self.visit(s),
            Statement::If(s) => self.visit(s),
            Statement::DoWhileLoop(s) => self.visit(s),
            Statement::WhileLoop(s) => self.visit(s),
            Statement::ForLoop(s) => self.visit(s),
            Statement::ForInLoop(s) => self.visit(s),
            Statement::ForOfLoop(s) => self.visit(s),
            Statement::Switch(s) => self.visit(s),
            Statement::Labelled(s) => self.visit(s),
            Statement::Try(s) => self.visit(s),
            Statement::With(s) => self.visit(s),
            Statement::Empty
            | Statement::Expression(_)
            | Statement::Continue(_)
            | Statement::Break(_)
            | Statement::Return(_)
            | Statement::Throw(_) => ControlFlow::Continue(()),
        }
    }

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            StatementListItem::Declaration(_) => ControlFlow::Continue(()),
            StatementListItem::Statement(s) => self.visit(s),
        }
    }

    fn visit_var_declaration(&mut self, node: &'ast VarDeclaration) -> ControlFlow<Self::BreakTy> {
        for var in node.0.as_ref() {
            self.0
                .push(VarScopedDeclaration::VariableDeclaration(var.clone()));
        }
        ControlFlow::Continue(())
    }

    fn visit_if(&mut self, node: &'ast crate::statement::If) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body());
        if let Some(else_node) = node.else_node() {
            self.visit(else_node);
        }
        ControlFlow::Continue(())
    }

    fn visit_do_while_loop(
        &mut self,
        node: &'ast crate::statement::DoWhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body());
        ControlFlow::Continue(())
    }

    fn visit_while_loop(
        &mut self,
        node: &'ast crate::statement::WhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body());
        ControlFlow::Continue(())
    }

    fn visit_for_loop(
        &mut self,
        node: &'ast crate::statement::ForLoop,
    ) -> ControlFlow<Self::BreakTy> {
        if let Some(ForLoopInitializer::Var(v)) = node.init() {
            self.visit(v);
        }
        self.visit(node.body());
        ControlFlow::Continue(())
    }

    fn visit_for_in_loop(
        &mut self,
        node: &'ast crate::statement::ForInLoop,
    ) -> ControlFlow<Self::BreakTy> {
        if let IterableLoopInitializer::Var(var) = node.initializer() {
            self.0
                .push(VarScopedDeclaration::VariableDeclaration(var.clone()));
        }
        self.visit(node.body());
        ControlFlow::Continue(())
    }

    fn visit_for_of_loop(
        &mut self,
        node: &'ast crate::statement::ForOfLoop,
    ) -> ControlFlow<Self::BreakTy> {
        if let IterableLoopInitializer::Var(var) = node.initializer() {
            self.0
                .push(VarScopedDeclaration::VariableDeclaration(var.clone()));
        }
        self.visit(node.body());
        ControlFlow::Continue(())
    }

    fn visit_with(&mut self, node: &'ast crate::statement::With) -> ControlFlow<Self::BreakTy> {
        self.visit(node.statement());
        ControlFlow::Continue(())
    }

    fn visit_switch(&mut self, node: &'ast crate::statement::Switch) -> ControlFlow<Self::BreakTy> {
        for case in node.cases() {
            self.visit(case);
        }
        if let Some(default) = node.default() {
            self.visit(default);
        }
        ControlFlow::Continue(())
    }

    fn visit_case(&mut self, node: &'ast crate::statement::Case) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body());
        ControlFlow::Continue(())
    }

    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        match node {
            LabelledItem::Statement(s) => self.visit(s),
            LabelledItem::Function(_) => ControlFlow::Continue(()),
        }
    }

    fn visit_catch(&mut self, node: &'ast crate::statement::Catch) -> ControlFlow<Self::BreakTy> {
        self.visit(node.block());
        ControlFlow::Continue(())
    }

    fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
        match node {
            // ModuleItem : ExportDeclaration
            ModuleItem::ExportDeclaration(decl) => {
                if let ExportDeclaration::VarStatement(var) = decl {
                    //     1. If ExportDeclaration is export VariableStatement, return VarScopedDeclarations of VariableStatement.
                    self.visit_var_declaration(var);
                }
                // 2. Return a new empty List.
            }
            ModuleItem::StatementListItem(item) => {
                self.visit_statement_list_item(item);
            }
            // ModuleItem : ImportDeclaration
            ModuleItem::ImportDeclaration(_) => {
                // 1. Return a new empty List.
            }
        }
        ControlFlow::Continue(())
    }
}

/// The [`Visitor`] used to obtain the top level var scoped declarations of a node.
///
/// This is equivalent to the [`TopLevelVarScopedDeclarations`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-toplevelvarscopeddeclarations
#[derive(Debug)]
struct TopLevelVarScopedDeclarationsVisitor<'a>(&'a mut Vec<VarScopedDeclaration>);

impl<'ast> Visitor<'ast> for TopLevelVarScopedDeclarationsVisitor<'_> {
    type BreakTy = Infallible;

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            StatementListItem::Declaration(d) => {
                match d {
                    Declaration::Function(f) => {
                        self.0.push(VarScopedDeclaration::Function(f.clone()));
                    }
                    Declaration::Generator(f) => {
                        self.0.push(VarScopedDeclaration::Generator(f.clone()));
                    }
                    Declaration::AsyncFunction(f) => {
                        self.0.push(VarScopedDeclaration::AsyncFunction(f.clone()));
                    }
                    Declaration::AsyncGenerator(f) => {
                        self.0.push(VarScopedDeclaration::AsyncGenerator(f.clone()));
                    }
                    _ => {}
                }
                ControlFlow::Continue(())
            }
            StatementListItem::Statement(Statement::Labelled(s)) => self.visit(s),
            StatementListItem::Statement(s) => {
                VarScopedDeclarationsVisitor(self.0).visit(s);
                ControlFlow::Continue(())
            }
        }
    }

    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        match node {
            LabelledItem::Statement(Statement::Labelled(s)) => self.visit(s),
            LabelledItem::Statement(s) => {
                VarScopedDeclarationsVisitor(self.0).visit(s);
                ControlFlow::Continue(())
            }
            LabelledItem::Function(f) => {
                self.0.push(VarScopedDeclaration::Function(f.clone()));
                ControlFlow::Continue(())
            }
        }
    }
}

/// Returns a list function declaration names that are directly contained in a statement lists
/// `Block`, `CaseClause` or `DefaultClause`.
/// If the function declaration would cause an early error it is not included in the list.
///
/// This behavior is used in the following annexB sections:
/// * [B.3.2.1 Changes to FunctionDeclarationInstantiation][spec0]
/// * [B.3.2.2 Changes to GlobalDeclarationInstantiation][spec1]
/// * [B.3.2.3 Changes to EvalDeclarationInstantiation][spec2]
///
/// [spec0]: https://tc39.es/ecma262/#sec-web-compat-functiondeclarationinstantiation
/// [spec1]: https://tc39.es/ecma262/#sec-web-compat-globaldeclarationinstantiation
/// [spec2]: https://tc39.es/ecma262/#sec-web-compat-evaldeclarationinstantiation
#[must_use]
pub fn annex_b_function_declarations_names<'a, N>(node: &'a N) -> Vec<Identifier>
where
    &'a N: Into<NodeRef<'a>>,
{
    let mut declarations = Vec::new();
    AnnexBFunctionDeclarationNamesVisitor(&mut declarations).visit(node.into());
    declarations
}

/// The [`Visitor`] used for [`annex_b_function_declarations_names`].
#[derive(Debug)]
struct AnnexBFunctionDeclarationNamesVisitor<'a>(&'a mut Vec<Identifier>);

impl<'ast> Visitor<'ast> for AnnexBFunctionDeclarationNamesVisitor<'_> {
    type BreakTy = Infallible;

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            StatementListItem::Statement(node) => self.visit(node),
            StatementListItem::Declaration(_) => ControlFlow::Continue(()),
        }
    }

    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        match node {
            Statement::Block(node) => self.visit(node),
            Statement::If(node) => self.visit(node),
            Statement::DoWhileLoop(node) => self.visit(node),
            Statement::WhileLoop(node) => self.visit(node),
            Statement::ForLoop(node) => self.visit(node),
            Statement::ForInLoop(node) => self.visit(node),
            Statement::ForOfLoop(node) => self.visit(node),
            Statement::Switch(node) => self.visit(node),
            Statement::Labelled(node) => self.visit(node),
            Statement::Try(node) => self.visit(node),
            Statement::With(node) => self.visit(node),
            _ => ControlFlow::Continue(()),
        }
    }

    fn visit_block(&mut self, node: &'ast crate::statement::Block) -> ControlFlow<Self::BreakTy> {
        self.visit(node.statement_list());
        for statement in node.statement_list().statements() {
            if let StatementListItem::Declaration(Declaration::Function(function)) = statement {
                let name = function
                    .name()
                    .expect("function declaration must have name");
                self.0.push(name);
            }
        }

        let lexically_declared_names = lexically_declared_names_legacy(node.statement_list());

        self.0
            .retain(|name| !lexically_declared_names.contains(&(*name, false)));

        ControlFlow::Continue(())
    }

    fn visit_switch(&mut self, node: &'ast crate::statement::Switch) -> ControlFlow<Self::BreakTy> {
        for case in node.cases() {
            self.visit(case);
            for statement in case.body().statements() {
                if let StatementListItem::Declaration(Declaration::Function(function)) = statement {
                    let name = function
                        .name()
                        .expect("function declaration must have name");
                    self.0.push(name);
                }
            }
        }
        if let Some(default) = node.default() {
            self.visit(default);
            for statement in default.statements() {
                if let StatementListItem::Declaration(Declaration::Function(function)) = statement {
                    let name = function
                        .name()
                        .expect("function declaration must have name");
                    self.0.push(name);
                }
            }
        }

        let lexically_declared_names = lexically_declared_names_legacy(node);

        self.0
            .retain(|name| !lexically_declared_names.contains(&(*name, false)));

        ControlFlow::Continue(())
    }

    fn visit_try(&mut self, node: &'ast crate::statement::Try) -> ControlFlow<Self::BreakTy> {
        self.visit(node.block());
        if let Some(catch) = node.catch() {
            self.visit(catch.block());

            if let Some(Binding::Pattern(pattern)) = catch.parameter() {
                let bound_names = bound_names(pattern);

                self.0.retain(|name| !bound_names.contains(name));
            }
        }
        if let Some(finally) = node.finally() {
            self.visit(finally.block());
        }
        ControlFlow::Continue(())
    }

    fn visit_if(&mut self, node: &'ast crate::statement::If) -> ControlFlow<Self::BreakTy> {
        if let Some(node) = node.else_node() {
            self.visit(node);
        }
        self.visit(node.body())
    }

    fn visit_do_while_loop(
        &mut self,
        node: &'ast crate::statement::DoWhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())
    }

    fn visit_while_loop(
        &mut self,
        node: &'ast crate::statement::WhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())
    }

    fn visit_for_loop(
        &mut self,
        node: &'ast crate::statement::ForLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body());

        if let Some(ForLoopInitializer::Lexical(node)) = node.init() {
            let bound_names = bound_names(node);
            self.0.retain(|name| !bound_names.contains(name));
        }

        ControlFlow::Continue(())
    }

    fn visit_for_in_loop(
        &mut self,
        node: &'ast crate::statement::ForInLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body());

        if let IterableLoopInitializer::Let(node) = node.initializer() {
            let bound_names = bound_names(node);
            self.0.retain(|name| !bound_names.contains(name));
        }
        if let IterableLoopInitializer::Const(node) = node.initializer() {
            let bound_names = bound_names(node);
            self.0.retain(|name| !bound_names.contains(name));
        }

        ControlFlow::Continue(())
    }

    fn visit_for_of_loop(
        &mut self,
        node: &'ast crate::statement::ForOfLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body());

        if let IterableLoopInitializer::Let(node) = node.initializer() {
            let bound_names = bound_names(node);
            self.0.retain(|name| !bound_names.contains(name));
        }
        if let IterableLoopInitializer::Const(node) = node.initializer() {
            let bound_names = bound_names(node);
            self.0.retain(|name| !bound_names.contains(name));
        }

        ControlFlow::Continue(())
    }

    fn visit_labelled(
        &mut self,
        node: &'ast crate::statement::Labelled,
    ) -> ControlFlow<Self::BreakTy> {
        if let LabelledItem::Statement(node) = node.item() {
            self.visit(node);
        }
        ControlFlow::Continue(())
    }

    fn visit_with(&mut self, node: &'ast crate::statement::With) -> ControlFlow<Self::BreakTy> {
        self.visit(node.statement())
    }
}

/// Returns `true` if the given statement returns a value.
#[must_use]
pub fn returns_value<'a, N>(node: &'a N) -> bool
where
    &'a N: Into<NodeRef<'a>>,
{
    ReturnsValueVisitor.visit(node.into()).is_break()
}

/// The [`Visitor`] used for [`returns_value`].
#[derive(Debug)]
struct ReturnsValueVisitor;

impl<'ast> Visitor<'ast> for ReturnsValueVisitor {
    type BreakTy = ();

    fn visit_block(&mut self, node: &'ast crate::statement::Block) -> ControlFlow<Self::BreakTy> {
        for statement in node.statement_list().statements() {
            match statement {
                StatementListItem::Declaration(_) => {}
                StatementListItem::Statement(node) => try_break!(self.visit(node)),
            }
        }
        ControlFlow::Continue(())
    }

    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        match node {
            Statement::Empty | Statement::Var(_) => {}
            Statement::Block(node) => try_break!(self.visit(node)),
            Statement::Labelled(node) => try_break!(self.visit(node)),
            _ => return ControlFlow::Break(()),
        }
        ControlFlow::Continue(())
    }

    fn visit_case(&mut self, node: &'ast crate::statement::Case) -> ControlFlow<Self::BreakTy> {
        for statement in node.body().statements() {
            match statement {
                StatementListItem::Declaration(_) => {}
                StatementListItem::Statement(node) => try_break!(self.visit(node)),
            }
        }
        ControlFlow::Continue(())
    }

    fn visit_labelled(
        &mut self,
        node: &'ast crate::statement::Labelled,
    ) -> ControlFlow<Self::BreakTy> {
        match node.item() {
            LabelledItem::Statement(node) => try_break!(self.visit(node)),
            LabelledItem::Function(_) => {}
        }
        ControlFlow::Continue(())
    }
}

/// Returns `true` if the given statement can optimize local variables.
#[must_use]
pub fn can_optimize_local_variables<'a, N>(node: &'a N) -> bool
where
    &'a N: Into<NodeRef<'a>>,
{
    CanOptimizeLocalVariables.visit(node.into()).is_continue()
}

/// The [`Visitor`] used for [`returns_value`].
#[derive(Debug)]
struct CanOptimizeLocalVariables;

impl<'ast> Visitor<'ast> for CanOptimizeLocalVariables {
    type BreakTy = ();

    fn visit_with(&mut self, _node: &'ast crate::statement::With) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Break(())
    }

    fn visit_call(&mut self, node: &'ast crate::expression::Call) -> ControlFlow<Self::BreakTy> {
        if let Expression::Identifier(identifier) = node.function() {
            if identifier.sym() == Sym::EVAL {
                return ControlFlow::Break(());
            }
        }

        try_break!(node.function().visit_with(self));

        for arg in node.args() {
            try_break!(arg.visit_with(self));
        }

        ControlFlow::Continue(())
    }

    fn visit_function(&mut self, _node: &'ast Function) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Break(())
    }

    fn visit_arrow_function(&mut self, _node: &'ast ArrowFunction) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Break(())
    }

    fn visit_async_function(&mut self, _node: &'ast AsyncFunction) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Break(())
    }

    fn visit_async_arrow_function(
        &mut self,
        _node: &'ast AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Break(())
    }

    fn visit_class(&mut self, _node: &'ast Class) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Break(())
    }
}
