//! Object Expression.

use crate::{
    block_to_string,
    expression::{operator::assign::AssignTarget, Expression, RESERVED_IDENTIFIERS_STRICT},
    join_nodes,
    pattern::{ObjectPattern, ObjectPatternElement},
    property::{MethodDefinition, PropertyDefinition, PropertyName},
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, Sym, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;

/// Objects in JavaScript may be defined as an unordered collection of related data, of
/// primitive or reference types, in the form of “key: value” pairs.
///
/// Objects can be initialized using `new Object()`, `Object.create()`, or using the literal
/// notation.
///
/// An object initializer is an expression that describes the initialization of an
/// [`Object`][object]. Objects consist of properties, which are used to describe an object.
/// Values of object properties can either contain [`primitive`][primitive] data types or other
/// objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ObjectLiteral
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer
/// [object]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object
/// [primitive]: https://developer.mozilla.org/en-US/docs/Glossary/primitive
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectLiteral {
    properties: Box<[PropertyDefinition]>,
}

impl ObjectLiteral {
    /// Gets the object literal properties
    #[inline]
    #[must_use]
    pub const fn properties(&self) -> &[PropertyDefinition] {
        &self.properties
    }

    /// Converts the object literal into an [`ObjectPattern`].
    #[must_use]
    pub fn to_pattern(&self, strict: bool) -> Option<ObjectPattern> {
        let mut bindings = Vec::new();
        let mut excluded_keys = Vec::new();
        for (i, property) in self.properties.iter().enumerate() {
            match property {
                PropertyDefinition::IdentifierReference(ident) if strict && *ident == Sym::EVAL => {
                    return None
                }
                PropertyDefinition::IdentifierReference(ident) => {
                    if strict && RESERVED_IDENTIFIERS_STRICT.contains(&ident.sym()) {
                        return None;
                    }

                    excluded_keys.push(*ident);
                    bindings.push(ObjectPatternElement::SingleName {
                        ident: *ident,
                        name: PropertyName::Literal(ident.sym()),
                        default_init: None,
                    });
                }
                PropertyDefinition::Property(name, expr) => match (name, expr) {
                    (PropertyName::Literal(name), Expression::Identifier(ident))
                        if *name == *ident =>
                    {
                        if strict && *name == Sym::EVAL {
                            return None;
                        }
                        if strict && RESERVED_IDENTIFIERS_STRICT.contains(name) {
                            return None;
                        }

                        excluded_keys.push(*ident);
                        bindings.push(ObjectPatternElement::SingleName {
                            ident: *ident,
                            name: PropertyName::Literal(*name),
                            default_init: None,
                        });
                    }
                    (PropertyName::Literal(name), Expression::Identifier(ident)) => {
                        bindings.push(ObjectPatternElement::SingleName {
                            ident: *ident,
                            name: PropertyName::Literal(*name),
                            default_init: None,
                        });
                    }
                    (PropertyName::Literal(name), Expression::ObjectLiteral(object)) => {
                        let pattern = object.to_pattern(strict)?.into();
                        bindings.push(ObjectPatternElement::Pattern {
                            name: PropertyName::Literal(*name),
                            pattern,
                            default_init: None,
                        });
                    }
                    (PropertyName::Literal(name), Expression::ArrayLiteral(array)) => {
                        let pattern = array.to_pattern(strict)?.into();
                        bindings.push(ObjectPatternElement::Pattern {
                            name: PropertyName::Literal(*name),
                            pattern,
                            default_init: None,
                        });
                    }
                    (_, Expression::Assign(assign)) => match assign.lhs() {
                        AssignTarget::Identifier(ident) => {
                            if let Some(name) = name.literal() {
                                if name == *ident {
                                    if strict && name == Sym::EVAL {
                                        return None;
                                    }
                                    if strict && RESERVED_IDENTIFIERS_STRICT.contains(&name) {
                                        return None;
                                    }
                                    excluded_keys.push(*ident);
                                }
                                bindings.push(ObjectPatternElement::SingleName {
                                    ident: *ident,
                                    name: PropertyName::Literal(name),
                                    default_init: Some(assign.rhs().clone()),
                                });
                            } else {
                                return None;
                            }
                        }
                        AssignTarget::Pattern(pattern) => {
                            bindings.push(ObjectPatternElement::Pattern {
                                name: name.clone(),
                                pattern: pattern.clone(),
                                default_init: Some(assign.rhs().clone()),
                            });
                        }
                        AssignTarget::Access(access) => {
                            bindings.push(ObjectPatternElement::AssignmentPropertyAccess {
                                name: name.clone(),
                                access: access.clone(),
                                default_init: Some(assign.rhs().clone()),
                            });
                        }
                    },
                    (_, Expression::PropertyAccess(access)) => {
                        bindings.push(ObjectPatternElement::AssignmentPropertyAccess {
                            name: name.clone(),
                            access: access.clone(),
                            default_init: None,
                        });
                    }
                    (PropertyName::Computed(name), Expression::Identifier(ident)) => {
                        bindings.push(ObjectPatternElement::SingleName {
                            ident: *ident,
                            name: PropertyName::Computed(name.clone()),
                            default_init: None,
                        });
                    }
                    _ => return None,
                },
                PropertyDefinition::SpreadObject(spread) => {
                    match spread {
                        Expression::Identifier(ident) => {
                            bindings.push(ObjectPatternElement::RestProperty {
                                ident: *ident,
                                excluded_keys: excluded_keys.clone(),
                            });
                        }
                        Expression::PropertyAccess(access) => {
                            bindings.push(ObjectPatternElement::AssignmentRestPropertyAccess {
                                access: access.clone(),
                                excluded_keys: excluded_keys.clone(),
                            });
                        }
                        _ => return None,
                    }
                    if i + 1 != self.properties.len() {
                        return None;
                    }
                }
                PropertyDefinition::MethodDefinition(_, _) => return None,
                PropertyDefinition::CoverInitializedName(ident, expr) => {
                    if strict && [Sym::EVAL, Sym::ARGUMENTS].contains(&ident.sym()) {
                        return None;
                    }

                    bindings.push(ObjectPatternElement::SingleName {
                        ident: *ident,
                        name: PropertyName::Literal(ident.sym()),
                        default_init: Some(expr.clone()),
                    });
                }
            }
        }

        Some(ObjectPattern::new(bindings.into()))
    }
}

impl ToIndentedString for ObjectLiteral {
    fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let mut buf = "{\n".to_owned();
        let indentation = "    ".repeat(indent_n + 1);
        for property in self.properties().iter() {
            buf.push_str(&match property {
                PropertyDefinition::IdentifierReference(ident) => {
                    format!("{indentation}{},\n", interner.resolve_expect(ident.sym()))
                }
                PropertyDefinition::Property(key, value) => {
                    format!(
                        "{indentation}{}: {},\n",
                        key.to_interned_string(interner),
                        value.to_no_indent_string(interner, indent_n + 1)
                    )
                }
                PropertyDefinition::SpreadObject(key) => {
                    format!("{indentation}...{},\n", key.to_interned_string(interner))
                }
                PropertyDefinition::MethodDefinition(key, method) => {
                    format!(
                        "{indentation}{}{}({}) {},\n",
                        match &method {
                            MethodDefinition::Get(_) => "get ",
                            MethodDefinition::Set(_) => "set ",
                            _ => "",
                        },
                        key.to_interned_string(interner),
                        match &method {
                            MethodDefinition::Get(expression)
                            | MethodDefinition::Set(expression)
                            | MethodDefinition::Ordinary(expression) => {
                                join_nodes(interner, expression.parameters().as_ref())
                            }
                            MethodDefinition::Generator(expression) => {
                                join_nodes(interner, expression.parameters().as_ref())
                            }
                            MethodDefinition::AsyncGenerator(expression) => {
                                join_nodes(interner, expression.parameters().as_ref())
                            }
                            MethodDefinition::Async(expression) => {
                                join_nodes(interner, expression.parameters().as_ref())
                            }
                        },
                        match &method {
                            MethodDefinition::Get(expression)
                            | MethodDefinition::Set(expression)
                            | MethodDefinition::Ordinary(expression) => {
                                block_to_string(expression.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Generator(expression) => {
                                block_to_string(expression.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::AsyncGenerator(expression) => {
                                block_to_string(expression.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Async(expression) => {
                                block_to_string(expression.body(), interner, indent_n + 1)
                            }
                        },
                    )
                }
                PropertyDefinition::CoverInitializedName(ident, expr) => {
                    format!(
                        "{indentation}{} = {},\n",
                        interner.resolve_expect(ident.sym()),
                        expr.to_no_indent_string(interner, indent_n + 1)
                    )
                }
            });
        }
        buf.push_str(&format!("{}}}", "    ".repeat(indent_n)));

        buf
    }
}

impl<T> From<T> for ObjectLiteral
where
    T: Into<Box<[PropertyDefinition]>>,
{
    #[inline]
    fn from(props: T) -> Self {
        Self {
            properties: props.into(),
        }
    }
}

impl From<ObjectLiteral> for Expression {
    #[inline]
    fn from(obj: ObjectLiteral) -> Self {
        Self::ObjectLiteral(obj)
    }
}

impl VisitWith for ObjectLiteral {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        for pd in self.properties.iter() {
            try_break!(visitor.visit_property_definition(pd));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        for pd in self.properties.iter_mut() {
            try_break!(visitor.visit_property_definition_mut(pd));
        }
        ControlFlow::Continue(())
    }
}
