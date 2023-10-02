//! The [`Expression`] Parse Node, as defined by the [spec].
//!
//! ECMAScript expressions include:
//! - [Primary][primary] expressions (`this`, function expressions, literals).
//! - [Left hand side][lhs] expressions (accessors, `new` operator, `super`).
//! - [operator] expressions.
//!
//! [spec]: https://tc39.es/ecma262/#prod-Expression
//! [primary]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#primary_expressions
//! [lhs]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#left-hand-side_expressions

use self::{
    access::PropertyAccess,
    literal::{ArrayLiteral, Literal, ObjectLiteral, TemplateLiteral},
    operator::{Assign, Binary, BinaryInPrivate, Conditional, Unary, Update},
};
use super::{
    function::{ArrowFunction, AsyncFunction, AsyncGenerator, Class, Function, Generator},
    function::{AsyncArrowFunction, FormalParameterList},
    Statement,
};
use boa_interner::{Interner, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;

mod r#await;
mod call;
mod identifier;
mod new;
mod optional;
mod parenthesized;
mod regexp;
mod spread;
mod tagged_template;
mod r#yield;

use crate::visitor::{VisitWith, Visitor, VisitorMut};
pub use call::{Call, ImportCall, SuperCall};
pub use identifier::{Identifier, RESERVED_IDENTIFIERS_STRICT};
pub use new::New;
pub use optional::{Optional, OptionalOperation, OptionalOperationKind};
pub use parenthesized::Parenthesized;
pub use r#await::Await;
pub use r#yield::Yield;
pub use regexp::RegExpLiteral;
pub use spread::Spread;
pub use tagged_template::TaggedTemplate;

pub mod access;
pub mod literal;
pub mod operator;

/// The `Expression` Parse Node.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// The ECMAScript `this` keyword refers to the object it belongs to.
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

    /// See [`RegExpLiteral`].
    RegExpLiteral(RegExpLiteral),

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

    /// See [`ImportCall`].
    ImportCall(ImportCall),

    /// See [`Optional`].
    Optional(Optional),

    /// See [`TaggedTemplate`].
    TaggedTemplate(TaggedTemplate),

    /// The `new.target` pseudo-property expression.
    NewTarget,

    /// The `import.meta` pseudo-property expression.
    ImportMeta,

    /// See [`Assign`].
    Assign(Assign),

    /// See [`Unary`].
    Unary(Unary),

    /// See [`Unary`].
    Update(Update),

    /// See [`Binary`].
    Binary(Binary),

    /// See [`BinaryInPrivate`].
    BinaryInPrivate(BinaryInPrivate),

    /// See [`Conditional`].
    Conditional(Conditional),

    /// See [`Await`].
    Await(Await),

    /// See [`Yield`].
    Yield(Yield),

    /// See [`Parenthesized`].
    Parenthesized(Parenthesized),

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
            Self::ImportCall(impc) => impc.to_interned_string(interner),
            Self::Optional(opt) => opt.to_interned_string(interner),
            Self::NewTarget => "new.target".to_owned(),
            Self::ImportMeta => "import.meta".to_owned(),
            Self::TaggedTemplate(tag) => tag.to_interned_string(interner),
            Self::Assign(assign) => assign.to_interned_string(interner),
            Self::Unary(unary) => unary.to_interned_string(interner),
            Self::Update(update) => update.to_interned_string(interner),
            Self::Binary(bin) => bin.to_interned_string(interner),
            Self::BinaryInPrivate(bin) => bin.to_interned_string(interner),
            Self::Conditional(cond) => cond.to_interned_string(interner),
            Self::Await(aw) => aw.to_interned_string(interner),
            Self::Yield(yi) => yi.to_interned_string(interner),
            Self::Parenthesized(expr) => expr.to_interned_string(interner),
            Self::RegExpLiteral(regexp) => regexp.to_interned_string(interner),
            Self::FormalParameterList(_) => unreachable!(),
        }
    }

    /// Returns if the expression is a function definition according to the spec.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-isfunctiondefinition
    #[must_use]
    #[inline]
    pub const fn is_function_definition(&self) -> bool {
        matches!(
            self,
            Self::ArrowFunction(_)
                | Self::AsyncArrowFunction(_)
                | Self::Function(_)
                | Self::Generator(_)
                | Self::AsyncGenerator(_)
                | Self::AsyncFunction(_)
                | Self::Class(_)
        )
    }

    /// Returns if the expression is a function definition without a name.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isanonymousfunctiondefinition
    #[must_use]
    #[inline]
    pub const fn is_anonymous_function_definition(&self) -> bool {
        match self {
            Self::ArrowFunction(f) => f.name().is_none(),
            Self::AsyncArrowFunction(f) => f.name().is_none(),
            Self::Function(f) => f.name().is_none(),
            Self::Generator(f) => f.name().is_none(),
            Self::AsyncGenerator(f) => f.name().is_none(),
            Self::AsyncFunction(f) => f.name().is_none(),
            Self::Class(f) => f.name().is_none(),
            Self::Parenthesized(p) => p.expression().is_anonymous_function_definition(),
            _ => false,
        }
    }

    /// Returns the expression without any outer parenthesized expressions.
    #[must_use]
    #[inline]
    pub const fn flatten(&self) -> &Self {
        let mut expression = self;
        while let Self::Parenthesized(p) = expression {
            expression = p.expression();
        }
        expression
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
            Self::RegExpLiteral(regexp) => visitor.visit_reg_exp_literal(regexp),
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
            Self::ImportCall(ic) => visitor.visit_import_call(ic),
            Self::Optional(opt) => visitor.visit_optional(opt),
            Self::TaggedTemplate(tt) => visitor.visit_tagged_template(tt),
            Self::Assign(a) => visitor.visit_assign(a),
            Self::Unary(u) => visitor.visit_unary(u),
            Self::Update(u) => visitor.visit_update(u),
            Self::Binary(b) => visitor.visit_binary(b),
            Self::BinaryInPrivate(b) => visitor.visit_binary_in_private(b),
            Self::Conditional(c) => visitor.visit_conditional(c),
            Self::Await(a) => visitor.visit_await(a),
            Self::Yield(y) => visitor.visit_yield(y),
            Self::Parenthesized(e) => visitor.visit_parenthesized(e),
            Self::FormalParameterList(fpl) => visitor.visit_formal_parameter_list(fpl),
            Self::This | Self::NewTarget | Self::ImportMeta => {
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
            Self::RegExpLiteral(regexp) => visitor.visit_reg_exp_literal_mut(regexp),
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
            Self::ImportCall(ic) => visitor.visit_import_call_mut(ic),
            Self::Optional(opt) => visitor.visit_optional_mut(opt),
            Self::TaggedTemplate(tt) => visitor.visit_tagged_template_mut(tt),
            Self::Assign(a) => visitor.visit_assign_mut(a),
            Self::Unary(u) => visitor.visit_unary_mut(u),
            Self::Update(u) => visitor.visit_update_mut(u),
            Self::Binary(b) => visitor.visit_binary_mut(b),
            Self::BinaryInPrivate(b) => visitor.visit_binary_in_private_mut(b),
            Self::Conditional(c) => visitor.visit_conditional_mut(c),
            Self::Await(a) => visitor.visit_await_mut(a),
            Self::Yield(y) => visitor.visit_yield_mut(y),
            Self::Parenthesized(e) => visitor.visit_parenthesized_mut(e),
            Self::FormalParameterList(fpl) => visitor.visit_formal_parameter_list_mut(fpl),
            Self::This | Self::NewTarget | Self::ImportMeta => {
                // do nothing; can be handled as special case by visitor
                ControlFlow::Continue(())
            }
        }
    }
}
