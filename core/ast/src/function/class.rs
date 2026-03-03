use super::{FormalParameterList, FunctionBody, FunctionExpression};
use crate::{
    Declaration, LinearPosition, LinearSpan, LinearSpanIgnoreEq, Span, Spanned, block_to_string,
    expression::{Expression, Identifier},
    join_nodes,
    operations::{ContainsSymbol, contains},
    property::{MethodDefinitionKind, PropertyName},
    scope::{FunctionScopes, Scope},
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, Sym, ToIndentedString, ToInternedString};
use core::{fmt::Write as _, ops::ControlFlow};
use std::hash::Hash;

/// A class declaration.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassDeclaration
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClassDeclaration<'arena> {
    name: Identifier,
    pub(crate) super_ref: Option<Expression<'arena>>,
    pub(crate) constructor: Option<FunctionExpression<'arena>>,
    pub(crate) elements: Box<[ClassElement<'arena>]>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) name_scope: Scope,
}

impl<'arena> ClassDeclaration<'arena> {
    /// Creates a new class declaration.
    #[inline]
    #[must_use]
    pub fn new(
        name: Identifier,
        super_ref: Option<Expression<'arena>>,
        constructor: Option<FunctionExpression<'arena>>,
        elements: Box<[ClassElement<'arena>]>,
    ) -> Self {
        Self {
            name,
            super_ref,
            constructor,
            elements,
            name_scope: Scope::default(),
        }
    }

    /// Returns the name of the class declaration.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Identifier {
        self.name
    }

    /// Returns the super class ref of the class declaration.
    #[inline]
    #[must_use]
    pub const fn super_ref(&self) -> Option<&Expression<'arena>> {
        self.super_ref.as_ref()
    }

    /// Returns the constructor of the class declaration.
    #[inline]
    #[must_use]
    pub const fn constructor(&self) -> Option<&FunctionExpression<'arena>> {
        self.constructor.as_ref()
    }

    /// Gets the list of all fields defined on the class declaration.
    #[inline]
    #[must_use]
    pub const fn elements(&self) -> &[ClassElement<'arena>] {
        &self.elements
    }

    /// Gets the scope containing the class name binding.
    #[inline]
    #[must_use]
    pub const fn name_scope(&self) -> &Scope {
        &self.name_scope
    }
}

impl<'arena> ToIndentedString for ClassDeclaration<'arena> {
    fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let mut buf = format!("class {}", interner.resolve_expect(self.name.sym()));
        if let Some(super_ref) = self.super_ref.as_ref() {
            let _ = write!(buf, " extends {}", super_ref.to_interned_string(interner));
        }
        if self.elements.is_empty() && self.constructor().is_none() {
            buf.push_str(" {}");
            return buf;
        }
        let indentation = "    ".repeat(indent_n + 1);
        buf.push_str(" {\n");
        if let Some(expr) = &self.constructor {
            let _ = writeln!(
                buf,
                "{indentation}constructor({}) {}",
                join_nodes(interner, expr.parameters().as_ref()),
                block_to_string(&expr.body.statements, interner, indent_n + 1)
            );
        }
        for element in &self.elements {
            buf.push_str(&element.to_indented_string(interner, indent_n));
        }
        buf.push('}');
        buf
    }
}

impl<'arena> VisitWith<'arena> for ClassDeclaration<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_identifier(&self.name)?;
        if let Some(expr) = &self.super_ref {
            visitor.visit_expression(expr)?;
        }
        if let Some(func) = &self.constructor {
            visitor.visit_function_expression(func)?;
        }
        for elem in &*self.elements {
            visitor.visit_class_element(elem)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_identifier_mut(&mut self.name)?;
        if let Some(expr) = &mut self.super_ref {
            visitor.visit_expression_mut(expr)?;
        }
        if let Some(func) = &mut self.constructor {
            visitor.visit_function_expression_mut(func)?;
        }
        for elem in &mut *self.elements {
            visitor.visit_class_element_mut(elem)?;
        }
        ControlFlow::Continue(())
    }
}

impl<'arena> From<ClassDeclaration<'arena>> for Declaration<'arena> {
    fn from(f: ClassDeclaration<'arena>) -> Self {
        Self::ClassDeclaration(Box::new(f))
    }
}

