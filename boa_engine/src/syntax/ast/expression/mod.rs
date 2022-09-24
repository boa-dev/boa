use boa_interner::{Interner, Sym, ToInternedString};

use self::{
    access::{PrivatePropertyAccess, PropertyAccess, PropertyAccessField, SuperPropertyAccess},
    literal::{ArrayLiteral, Literal, ObjectLiteral, TemplateElement, TemplateLiteral},
    operator::{assign::AssignTarget, conditional::Conditional, Assign, Binary, Unary},
};

use super::{
    function::FormalParameterList,
    function::{
        ArrowFunction, AsyncFunction, AsyncGenerator, Class, ClassElement, Function, Generator,
    },
    property::{MethodDefinition, PropertyDefinition},
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
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            Expression::Identifier(ident) if ident.sym() == Sym::ARGUMENTS => return true,
            Expression::ArrayLiteral(array) => {
                for expr in array.as_ref() {
                    if matches!(expr, Some(expr) if expr.contains_arguments()) {
                        return true;
                    }
                }
            }
            Expression::ObjectLiteral(object) => {
                for property in object.properties() {
                    match property {
                        PropertyDefinition::IdentifierReference(ident) => {
                            if *ident == Sym::ARGUMENTS {
                                return true;
                            }
                        }
                        PropertyDefinition::Property(_, node)
                        | PropertyDefinition::SpreadObject(node) => {
                            if node.contains_arguments() {
                                return true;
                            }
                        }
                        PropertyDefinition::MethodDefinition(method, _) => match method {
                            MethodDefinition::Get(function)
                            | MethodDefinition::Set(function)
                            | MethodDefinition::Ordinary(function) => {
                                if let Some(Sym::ARGUMENTS) = function.name() {
                                    return true;
                                }
                            }
                            MethodDefinition::Generator(generator) => {
                                if let Some(Sym::ARGUMENTS) = generator.name() {
                                    return true;
                                }
                            }
                            MethodDefinition::AsyncGenerator(async_generator) => {
                                if let Some(Sym::ARGUMENTS) = async_generator.name() {
                                    return true;
                                }
                            }
                            MethodDefinition::Async(function) => {
                                if let Some(Sym::ARGUMENTS) = function.name() {
                                    return true;
                                }
                            }
                        },
                        PropertyDefinition::CoverInitializedName(_, _) => {}
                    }
                }
            }
            Expression::Spread(spread) => {
                if spread.val().contains_arguments() {
                    return true;
                }
            }
            Expression::Assign(assign) => {
                if assign.rhs().contains_arguments() {
                    return true;
                }
            }
            Expression::Await(r#await) => {
                if r#await.expr().contains_arguments() {
                    return true;
                }
            }
            Expression::Binary(bin_op) => {
                if bin_op.lhs().contains_arguments() || bin_op.rhs().contains_arguments() {
                    return true;
                }
            }
            Expression::Call(call) => {
                if call.expr().contains_arguments() {
                    return true;
                }
                for node in call.args() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Expression::Conditional(conditional) => {
                if conditional.cond().contains_arguments() {
                    return true;
                }
                if conditional.if_true().contains_arguments() {
                    return true;
                }
                if conditional.if_false().contains_arguments() {
                    return true;
                }
            }
            Expression::PropertyAccess(access) => {
                if access.target().contains_arguments() {
                    return true;
                }
                if let PropertyAccessField::Expr(expr) = access.field() {
                    if expr.contains_arguments() {
                        return true;
                    }
                }
            }
            Expression::PrivatePropertyAccess(access) => {
                if access.target().contains_arguments() {
                    return true;
                }
            }
            Expression::New(new) => {
                if new.expr().contains_arguments() {
                    return true;
                }
                for node in new.args() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Expression::TaggedTemplate(tagged_template) => {
                if tagged_template.tag().contains_arguments() {
                    return true;
                }
                for node in tagged_template.exprs() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Expression::TemplateLiteral(template_lit) => {
                for element in template_lit.elements() {
                    if let TemplateElement::Expr(node) = element {
                        if node.contains_arguments() {
                            return false;
                        }
                    }
                }
            }
            Expression::Unary(unary_op) => {
                if unary_op.target().contains_arguments() {
                    return true;
                }
            }
            Expression::Yield(r#yield) => {
                if let Some(node) = r#yield.expr() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Expression::Class(class) => {
                if let Some(node) = class.super_ref() {
                    if node.contains_arguments() {
                        return true;
                    }
                    for element in class.elements() {
                        match element {
                            ClassElement::MethodDefinition(_, method)
                            | ClassElement::StaticMethodDefinition(_, method) => match method {
                                MethodDefinition::Get(function)
                                | MethodDefinition::Set(function)
                                | MethodDefinition::Ordinary(function) => {
                                    if let Some(Sym::ARGUMENTS) = function.name() {
                                        return true;
                                    }
                                }
                                MethodDefinition::Generator(generator) => {
                                    if let Some(Sym::ARGUMENTS) = generator.name() {
                                        return true;
                                    }
                                }
                                MethodDefinition::AsyncGenerator(async_generator) => {
                                    if let Some(Sym::ARGUMENTS) = async_generator.name() {
                                        return true;
                                    }
                                }
                                MethodDefinition::Async(function) => {
                                    if let Some(Sym::ARGUMENTS) = function.name() {
                                        return true;
                                    }
                                }
                            },
                            ClassElement::FieldDefinition(_, node)
                            | ClassElement::StaticFieldDefinition(_, node)
                            | ClassElement::PrivateFieldDefinition(_, node)
                            | ClassElement::PrivateStaticFieldDefinition(_, node) => {
                                if let Some(node) = node {
                                    if node.contains_arguments() {
                                        return true;
                                    }
                                }
                            }
                            ClassElement::StaticBlock(statement_list) => {
                                for node in statement_list.statements() {
                                    if node.contains_arguments() {
                                        return true;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        false
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
            Expression::ArrayLiteral(array) => {
                for expr in array.as_ref().iter().flatten() {
                    if expr.contains(symbol) {
                        return true;
                    }
                }
            }
            Expression::Assign(assign) => {
                match assign.lhs() {
                    AssignTarget::Property(field) => {
                        if field.target().contains(symbol) {
                            return true;
                        }
                        match field.field() {
                            PropertyAccessField::Expr(expr) if expr.contains(symbol) => {
                                return true
                            }
                            _ => {}
                        }
                    }
                    AssignTarget::PrivateProperty(access) => {
                        if access.target().contains(symbol) {
                            return true;
                        }
                    }
                    AssignTarget::SuperProperty(_) => {
                        if symbol == ContainsSymbol::SuperProperty {
                            return true;
                        }
                    }
                    AssignTarget::Pattern(pattern) => {
                        if pattern.contains(symbol) {
                            return true;
                        }
                    }
                    AssignTarget::Identifier(_) => {}
                }
                if assign.rhs().contains(symbol) {
                    return true;
                }
            }
            Expression::Await(_) if symbol == ContainsSymbol::AwaitExpression => return true,
            Expression::Await(expr) => {
                if expr.expr().contains(symbol) {
                    return true;
                }
            }
            Expression::Binary(bin_op) => {
                if bin_op.lhs().contains(symbol) || bin_op.rhs().contains(symbol) {
                    return true;
                }
            }
            Expression::Call(call) => {
                if call.expr().contains(symbol) {
                    return true;
                }
                for node in call.args() {
                    if node.contains(symbol) {
                        return true;
                    }
                }
            }
            Expression::New(new) => {
                if new.call().expr().contains(symbol) {
                    return true;
                }
                for node in new.call().args() {
                    if node.contains(symbol) {
                        return true;
                    }
                }
            }
            Expression::Spread(spread) => {
                if spread.val().contains(symbol) {
                    return true;
                }
            }
            Expression::TaggedTemplate(template) => {
                if template.tag().contains(symbol) {
                    return true;
                }
                for node in template.exprs() {
                    if node.contains(symbol) {
                        return true;
                    }
                }
            }
            Expression::TemplateLiteral(template) => {
                for element in template.elements() {
                    if let TemplateElement::Expr(node) = element {
                        if node.contains(symbol) {
                            return true;
                        }
                    }
                }
            }
            Expression::SuperCall(_) if symbol == ContainsSymbol::SuperCall => return true,
            Expression::SuperPropertyAccess(_) if symbol == ContainsSymbol::SuperProperty => {
                return true
            }
            Expression::ObjectLiteral(object) => {
                for property in object.properties() {
                    match property {
                        PropertyDefinition::Property(name, init) => {
                            if let Some(node) = name.computed() {
                                if node.contains(symbol) {
                                    return true;
                                }
                            }
                            if init.contains(symbol) {
                                return true;
                            }
                        }
                        PropertyDefinition::SpreadObject(spread) => {
                            if spread.contains(symbol) {
                                return true;
                            }
                        }
                        PropertyDefinition::MethodDefinition(_, name) => {
                            if let Some(node) = name.computed() {
                                if node.contains(symbol) {
                                    return true;
                                }
                            }
                        }
                        PropertyDefinition::IdentifierReference(_)
                        | PropertyDefinition::CoverInitializedName(_, _) => {}
                    }
                }
            }
            Expression::Class(class) => {
                if let Some(node) = class.super_ref() {
                    if node.contains(symbol) {
                        return true;
                    }
                }
                for element in class.elements() {
                    match element {
                        ClassElement::MethodDefinition(name, _)
                        | ClassElement::StaticMethodDefinition(name, _)
                        | ClassElement::FieldDefinition(name, _)
                        | ClassElement::StaticFieldDefinition(name, _) => {
                            if let Some(node) = name.computed() {
                                if node.contains(symbol) {
                                    return true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Expression::Yield(_) if symbol == ContainsSymbol::YieldExpression => return true,
            _ => {}
        }
        false
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
