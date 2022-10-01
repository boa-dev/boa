use boa_interner::{Interner, Sym, ToInternedString};

use self::{
    access::{PrivatePropertyAccess, PropertyAccess, SuperPropertyAccess},
    literal::{ArrayLiteral, Literal, ObjectLiteral, TemplateLiteral},
    operator::{conditional::Conditional, Assign, Binary, Unary},
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
mod spread;
mod tagged_template;
mod r#yield;

pub use call::{Call, SuperCall};
pub use identifier::Identifier;
pub use new::New;
pub use r#await::Await;
pub use r#yield::Yield;
pub use spread::Spread;
pub use tagged_template::TaggedTemplate;

pub mod access;
pub mod literal;
pub mod operator;

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

    /// See [`SuperPropertyAccess`]
    SuperPropertyAccess(SuperPropertyAccess),

    /// See [`PrivatePropertyAccess].
    PrivatePropertyAccess(PrivatePropertyAccess),

    /// See [`New`].
    New(New),

    /// See [`Call`].
    Call(Call),

    /// See [`SuperCall`]
    SuperCall(SuperCall),

    // TODO: Optional chains

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
    /// Creates a string of the value of the expression with the given indentation.
    pub fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        self.to_no_indent_string(interner, indentation)
    }

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
            Self::SuperPropertyAccess(supp) => supp.to_interned_string(interner),
            Self::PrivatePropertyAccess(private) => private.to_interned_string(interner),
            Self::New(new) => new.to_interned_string(interner),
            Self::Call(call) => call.to_interned_string(interner),
            Self::SuperCall(supc) => supc.to_interned_string(interner),
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
            Expression::SuperPropertyAccess(access) => access.contains_arguments(),
            Expression::PrivatePropertyAccess(access) => access.contains_arguments(),
            Expression::New(new) => new.contains_arguments(),
            Expression::Call(call) => call.contains_arguments(),
            Expression::SuperCall(call) => call.contains_arguments(),
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
            Expression::PropertyAccess(access) => access.contains(symbol),
            Expression::SuperPropertyAccess(_access) if symbol == ContainsSymbol::SuperProperty => {
                true
            }
            Expression::SuperPropertyAccess(access) => access.contains(symbol),
            Expression::PrivatePropertyAccess(access) => access.contains(symbol),
            Expression::New(new) => new.contains(symbol),
            Expression::Call(call) => call.contains(symbol),
            Expression::SuperCall(_) if symbol == ContainsSymbol::SuperCall => true,
            Expression::SuperCall(expr) => expr.contains(symbol),
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
    fn from(expr: Expression) -> Self {
        Statement::Expression(expr)
    }
}

impl ToInternedString for Expression {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}