/// A class expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClassExpression<'arena> {
    pub(crate) name: Option<Identifier>,
    pub(crate) super_ref: Option<Expression<'arena>>,
    pub(crate) constructor: Option<FunctionExpression<'arena>>,
    pub(crate) elements: Box<[ClassElement<'arena>]>,

    span: Span,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) name_scope: Option<Scope>,
}

impl<'arena> ClassExpression<'arena> {
    /// Creates a new class expression.
    #[inline]
    #[must_use]
    pub fn new(
        name: Option<Identifier>,
        super_ref: Option<Expression<'arena>>,
        constructor: Option<FunctionExpression<'arena>>,
        elements: Box<[ClassElement<'arena>]>,
        has_binding_identifier: bool,
        span: Span,
    ) -> Self {
        let name_scope = if has_binding_identifier {
            Some(Scope::default())
        } else {
            None
        };
        Self {
            name,
            super_ref,
            constructor,
            elements,
            span,
            name_scope,
        }
    }

    /// Returns the name of the class expression.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Returns the super class ref of the class expression.
    #[inline]
    #[must_use]
    pub const fn super_ref(&self) -> Option<&Expression<'arena>> {
        self.super_ref.as_ref()
    }

    /// Returns the constructor of the class expression.
    #[inline]
    #[must_use]
    pub const fn constructor(&self) -> Option<&FunctionExpression<'arena>> {
        self.constructor.as_ref()
    }

    /// Gets the list of all fields defined on the class expression.
    #[inline]
    #[must_use]
    pub const fn elements(&self) -> &[ClassElement<'arena>] {
        &self.elements
    }

    /// Gets the scope containing the class name binding if it exists.
    #[inline]
    #[must_use]
    pub const fn name_scope(&self) -> Option<&Scope> {
        self.name_scope.as_ref()
    }
}

impl<'arena> Spanned for ClassExpression<'arena> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> ToIndentedString for ClassExpression<'arena> {
    fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let mut buf = "class".to_string();
        if self.name_scope.is_some()
            && let Some(name) = self.name
        {
            let _ = write!(buf, " {}", interner.resolve_expect(name.sym()));
        }
        if let Some(super_ref) = self.super_ref.as_ref() {
            let _ = write!(buf, " extends {}", super_ref.to_interned_string(interner));
        }
        if self.elements.is_empty() && self.constructor().is_none() {
            buf.push_str(" {}");
            return buf;
        }
        let indentation = "    ".repeat(indent_n + 1);
        buf.push_str(" {\n");
        if let Some(expr) = &self.constructor {
            let _ = writeln!(
                buf,
                "{indentation}constructor({}) {}",
                join_nodes(interner, expr.parameters().as_ref()),
                block_to_string(&expr.body.statements, interner, indent_n + 1)
            );
        }
        for element in &self.elements {
            buf.push_str(&element.to_indented_string(interner, indent_n));
        }
        buf.push('}');
        buf
    }
}

impl<'arena> From<ClassExpression<'arena>> for Expression<'arena> {
    fn from(expr: ClassExpression<'arena>) -> Self {
        Self::ClassExpression(Box::new(expr))
    }
}

impl<'arena> VisitWith<'arena> for ClassExpression<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        if let Some(ident) = &self.name {
            visitor.visit_identifier(ident)?;
        }
        if let Some(expr) = &self.super_ref {
            visitor.visit_expression(expr)?;
        }
        if let Some(func) = &self.constructor {
            visitor.visit_function_expression(func)?;
        }
        for elem in &*self.elements {
            visitor.visit_class_element(elem)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        if let Some(ident) = &mut self.name {
            visitor.visit_identifier_mut(ident)?;
        }
        if let Some(expr) = &mut self.super_ref {
            visitor.visit_expression_mut(expr)?;
        }
        if let Some(func) = &mut self.constructor {
            visitor.visit_function_expression_mut(func)?;
        }
        for elem in &mut *self.elements {
            visitor.visit_class_element_mut(elem)?;
        }
        ControlFlow::Continue(())
    }
}

/// The body of a class' static block, as defined by the [spec].
///
/// Just an alias for [`Script`](crate::Script), since it has the same exact semantics.
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassStaticBlockBody
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct StaticBlockBody<'arena> {
    pub(crate) body: FunctionBody<'arena>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scopes: FunctionScopes,
}

impl<'arena> StaticBlockBody<'arena> {
    /// Creates a new static block body.
    #[inline]
    #[must_use]
    pub fn new(body: FunctionBody<'arena>) -> Self {
        Self {
            body,
            scopes: FunctionScopes::default(),
        }
    }

