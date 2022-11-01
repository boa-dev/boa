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

use boa_interner::{Interner, Sym, ToIndentedString, ToInternedString};
use std::ops::ControlFlow;

use self::{
    access::PropertyAccess,
    literal::{ArrayLiteral, Literal, ObjectLiteral, TemplateLiteral},
    operator::{Assign, Binary, Conditional, Unary},
};

use super::{
    function::FormalParameterList,
    function::{ArrowFunction, AsyncFunction, AsyncGenerator, Class, Function, Generator},
    ContainsSymbol, Statement,
};

mod r#await;
mod call;
mod identifier;
mod new;
mod optional;
mod spread;
mod tagged_template;
mod r#yield;

use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
pub use call::{Call, SuperCall};
pub use identifier::Identifier;
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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
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
    /// It is not a valid AST node.
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

    /// Returns true if the expression contains a identifier reference named 'arguments'.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
    // TODO: replace with a visitor
    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            Expression::Identifier(ident) => *ident == Sym::ARGUMENTS,
            Expression::Function(_)
            | Expression::Generator(_)
            | Expression::AsyncFunction(_)
            | Expression::AsyncGenerator(_)
            | Expression::Literal(_)
            | Expression::This
            | Expression::NewTarget => false,
            Expression::ArrayLiteral(array) => array.contains_arguments(),
            Expression::ObjectLiteral(object) => object.contains_arguments(),
            Expression::Spread(spread) => spread.contains_arguments(),
            Expression::ArrowFunction(arrow) => arrow.contains_arguments(),
            Expression::Class(class) => class.contains_arguments(),
            Expression::TemplateLiteral(template) => template.contains_arguments(),
            Expression::PropertyAccess(access) => access.contains_arguments(),
            Expression::New(new) => new.contains_arguments(),
            Expression::Call(call) => call.contains_arguments(),
            Expression::SuperCall(call) => call.contains_arguments(),
            Expression::Optional(opt) => opt.contains_arguments(),
            Expression::TaggedTemplate(tag) => tag.contains_arguments(),
            Expression::Assign(assign) => assign.contains_arguments(),
            Expression::Unary(unary) => unary.contains_arguments(),
            Expression::Binary(binary) => binary.contains_arguments(),
            Expression::Conditional(cond) => cond.contains_arguments(),
            Expression::Await(r#await) => r#await.contains_arguments(),
            Expression::Yield(r#yield) => r#yield.contains_arguments(),
            // TODO: remove variant
            Expression::FormalParameterList(_) => unreachable!(),
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
            Expression::This => symbol == ContainsSymbol::This,
            Expression::Identifier(_)
            | Expression::Literal(_)
            | Expression::Function(_)
            | Expression::Generator(_)
            | Expression::AsyncFunction(_)
            | Expression::AsyncGenerator(_) => false,
            Expression::ArrayLiteral(array) => array.contains(symbol),
            Expression::ObjectLiteral(obj) => obj.contains(symbol),
            Expression::Spread(spread) => spread.contains(symbol),
            Expression::ArrowFunction(arrow) => arrow.contains(symbol),
            Expression::Class(class) => class.contains(symbol),
            Expression::TemplateLiteral(temp) => temp.contains(symbol),
            Expression::PropertyAccess(prop) => prop.contains(symbol),
            Expression::New(new) => new.contains(symbol),
            Expression::Call(call) => call.contains(symbol),
            Expression::SuperCall(_) if symbol == ContainsSymbol::SuperCall => true,
            Expression::SuperCall(expr) => expr.contains(symbol),
            Expression::Optional(opt) => opt.contains(symbol),
            Expression::TaggedTemplate(temp) => temp.contains(symbol),
            Expression::NewTarget => symbol == ContainsSymbol::NewTarget,
            Expression::Assign(assign) => assign.contains(symbol),
            Expression::Unary(unary) => unary.contains(symbol),
            Expression::Binary(binary) => binary.contains(symbol),
            Expression::Conditional(cond) => cond.contains(symbol),
            Expression::Await(_) if symbol == ContainsSymbol::AwaitExpression => true,
            Expression::Await(r#await) => r#await.contains(symbol),
            Expression::Yield(_) if symbol == ContainsSymbol::YieldExpression => true,
            Expression::Yield(r#yield) => r#yield.contains(symbol),
            Expression::FormalParameterList(_) => unreachable!(),
        }
    }
}

impl From<Expression> for Statement {
    #[inline]
    fn from(expr: Expression) -> Self {
        Statement::Expression(expr)
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
            Expression::Identifier(id) => visitor.visit_identifier(id),
            Expression::Literal(lit) => visitor.visit_literal(lit),
            Expression::ArrayLiteral(arlit) => visitor.visit_array_literal(arlit),
            Expression::ObjectLiteral(olit) => visitor.visit_object_literal(olit),
            Expression::Spread(sp) => visitor.visit_spread(sp),
            Expression::Function(f) => visitor.visit_function(f),
            Expression::ArrowFunction(af) => visitor.visit_arrow_function(af),
            Expression::Generator(g) => visitor.visit_generator(g),
            Expression::AsyncFunction(af) => visitor.visit_async_function(af),
            Expression::AsyncGenerator(ag) => visitor.visit_async_generator(ag),
            Expression::Class(c) => visitor.visit_class(&**c),
            Expression::TemplateLiteral(tlit) => visitor.visit_template_literal(tlit),
            Expression::PropertyAccess(pa) => visitor.visit_property_access(pa),
            Expression::New(n) => visitor.visit_new(n),
            Expression::Call(c) => visitor.visit_call(c),
            Expression::SuperCall(sc) => visitor.visit_super_call(sc),
            Expression::Optional(opt) => visitor.visit_optional(opt),
            Expression::TaggedTemplate(tt) => visitor.visit_tagged_template(tt),
            Expression::Assign(a) => visitor.visit_assign(a),
            Expression::Unary(u) => visitor.visit_unary(u),
            Expression::Binary(b) => visitor.visit_binary(b),
            Expression::Conditional(c) => visitor.visit_conditional(c),
            Expression::Await(a) => visitor.visit_await(a),
            Expression::Yield(y) => visitor.visit_yield(y),
            Expression::FormalParameterList(fpl) => visitor.visit_formal_parameter_list(fpl),
            Expression::This | Expression::NewTarget => {
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
            Expression::Identifier(id) => visitor.visit_identifier_mut(id),
            Expression::Literal(lit) => visitor.visit_literal_mut(lit),
            Expression::ArrayLiteral(arlit) => visitor.visit_array_literal_mut(arlit),
            Expression::ObjectLiteral(olit) => visitor.visit_object_literal_mut(olit),
            Expression::Spread(sp) => visitor.visit_spread_mut(sp),
            Expression::Function(f) => visitor.visit_function_mut(f),
            Expression::ArrowFunction(af) => visitor.visit_arrow_function_mut(af),
            Expression::Generator(g) => visitor.visit_generator_mut(g),
            Expression::AsyncFunction(af) => visitor.visit_async_function_mut(af),
            Expression::AsyncGenerator(ag) => visitor.visit_async_generator_mut(ag),
            Expression::Class(c) => visitor.visit_class_mut(&mut **c),
            Expression::TemplateLiteral(tlit) => visitor.visit_template_literal_mut(tlit),
            Expression::PropertyAccess(pa) => visitor.visit_property_access_mut(pa),
            Expression::New(n) => visitor.visit_new_mut(n),
            Expression::Call(c) => visitor.visit_call_mut(c),
            Expression::SuperCall(sc) => visitor.visit_super_call_mut(sc),
            Expression::Optional(opt) => visitor.visit_optional_mut(opt),
            Expression::TaggedTemplate(tt) => visitor.visit_tagged_template_mut(tt),
            Expression::Assign(a) => visitor.visit_assign_mut(a),
            Expression::Unary(u) => visitor.visit_unary_mut(u),
            Expression::Binary(b) => visitor.visit_binary_mut(b),
            Expression::Conditional(c) => visitor.visit_conditional_mut(c),
            Expression::Await(a) => visitor.visit_await_mut(a),
            Expression::Yield(y) => visitor.visit_yield_mut(y),
            Expression::FormalParameterList(fpl) => visitor.visit_formal_parameter_list_mut(fpl),
            Expression::This | Expression::NewTarget => {
                // do nothing; can be handled as special case by visitor
                ControlFlow::Continue(())
            }
        }
    }
}
