//! The [`Expression`] Parse Node, as defined by the [spec].
//!
//! Javascript expressions include:
//! - [Primary][primary] expressions (`this`, function expressions, literals).
//! - [Left hand side][lhs] expressions (accessors, `new` operator, `super`).
//! - [operator] expressions.
//!
//! [spec]: https://tc39.es/ecma262/#prod-Expression
//! [primary]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#primary_expressions
//! [lhs]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#left-hand-side_expressions

use boa_interner::{Interner, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;

use self::{
    access::PropertyAccess,
    literal::{ArrayLiteral, Literal, ObjectLiteral, TemplateLiteral},
    operator::{Assign, Binary, Conditional, Unary},
};

use super::{
    function::{ArrowFunction, AsyncFunction, AsyncGenerator, Class, Function, Generator},
    function::{AsyncArrowFunction, FormalParameterList},
    Statement,
};

mod r#await;
mod call;
mod identifier;
mod new;
mod optional;
mod spread;
mod tagged_template;
mod r#yield;

use crate::visitor::{VisitWith, Visitor, VisitorMut};
pub use call::{Call, SuperCall};
pub use identifier::{Identifier, RESERVED_IDENTIFIERS_STRICT};
pub use new::New;
pub use optional::{Optional, OptionalOperation, OptionalOperationKind};
pub use r#await::Await;
pub use r#yield::Yield;
pub use spread::Spread;
pub use tagged_template::TaggedTemplate;

pub mod access;
pub mod literal;
pub mod operator;

/// The `Expression` Parse Node.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// The JavaScript `this` keyword refers to the object it belongs to.
    ///
    /// A property of an execution context (global, function or eval) that,
    /// in non–strict mode, is always a reference to an object and in strict
    /// mode can be any value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-this-keyword
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/this
    This,

    /// See [`Identifier`].
    Identifier(Identifier),

    /// See [`Literal`].
    Literal(Literal),

    /// See [`ArrayLiteral`].
    ArrayLiteral(ArrayLiteral),

    /// See [`ObjectLiteral`].
    ObjectLiteral(ObjectLiteral),

    /// See [`Spread`],
    Spread(Spread),

    /// See [`Function`].
    Function(Function),

    /// See [`ArrowFunction`].
    ArrowFunction(ArrowFunction),

    /// See [`AsyncArrowFunction`].
    AsyncArrowFunction(AsyncArrowFunction),

    /// See [`Generator`].
    Generator(Generator),

    /// See [`AsyncFunction`].
    AsyncFunction(AsyncFunction),

    /// See [`AsyncGenerator`].
    AsyncGenerator(AsyncGenerator),

    /// See [`Class`].
    Class(Box<Class>),

    // TODO: Extract regexp literal Expression
    // RegExpLiteral,
    /// See [`TemplateLiteral`].
    TemplateLiteral(TemplateLiteral),

    /// See [`PropertyAccess`].
    PropertyAccess(PropertyAccess),

    /// See [`New`].
    New(New),

    /// See [`Call`].
    Call(Call),

    /// See [`SuperCall`].
    SuperCall(SuperCall),

    /// See [`Optional`].
    Optional(Optional),

    // TODO: Import calls
    /// See [`TaggedTemplate`].
    TaggedTemplate(TaggedTemplate),

    /// The `new.target` pseudo-property expression.
    NewTarget,

    // TODO: import.meta
    /// See [`Assign`].
    Assign(Assign),

    /// See [`Unary`].
    Unary(Unary),

    /// See [`Binary`].
    Binary(Binary),

    /// See [`Conditional`].
    Conditional(Conditional),

    /// See [`Await`].
    Await(Await),

    /// See [`Yield`].
    Yield(Yield),

    /// A FormalParameterList.
    ///
    /// This is only used in the parser itself.
    /// It is not a valid expression node.
    #[doc(hidden)]
    FormalParameterList(FormalParameterList),
}

impl Expression {
    /// Implements the display formatting with indentation.
    ///
    /// This will not prefix the value with any indentation. If you want to prefix this with proper
    /// indents, use [`to_indented_string()`](Self::to_indented_string).
    pub(crate) fn to_no_indent_string(&self, interner: &Interner, indentation: usize) -> String {
        match self {
            Self::This => "this".to_owned(),
            Self::Identifier(id) => id.to_interned_string(interner),
            Self::Literal(lit) => lit.to_interned_string(interner),
            Self::ArrayLiteral(arr) => arr.to_interned_string(interner),
            Self::ObjectLiteral(o) => o.to_indented_string(interner, indentation),
            Self::Spread(sp) => sp.to_interned_string(interner),
            Self::Function(f) => f.to_indented_string(interner, indentation),
            Self::AsyncArrowFunction(f) => f.to_indented_string(interner, indentation),
            Self::ArrowFunction(arrf) => arrf.to_indented_string(interner, indentation),
            Self::Class(cl) => cl.to_indented_string(interner, indentation),
            Self::Generator(gen) => gen.to_indented_string(interner, indentation),
            Self::AsyncFunction(asf) => asf.to_indented_string(interner, indentation),
            Self::AsyncGenerator(asgen) => asgen.to_indented_string(interner, indentation),
            Self::TemplateLiteral(tem) => tem.to_interned_string(interner),
            Self::PropertyAccess(prop) => prop.to_interned_string(interner),
            Self::New(new) => new.to_interned_string(interner),
            Self::Call(call) => call.to_interned_string(interner),
            Self::SuperCall(supc) => supc.to_interned_string(interner),
            Self::Optional(opt) => opt.to_interned_string(interner),
            Self::NewTarget => "new.target".to_owned(),
            Self::TaggedTemplate(tag) => tag.to_interned_string(interner),
            Self::Assign(assign) => assign.to_interned_string(interner),
            Self::Unary(unary) => unary.to_interned_string(interner),
            Self::Binary(bin) => bin.to_interned_string(interner),
            Self::Conditional(cond) => cond.to_interned_string(interner),
            Self::Await(aw) => aw.to_interned_string(interner),
            Self::Yield(yi) => yi.to_interned_string(interner),
            Self::FormalParameterList(_) => unreachable!(),
        }
    }
}