    /// Gets the body static block.
    #[inline]
    #[must_use]
    pub const fn statements(&self) -> &FunctionBody<'arena> {
        &self.body
    }

    /// Gets the scopes of the static block body.
    #[inline]
    #[must_use]
    pub const fn scopes(&self) -> &FunctionScopes {
        &self.scopes
    }
}

/// An element that can be within a class.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassElement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum ClassElement<'arena> {
    /// A method definition.
    MethodDefinition(ClassMethodDefinition<'arena>),

    /// A field definition.
    FieldDefinition(ClassFieldDefinition<'arena>),

    /// A static field definition, accessible from the class constructor object
    StaticFieldDefinition(ClassFieldDefinition<'arena>),

    /// A private field definition, only accessible inside the class declaration.
    PrivateFieldDefinition(PrivateFieldDefinition<'arena>),

    /// A private static field definition, only accessible from static methods and fields inside the
    /// class declaration.
    PrivateStaticFieldDefinition(PrivateFieldDefinition<'arena>),

    /// A static block, where a class can have initialization logic for its static fields.
    StaticBlock(StaticBlockBody<'arena>),
}

/// A non-private class element field definition.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FieldDefinition
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClassFieldDefinition<'arena> {
    pub(crate) name: PropertyName<'arena>,
    pub(crate) initializer: Option<Expression<'arena>>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scope: Scope,
}

impl<'arena> ClassFieldDefinition<'arena> {
    /// Creates a new class field definition.
    #[inline]
    #[must_use]
    pub fn new(name: PropertyName<'arena>, initializer: Option<Expression<'arena>>) -> Self {
        Self {
            name,
            initializer,
            scope: Scope::default(),
        }
    }

    /// Returns the name of the class field definition.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &PropertyName<'arena> {
        &self.name
    }

    /// Returns the initializer of the class field definition.
    #[inline]
    #[must_use]
    pub const fn initializer(&self) -> Option<&Expression<'arena>> {
        self.initializer.as_ref()
    }

    /// Returns the scope of the class field definition.
    #[inline]
    #[must_use]
    pub const fn scope(&self) -> &Scope {
        &self.scope
    }
}

/// A private class element field definition.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FieldDefinition
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct PrivateFieldDefinition<'arena> {
    pub(crate) name: PrivateName,
    pub(crate) initializer: Option<Expression<'arena>>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scope: Scope,
}

impl<'arena> PrivateFieldDefinition<'arena> {
    /// Creates a new private field definition.
    #[inline]
    #[must_use]
    pub fn new(name: PrivateName, initializer: Option<Expression<'arena>>) -> Self {
        Self {
            name,
            initializer,
            scope: Scope::default(),
        }
    }

    /// Returns the name of the private field definition.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &PrivateName {
        &self.name
    }

    /// Returns the initializer of the private field definition.
    #[inline]
    #[must_use]
    pub const fn initializer(&self) -> Option<&Expression<'arena>> {
        self.initializer.as_ref()
    }

    /// Returns the scope of the private field definition.
    #[inline]
    #[must_use]
    pub const fn scope(&self) -> &Scope {
        &self.scope
    }
}

impl<'arena> ToIndentedString for ClassElement<'arena> {
    fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let indentation = "    ".repeat(indent_n + 1);
        match self {
            Self::MethodDefinition(m) => m.to_indented_string(interner, indent_n),
            Self::FieldDefinition(field) => match &field.initializer {
                Some(expr) => {
                    format!(
                        "{indentation}{} = {};\n",
                        field.name.to_interned_string(interner),
                        expr.to_no_indent_string(interner, indent_n + 1)
                    )
                }
                None => {
                    format!(
                        "{indentation}{};\n",
                        field.name.to_interned_string(interner),
                    )
                }
            },
            Self::StaticFieldDefinition(field) => match &field.initializer {
                Some(expr) => {
                    format!(
                        "{indentation}static {} = {};\n",
                        field.name.to_interned_string(interner),
                        expr.to_no_indent_string(interner, indent_n + 1)
                    )
                }
                None => {
                    format!(
                        "{indentation}static {};\n",
                        field.name.to_interned_string(interner),
                    )
                }
            },
            Self::PrivateFieldDefinition(PrivateFieldDefinition {
                name, initializer, ..
            }) => match initializer {
                Some(expr) => {
                    format!(
                        "{indentation}#{} = {};\n",
                        interner.resolve_expect(name.description()),
                        expr.to_no_indent_string(interner, indent_n + 1)
                    )
                }
                None => {
                    format!(
                        "{indentation}#{};\n",
                        interner.resolve_expect(name.description()),
                    )
                }
            },
            Self::PrivateStaticFieldDefinition(PrivateFieldDefinition {
                name,
                initializer,
                ..
            }) => match initializer {
                Some(expr) => {
                    format!(
                        "{indentation}static #{} = {};\n",
                        interner.resolve_expect(name.description()),
                        expr.to_no_indent_string(interner, indent_n + 1)
                    )
                }
                None => {
                    format!(
                        "{indentation}static #{};\n",
                        interner.resolve_expect(name.description()),
                    )
                }
            },
            Self::StaticBlock(block) => {
                format!(
                    "{indentation}static {}\n",
                    block_to_string(&block.body.statements, interner, indent_n + 1)
                )
            }
        }
    }
}

