use boa_interner::{Interner, Sym, ToInternedString};

use crate::string::ToStringEscaped;

use super::{
    expression::literal::Literal,
    function::{AsyncFunction, AsyncGenerator, FormalParameterList, Function, Generator},
    statement::StatementList,
    Expression,
};

/// A JavaScript property is a characteristic of an object, often describing attributes associated with a data structure.
///
/// A property has a name (a string) and a value (primitive, method, or object reference).
/// Note that when we say that "a property holds an object", that is shorthand for "a property holds an object reference".
/// This distinction matters because the original referenced object remains unchanged when you change the property's value.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-PropertyDefinition
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/property/JavaScript
// TODO: Support all features: https://tc39.es/ecma262/#prod-PropertyDefinition
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyDefinition {
    /// Puts a variable into an object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-IdentifierReference
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#Property_definitions
    IdentifierReference(Sym),

    /// Binds a property name to a JavaScript value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-PropertyDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#Property_definitions
    Property(PropertyName, Expression),

    /// A property of an object can also refer to a function or a getter or setter method.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#Method_definitions
    MethodDefinition(MethodDefinition, PropertyName),

    /// The Rest/Spread Properties for ECMAScript proposal (stage 4) adds spread properties to object literals.
    /// It copies own enumerable properties from a provided object onto a new object.
    ///
    /// Shallow-cloning (excluding `prototype`) or merging objects is now possible using a shorter syntax than `Object.assign()`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-PropertyDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#Spread_properties
    SpreadObject(Expression),

    /// Cover grammar for when an object literal is used as an object biding pattern.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-CoverInitializedName
    CoverInitializedName(Sym, Expression),
}

/// Method definition.
///
/// Starting with ECMAScript 2015, a shorter syntax for method definitions on objects initializers is introduced.
/// It is a shorthand for a function assigned to the method's name.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Method_definitions
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum MethodDefinition {
    /// The `get` syntax binds an object property to a function that will be called when that property is looked up.
    ///
    /// Sometimes it is desirable to allow access to a property that returns a dynamically computed value,
    /// or you may want to reflect the status of an internal variable without requiring the use of explicit method calls.
    /// In JavaScript, this can be accomplished with the use of a getter.
    ///
    /// It is not possible to simultaneously have a getter bound to a property and have that property actually hold a value,
    /// although it is possible to use a getter and a setter in conjunction to create a type of pseudo-property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/get
    Get(Function),

    /// The `set` syntax binds an object property to a function to be called when there is an attempt to set that property.
    ///
    /// In JavaScript, a setter can be used to execute a function whenever a specified property is attempted to be changed.
    /// Setters are most often used in conjunction with getters to create a type of pseudo-property.
    /// It is not possible to simultaneously have a setter on a property that holds an actual value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/set
    Set(Function),

    /// Starting with ECMAScript 2015, you are able to define own methods in a shorter syntax, similar to the getters and setters.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions#Method_definition_syntax
    Ordinary(Function),

    /// Starting with ECMAScript 2015, you are able to define own methods in a shorter syntax, similar to the getters and setters.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Method_definitions#generator_methods
    Generator(Generator),

    /// Async generators can be used to define a method
    ///
    /// More information
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorMethod
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Method_definitions#async_generator_methods
    AsyncGenerator(AsyncGenerator),

    /// Async function can be used to define a method
    ///
    /// More information
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AsyncMethod
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Method_definitions#async_methods
    Async(AsyncFunction),
}

impl MethodDefinition {
    /// Return the body of the method.
    pub(crate) fn body(&self) -> &StatementList {
        match self {
            MethodDefinition::Get(expr)
            | MethodDefinition::Set(expr)
            | MethodDefinition::Ordinary(expr) => expr.body(),
            MethodDefinition::Generator(expr) => expr.body(),
            MethodDefinition::AsyncGenerator(expr) => expr.body(),
            MethodDefinition::Async(expr) => expr.body(),
        }
    }

    /// Return the parameters of the method.
    pub(crate) fn parameters(&self) -> &FormalParameterList {
        match self {
            MethodDefinition::Get(expr)
            | MethodDefinition::Set(expr)
            | MethodDefinition::Ordinary(expr) => expr.parameters(),
            MethodDefinition::Generator(expr) => expr.parameters(),
            MethodDefinition::AsyncGenerator(expr) => expr.parameters(),
            MethodDefinition::Async(expr) => expr.parameters(),
        }
    }
}

/// `PropertyName` can be either a literal or computed.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-PropertyName
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyName {
    /// A `Literal` property name can be either an identifier, a string or a numeric literal.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-LiteralPropertyName
    Literal(Sym),

    /// A `Computed` property name is an expression that gets evaluated and converted into a property name.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ComputedPropertyName
    Computed(Expression),
}

impl PropertyName {
    /// Returns the literal property name if it exists.
    pub(crate) fn literal(&self) -> Option<Sym> {
        if let Self::Literal(sym) = self {
            Some(*sym)
        } else {
            None
        }
    }

    /// Returns the expression if the property name is computed.
    pub(crate) fn computed(&self) -> Option<&Expression> {
        if let Self::Computed(expr) = self {
            Some(expr)
        } else {
            None
        }
    }

    /// Returns either the literal property name or the computed const string property name.
    pub(in crate::syntax) fn prop_name(&self) -> Option<Sym> {
        match self {
            PropertyName::Literal(sym)
            | PropertyName::Computed(Expression::Literal(Literal::String(sym))) => Some(*sym),
            PropertyName::Computed(_) => None,
        }
    }
}

impl ToInternedString for PropertyName {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            PropertyName::Literal(key) => interner.resolve_expect(*key).join(
                String::from,
                ToStringEscaped::to_string_escaped,
                true,
            ),
            PropertyName::Computed(key) => key.to_interned_string(interner),
        }
    }
}

impl From<Sym> for PropertyName {
    fn from(name: Sym) -> Self {
        Self::Literal(name)
    }
}

impl From<Expression> for PropertyName {
    fn from(name: Expression) -> Self {
        Self::Computed(name)
    }
}

/// `ClassElementName` can be either a property name or a private identifier.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassElementName
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ClassElementName {
    PropertyName(PropertyName),
    PrivateIdentifier(Sym),
}