impl From<Expression> for Statement {
    #[inline]
    fn from(expr: Expression) -> Self {
        Self::Expression(expr)
    }
}

impl ToIndentedString for Expression {
    #[inline]
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        self.to_no_indent_string(interner, indentation)
    }
}

impl VisitWith for Expression {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            Self::Identifier(id) => visitor.visit_identifier(id),
            Self::Literal(lit) => visitor.visit_literal(lit),
            Self::ArrayLiteral(arlit) => visitor.visit_array_literal(arlit),
            Self::ObjectLiteral(olit) => visitor.visit_object_literal(olit),
            Self::Spread(sp) => visitor.visit_spread(sp),
            Self::Function(f) => visitor.visit_function(f),
            Self::ArrowFunction(af) => visitor.visit_arrow_function(af),
            Self::AsyncArrowFunction(af) => visitor.visit_async_arrow_function(af),
            Self::Generator(g) => visitor.visit_generator(g),
            Self::AsyncFunction(af) => visitor.visit_async_function(af),
            Self::AsyncGenerator(ag) => visitor.visit_async_generator(ag),
            Self::Class(c) => visitor.visit_class(c),
            Self::TemplateLiteral(tlit) => visitor.visit_template_literal(tlit),
            Self::PropertyAccess(pa) => visitor.visit_property_access(pa),
            Self::New(n) => visitor.visit_new(n),
            Self::Call(c) => visitor.visit_call(c),
            Self::SuperCall(sc) => visitor.visit_super_call(sc),
            Self::Optional(opt) => visitor.visit_optional(opt),
            Self::TaggedTemplate(tt) => visitor.visit_tagged_template(tt),
            Self::Assign(a) => visitor.visit_assign(a),
            Self::Unary(u) => visitor.visit_unary(u),
            Self::Binary(b) => visitor.visit_binary(b),
            Self::Conditional(c) => visitor.visit_conditional(c),
            Self::Await(a) => visitor.visit_await(a),
            Self::Yield(y) => visitor.visit_yield(y),
            Self::FormalParameterList(fpl) => visitor.visit_formal_parameter_list(fpl),
            Self::This | Self::NewTarget => {
                // do nothing; can be handled as special case by visitor
                ControlFlow::Continue(())
            }
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            Self::Identifier(id) => visitor.visit_identifier_mut(id),
            Self::Literal(lit) => visitor.visit_literal_mut(lit),
            Self::ArrayLiteral(arlit) => visitor.visit_array_literal_mut(arlit),
            Self::ObjectLiteral(olit) => visitor.visit_object_literal_mut(olit),
            Self::Spread(sp) => visitor.visit_spread_mut(sp),
            Self::Function(f) => visitor.visit_function_mut(f),
            Self::ArrowFunction(af) => visitor.visit_arrow_function_mut(af),
            Self::AsyncArrowFunction(af) => visitor.visit_async_arrow_function_mut(af),
            Self::Generator(g) => visitor.visit_generator_mut(g),
            Self::AsyncFunction(af) => visitor.visit_async_function_mut(af),
            Self::AsyncGenerator(ag) => visitor.visit_async_generator_mut(ag),
            Self::Class(c) => visitor.visit_class_mut(c),
            Self::TemplateLiteral(tlit) => visitor.visit_template_literal_mut(tlit),
            Self::PropertyAccess(pa) => visitor.visit_property_access_mut(pa),
            Self::New(n) => visitor.visit_new_mut(n),
            Self::Call(c) => visitor.visit_call_mut(c),
            Self::SuperCall(sc) => visitor.visit_super_call_mut(sc),
            Self::Optional(opt) => visitor.visit_optional_mut(opt),
            Self::TaggedTemplate(tt) => visitor.visit_tagged_template_mut(tt),
            Self::Assign(a) => visitor.visit_assign_mut(a),
            Self::Unary(u) => visitor.visit_unary_mut(u),
            Self::Binary(b) => visitor.visit_binary_mut(b),
            Self::Conditional(c) => visitor.visit_conditional_mut(c),
            Self::Await(a) => visitor.visit_await_mut(a),
            Self::Yield(y) => visitor.visit_yield_mut(y),
            Self::FormalParameterList(fpl) => visitor.visit_formal_parameter_list_mut(fpl),
            Self::This | Self::NewTarget => {
                // do nothing; can be handled as special case by visitor
                ControlFlow::Continue(())
            }
        }
    }
}
