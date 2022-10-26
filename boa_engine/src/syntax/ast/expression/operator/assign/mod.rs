//! Assignment expression nodes, as defined by the [spec].
//!
//! An [assignment operator][mdn] assigns a value to its left operand based on the value of its right
//! operand. Almost any [`LeftHandSideExpression`][lhs] Parse Node can be the target of a simple
//! assignment expression (`=`). However, the compound assignment operations such as `%=` or `??=`
//! only allow ["simple"][simple] left hand side expressions as an assignment target.
//!
//! [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators
//! [lhs]: https://tc39.es/ecma262/#prod-LeftHandSideExpression
//! [simple]: https://tc39.es/ecma262/#sec-static-semantics-assignmenttargettype

mod op;

pub use op::*;

use boa_interner::{Interner, Sym, ToInternedString};

use crate::syntax::{
    ast::{
        expression::{
            access::PropertyAccess,
            identifier::Identifier,
            literal::{ArrayLiteral, ObjectLiteral},
            Expression,
        },
        pattern::{
            ArrayPattern, ArrayPatternElement, ObjectPattern, ObjectPatternElement, Pattern,
        },
        property::{PropertyDefinition, PropertyName},
        ContainsSymbol,
    },
    parser::RESERVED_IDENTIFIERS_STRICT,
};
/// An assignment operator expression.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Assign {
    op: AssignOp,
    lhs: Box<AssignTarget>,
    rhs: Box<Expression>,
}

impl Assign {
    /// Creates an `Assign` AST Expression.
    pub(in crate::syntax) fn new(op: AssignOp, lhs: AssignTarget, rhs: Expression) -> Self {
        Self {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Gets the operator of the assignment operation.
    #[inline]
    pub fn op(&self) -> AssignOp {
        self.op
    }

    /// Gets the left hand side of the assignment operation.
    #[inline]
    pub fn lhs(&self) -> &AssignTarget {
        &self.lhs
    }

    /// Gets the right hand side of the assignment operation.
    #[inline]
    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        (match &*self.lhs {
            AssignTarget::Identifier(ident) => *ident == Sym::ARGUMENTS,
            AssignTarget::Access(access) => access.contains_arguments(),
            AssignTarget::Pattern(pattern) => pattern.contains_arguments(),
        } || self.rhs.contains_arguments())
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        (match &*self.lhs {
            AssignTarget::Identifier(_) => false,
            AssignTarget::Access(access) => access.contains(symbol),
            AssignTarget::Pattern(pattern) => pattern.contains(symbol),
        } || self.rhs.contains(symbol))
    }
}

impl ToInternedString for Assign {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{} {} {}",
            self.lhs.to_interned_string(interner),
            self.op,
            self.rhs.to_interned_string(interner)
        )
    }
}

impl From<Assign> for Expression {
    #[inline]
    fn from(op: Assign) -> Self {
        Self::Assign(op)
    }
}

/// The valid left-hand-side expressions of an assignment operator. Also called
/// [`LeftHandSideExpression`][spec] in the spec.
///
/// [spec]: hhttps://tc39.es/ecma262/#prod-LeftHandSideExpression
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum AssignTarget {
    /// A simple identifier, such as `a`.
    Identifier(Identifier),
    /// A property access, such as `a.prop`.
    Access(PropertyAccess),
    /// A pattern assignment, such as `{a, b, ...c}`.
    Pattern(Pattern),
}

impl AssignTarget {
    /// Converts the left-hand-side Expression of an assignment expression into an [`AssignTarget`].
    /// Returns `None` if the given Expression is an invalid left-hand-side for a assignment expression.
    pub(crate) fn from_expression(
        expression: &Expression,
        strict: bool,
        destructure: bool,
    ) -> Option<Self> {
        match expression {
            Expression::Identifier(id) => Some(Self::Identifier(*id)),
            Expression::PropertyAccess(access) => Some(Self::Access(access.clone())),
            Expression::ObjectLiteral(object) if destructure => {
                let pattern = object_decl_to_declaration_pattern(object, strict)?;
                Some(Self::Pattern(pattern.into()))
            }
            Expression::ArrayLiteral(array) if destructure => {
                let pattern = array_decl_to_declaration_pattern(array, strict)?;
                Some(Self::Pattern(pattern.into()))
            }
            _ => None,
        }
    }
}

impl ToInternedString for AssignTarget {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            AssignTarget::Identifier(id) => id.to_interned_string(interner),
            AssignTarget::Access(access) => access.to_interned_string(interner),
            AssignTarget::Pattern(pattern) => pattern.to_interned_string(interner),
        }
    }
}

impl From<Identifier> for AssignTarget {
    #[inline]
    fn from(target: Identifier) -> Self {
        Self::Identifier(target)
    }
}