impl<'arena> VisitWith<'arena> for ClassElement<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        match self {
            Self::MethodDefinition(m) => {
                match &m.name {
                    ClassElementName::PropertyName(pn) => {
                        visitor.visit_property_name(pn)?;
                    }
                    ClassElementName::PrivateName(pn) => {
                        visitor.visit_private_name(pn)?;
                    }
                }
                visitor.visit_formal_parameter_list(&m.parameters)?;
                visitor.visit_function_body(&m.body)
            }
            Self::FieldDefinition(field) | Self::StaticFieldDefinition(field) => {
                visitor.visit_property_name(&field.name)?;
                if let Some(expr) = &field.initializer {
                    visitor.visit_expression(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            Self::PrivateFieldDefinition(PrivateFieldDefinition {
                name, initializer, ..
            })
            | Self::PrivateStaticFieldDefinition(PrivateFieldDefinition {
                name,
                initializer,
                ..
            }) => {
                visitor.visit_private_name(name)?;
                if let Some(expr) = initializer {
                    visitor.visit_expression(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            Self::StaticBlock(block) => visitor.visit_function_body(&block.body),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        match self {
            Self::MethodDefinition(m) => {
                match m.name {
                    ClassElementName::PropertyName(ref mut pn) => {
                        visitor.visit_property_name_mut(pn)?;
                    }
                    ClassElementName::PrivateName(ref mut pn) => {
                        visitor.visit_private_name_mut(pn)?;
                    }
                }
                visitor.visit_formal_parameter_list_mut(&mut m.parameters)?;
                visitor.visit_function_body_mut(&mut m.body)
            }
            Self::FieldDefinition(field) | Self::StaticFieldDefinition(field) => {
                visitor.visit_property_name_mut(&mut field.name)?;
                if let Some(expr) = &mut field.initializer {
                    visitor.visit_expression_mut(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            Self::PrivateFieldDefinition(PrivateFieldDefinition {
                name, initializer, ..
            })
            | Self::PrivateStaticFieldDefinition(PrivateFieldDefinition {
                name,
                initializer,
                ..
            }) => {
                visitor.visit_private_name_mut(name)?;
                if let Some(expr) = initializer {
                    visitor.visit_expression_mut(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            Self::StaticBlock(block) => visitor.visit_function_body_mut(&mut block.body),
        }
    }
}

/// A method definition.
///
/// This type is specific to class method definitions.
/// It includes private names and the information about whether the method is static or not.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClassMethodDefinition<'arena> {
    name: ClassElementName<'arena>,
    pub(crate) parameters: FormalParameterList<'arena>,
    pub(crate) body: FunctionBody<'arena>,
    pub(crate) contains_direct_eval: bool,
    kind: MethodDefinitionKind,
    is_static: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scopes: FunctionScopes,
    linear_span: LinearSpanIgnoreEq,
}

impl<'arena> ClassMethodDefinition<'arena> {
    /// Creates a new class method definition.
    #[inline]
    #[must_use]
    pub fn new(
        name: ClassElementName<'arena>,
        parameters: FormalParameterList<'arena>,
        body: FunctionBody<'arena>,
        kind: MethodDefinitionKind,
        is_static: bool,
        start_linear_pos: LinearPosition,
    ) -> Self {
        let contains_direct_eval = contains(&parameters, ContainsSymbol::DirectEval)
            || contains(&body, ContainsSymbol::DirectEval);

        let linear_span = LinearSpan::new(start_linear_pos, body.linear_pos_end());

        Self {
            name,
            parameters,
            body,
            contains_direct_eval,
            kind,
            is_static,
            scopes: FunctionScopes::default(),
            linear_span: linear_span.into(),
        }
    }

    /// Returns the name of the class method definition.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &ClassElementName<'arena> {
        &self.name
    }

    /// Returns the parameters of the class method definition.
    #[inline]
    #[must_use]
    pub const fn parameters(&self) -> &FormalParameterList<'arena> {
        &self.parameters
    }

    /// Returns the body of the class method definition.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &FunctionBody<'arena> {
        &self.body
    }

    /// Returns the kind of the class method definition.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> MethodDefinitionKind {
        self.kind
    }

    /// Returns whether the class method definition is static.
    #[inline]
    #[must_use]
    pub const fn is_static(&self) -> bool {
        self.is_static
    }

    /// Returns whether the class method definition is private.
    #[inline]
    #[must_use]
    pub const fn is_private(&self) -> bool {
        self.name.is_private()
    }

    /// Gets the scopes of the class method definition.
    #[inline]
    #[must_use]
    pub const fn scopes(&self) -> &FunctionScopes {
        &self.scopes
    }

    /// Gets linear span of the function declaration.
    #[inline]
    #[must_use]
    pub const fn linear_span(&self) -> LinearSpan {
        self.linear_span.0
    }

    /// Returns `true` if the class method definition contains a direct call to `eval`.
    #[inline]
    #[must_use]
    pub const fn contains_direct_eval(&self) -> bool {
        self.contains_direct_eval
    }
}

impl<'arena> ToIndentedString for ClassMethodDefinition<'arena> {
    fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let indentation = "    ".repeat(indent_n + 1);
        let prefix = match (self.is_static, &self.kind) {
            (true, MethodDefinitionKind::Get) => "static get ",
            (true, MethodDefinitionKind::Set) => "static set ",
            (true, MethodDefinitionKind::Ordinary) => "static ",
            (true, MethodDefinitionKind::Generator) => "static *",
            (true, MethodDefinitionKind::AsyncGenerator) => "static async *",
            (true, MethodDefinitionKind::Async) => "static async ",
            (false, MethodDefinitionKind::Get) => "get ",
            (false, MethodDefinitionKind::Set) => "set ",
            (false, MethodDefinitionKind::Ordinary) => "",
            (false, MethodDefinitionKind::Generator) => "*",
            (false, MethodDefinitionKind::AsyncGenerator) => "async *",
            (false, MethodDefinitionKind::Async) => "async ",
        };
        let name = self.name.to_interned_string(interner);
        let parameters = join_nodes(interner, self.parameters.as_ref());
        let body = block_to_string(&self.body.statements, interner, indent_n + 1);
        format!("{indentation}{prefix}{name}({parameters}) {body}\n")
    }
}

/// A class element name.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassElementName
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum ClassElementName<'arena> {
    /// A property name.
    PropertyName(PropertyName<'arena>),

    /// A private name.
    PrivateName(PrivateName),
}

impl<'arena> ClassElementName<'arena> {
    /// Returns whether the class element name is private.
    #[inline]
    #[must_use]
    pub const fn is_private(&self) -> bool {
        matches!(self, Self::PrivateName(_))
    }
}

impl<'arena> ToInternedString for ClassElementName<'arena> {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match &self {
            Self::PropertyName(name) => name.to_interned_string(interner),
            Self::PrivateName(name) => format!("#{}", interner.resolve_expect(name.description())),
        }
    }
}

/// A private name as defined by the [spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-private-names
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PrivateName {
    /// The `[[Description]]` internal slot of the private name.
    description: Sym,
    span: Span,
}

impl PrivateName {
    /// Create a new private name.
    #[inline]
    #[must_use]
    pub const fn new(description: Sym, span: Span) -> Self {
        Self { description, span }
    }

    /// Get the description of the private name.
    #[inline]
    #[must_use]
    pub const fn description(&self) -> Sym {
        self.description
    }
}

impl Spanned for PrivateName {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> VisitWith<'arena> for PrivateName {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_sym(&self.description)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_sym_mut(&mut self.description)
    }
}
