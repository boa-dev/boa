//! Definitions of various **Syntax-Directed Operations** used in the [spec].
//!
//! [spec]: https://tc39.es/ecma262/#sec-syntax-directed-operations

use core::ops::ControlFlow;
use std::convert::Infallible;

use boa_interner::Sym;

use crate::{
    declaration::{Binding, Variable},
    expression::{access::SuperPropertyAccess, Await, Identifier, SuperCall, Yield},
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunction, AsyncGenerator, Class, ClassElement,
        Function, Generator,
    },
    property::{MethodDefinition, PropertyDefinition},
    statement::iteration::{ForLoopInitializer, IterableLoopInitializer},
    visitor::{NodeRef, VisitWith, Visitor},
    Declaration, Expression,
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
pub fn contains<N>(node: &N, symbol: ContainsSymbol) -> bool
where
    N: VisitWith,
{
    /// Visitor used by the function to search for a specific symbol in a node.
    #[derive(Debug, Clone, Copy)]
    struct ContainsVisitor(ContainsSymbol);

    impl<'ast> Visitor<'ast> for ContainsVisitor {
        type BreakTy = ();

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
    struct BoundNamesVisitor(Vec<Identifier>);

    impl<'ast> Visitor<'ast> for BoundNamesVisitor {
        type BreakTy = Infallible;

        fn visit_identifier(&mut self, node: &'ast Identifier) -> ControlFlow<Self::BreakTy> {
            self.0.push(*node);
            ControlFlow::Continue(())
        }
        fn visit_binding(&mut self, _: &'ast Binding) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_variable(&mut self, node: &'ast Variable) -> ControlFlow<Self::BreakTy> {
            node.binding().visit_with(self)
        }
        fn visit_iterable_loop_initializer(
            &mut self,
            node: &'ast IterableLoopInitializer,
        ) -> ControlFlow<Self::BreakTy> {
            match node {
                IterableLoopInitializer::Var(b)
                | IterableLoopInitializer::Let(b)
                | IterableLoopInitializer::Const(b) => b.visit_with(self),
                _ => ControlFlow::Continue(()),
            }
        }

        fn visit_for_loop_initializer(
            &mut self,
            node: &'ast ForLoopInitializer,
        ) -> ControlFlow<Self::BreakTy> {
            match node {
                ForLoopInitializer::Expression(_) => ControlFlow::Continue(()),
                ForLoopInitializer::Var(decl) => decl.visit_with(self),
                ForLoopInitializer::Lexical(decl) => decl.visit_with(self),
            }
        }

        fn visit_declaration(&mut self, node: &'ast Declaration) -> ControlFlow<Self::BreakTy> {
            // TODO: add "*default" for module default functions without name
            match node {
                Declaration::Function(f) => {
                    if let Some(ident) = f.name() {
                        self.0.push(ident);
                    }
                }
                Declaration::Generator(f) => {
                    if let Some(ident) = f.name() {
                        self.0.push(ident);
                    }
                }
                Declaration::AsyncFunction(f) => {
                    if let Some(ident) = f.name() {
                        self.0.push(ident);
                    }
                }
                Declaration::AsyncGenerator(f) => {
                    if let Some(ident) = f.name() {
                        self.0.push(ident);
                    }
                }
                Declaration::Class(f) => {
                    if let Some(ident) = f.name() {
                        self.0.push(ident);
                    }
                }
                Declaration::Lexical(decl) => {
                    decl.visit_with(self);
                }
            }
            ControlFlow::Continue(())
        }
    }

    let mut visitor = BoundNamesVisitor(Vec::new());

    visitor.visit(node.into());

    visitor.0
}