/// Converts an object literal into an object declaration pattern.
pub(crate) fn object_decl_to_declaration_pattern(
    object: &ObjectLiteral,
    strict: bool,
) -> Option<ObjectPattern> {
    let mut bindings = Vec::new();
    let mut excluded_keys = Vec::new();
    for (i, property) in object.properties().iter().enumerate() {
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
                (PropertyName::Literal(name), Expression::Identifier(ident)) if *name == *ident => {
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
                    let pattern = object_decl_to_declaration_pattern(object, strict)?.into();
                    bindings.push(ObjectPatternElement::Pattern {
                        name: PropertyName::Literal(*name),
                        pattern,
                        default_init: None,
                    });
                }
                (PropertyName::Literal(name), Expression::ArrayLiteral(array)) => {
                    let pattern = array_decl_to_declaration_pattern(array, strict)?.into();
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
                                bindings.push(ObjectPatternElement::SingleName {
                                    ident: *ident,
                                    name: PropertyName::Literal(name),
                                    default_init: Some(assign.rhs().clone()),
                                });
                            } else {
                                bindings.push(ObjectPatternElement::SingleName {
                                    ident: *ident,
                                    name: PropertyName::Literal(name),
                                    default_init: Some(assign.rhs().clone()),
                                });
                            }
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
                if i + 1 != object.properties().len() {
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

/// Converts an array declaration into an array declaration pattern.
pub(crate) fn array_decl_to_declaration_pattern(
    array: &ArrayLiteral,
    strict: bool,
) -> Option<ArrayPattern> {
    if array.has_trailing_comma_spread() {
        return None;
    }

    let mut bindings = Vec::new();
    for (i, expr) in array.as_ref().iter().enumerate() {
        let expr = if let Some(expr) = expr {
            expr
        } else {
            bindings.push(ArrayPatternElement::Elision);
            continue;
        };
        match expr {
            Expression::Identifier(ident) => {
                if strict && *ident == Sym::ARGUMENTS {
                    return None;
                }

                bindings.push(ArrayPatternElement::SingleName {
                    ident: *ident,
                    default_init: None,
                });
            }
            Expression::Spread(spread) => {
                match spread.target() {
                    Expression::Identifier(ident) => {
                        bindings.push(ArrayPatternElement::SingleNameRest { ident: *ident });
                    }
                    Expression::PropertyAccess(access) => {
                        bindings.push(ArrayPatternElement::PropertyAccessRest {
                            access: access.clone(),
                        });
                    }
                    Expression::ArrayLiteral(array) => {
                        let pattern = array_decl_to_declaration_pattern(array, strict)?.into();
                        bindings.push(ArrayPatternElement::PatternRest { pattern });
                    }
                    Expression::ObjectLiteral(object) => {
                        let pattern = object_decl_to_declaration_pattern(object, strict)?.into();
                        bindings.push(ArrayPatternElement::PatternRest { pattern });
                    }
                    _ => return None,
                }
                if i + 1 != array.as_ref().len() {
                    return None;
                }
            }
            Expression::Assign(assign) => match assign.lhs() {
                AssignTarget::Identifier(ident) => {
                    bindings.push(ArrayPatternElement::SingleName {
                        ident: *ident,
                        default_init: Some(assign.rhs().clone()),
                    });
                }
                AssignTarget::Access(access) => {
                    bindings.push(ArrayPatternElement::PropertyAccess {
                        access: access.clone(),
                    });
                }
                AssignTarget::Pattern(pattern) => match pattern {
                    Pattern::Object(pattern) => {
                        bindings.push(ArrayPatternElement::Pattern {
                            pattern: Pattern::Object(pattern.clone()),
                            default_init: Some(assign.rhs().clone()),
                        });
                    }
                    Pattern::Array(pattern) => {
                        bindings.push(ArrayPatternElement::Pattern {
                            pattern: Pattern::Array(pattern.clone()),
                            default_init: Some(assign.rhs().clone()),
                        });
                    }
                },
            },
            Expression::ArrayLiteral(array) => {
                let pattern = array_decl_to_declaration_pattern(array, strict)?.into();
                bindings.push(ArrayPatternElement::Pattern {
                    pattern,
                    default_init: None,
                });
            }
            Expression::ObjectLiteral(object) => {
                let pattern = object_decl_to_declaration_pattern(object, strict)?.into();
                bindings.push(ArrayPatternElement::Pattern {
                    pattern,
                    default_init: None,
                });
            }
            Expression::PropertyAccess(access) => {
                bindings.push(ArrayPatternElement::PropertyAccess {
                    access: access.clone(),
                });
            }
            _ => return None,
        }
    }
    Some(ArrayPattern::new(bindings.into()))
}
