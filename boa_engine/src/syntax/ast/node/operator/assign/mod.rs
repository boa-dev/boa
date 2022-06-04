use crate::syntax::{
    ast::node::{
        declaration::{
            BindingPatternTypeArray, BindingPatternTypeObject, DeclarationPatternArray,
            DeclarationPatternObject,
        },
        field::get_private_field::GetPrivateField,
        object::{PropertyDefinition, PropertyName},
        ArrayDecl, DeclarationPattern, GetConstField, GetField, Identifier, Node, Object,
    },
    parser::RESERVED_IDENTIFIERS_STRICT,
};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// An assignment operator assigns a value to its left operand based on the value of its right
/// operand.
///
/// Assignment operator (`=`), assigns the value of its right operand to its left operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Assign {
    lhs: Box<AssignTarget>,
    rhs: Box<Node>,
}

impl Assign {
    /// Creates an `Assign` AST node.
    pub(in crate::syntax) fn new<L, R>(lhs: L, rhs: R) -> Self
    where
        L: Into<AssignTarget>,
        R: Into<Node>,
    {
        Self {
            lhs: Box::new(lhs.into()),
            rhs: Box::new(rhs.into()),
        }
    }

    /// Gets the left hand side of the assignment operation.
    pub fn lhs(&self) -> &AssignTarget {
        &self.lhs
    }

    /// Gets the right hand side of the assignment operation.
    pub fn rhs(&self) -> &Node {
        &self.rhs
    }
}

impl ToInternedString for Assign {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{} = {}",
            self.lhs.to_interned_string(interner),
            self.rhs.to_interned_string(interner)
        )
    }
}

impl From<Assign> for Node {
    fn from(op: Assign) -> Self {
        Self::Assign(op)
    }
}

/// This type represents all valid left-had-side expressions of an assignment operator.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum AssignTarget {
    Identifier(Identifier),
    GetPrivateField(GetPrivateField),
    GetConstField(GetConstField),
    GetField(GetField),
    DeclarationPattern(DeclarationPattern),
}

impl AssignTarget {
    /// Converts the left-hand-side node of an assignment expression into it's an [`AssignTarget`].
    /// Returns `None` if the given node is an invalid left-hand-side for a assignment expression.
    pub(crate) fn from_node(node: &Node, strict: bool) -> Option<Self> {
        match node {
            Node::Identifier(target) => Some(Self::Identifier(*target)),
            Node::GetPrivateField(target) => Some(Self::GetPrivateField(target.clone())),
            Node::GetConstField(target) => Some(Self::GetConstField(target.clone())),
            Node::GetField(target) => Some(Self::GetField(target.clone())),
            Node::Object(object) => {
                let pattern = object_decl_to_declaration_pattern(object, strict)?;
                Some(Self::DeclarationPattern(pattern))
            }
            Node::ArrayDecl(array) => {
                let pattern = array_decl_to_declaration_pattern(array, strict)?;
                Some(Self::DeclarationPattern(pattern))
            }
            _ => None,
        }
    }
}

impl ToInternedString for AssignTarget {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            AssignTarget::Identifier(target) => target.to_interned_string(interner),
            AssignTarget::GetPrivateField(target) => target.to_interned_string(interner),
            AssignTarget::GetConstField(target) => target.to_interned_string(interner),
            AssignTarget::GetField(target) => target.to_interned_string(interner),
            AssignTarget::DeclarationPattern(target) => target.to_interned_string(interner),
        }
    }
}

impl From<Identifier> for AssignTarget {
    fn from(target: Identifier) -> Self {
        Self::Identifier(target)
    }
}

impl From<GetConstField> for AssignTarget {
    fn from(target: GetConstField) -> Self {
        Self::GetConstField(target)
    }
}

impl From<GetField> for AssignTarget {
    fn from(target: GetField) -> Self {
        Self::GetField(target)
    }
}

/// Converts an object literal into an object declaration pattern.
pub(crate) fn object_decl_to_declaration_pattern(
    object: &Object,
    strict: bool,
) -> Option<DeclarationPattern> {
    let mut bindings = Vec::new();
    let mut excluded_keys = Vec::new();
    for (i, property) in object.properties().iter().enumerate() {
        match property {
            PropertyDefinition::IdentifierReference(ident) if strict && *ident == Sym::EVAL => {
                return None
            }
            PropertyDefinition::IdentifierReference(ident) => {
                if strict && RESERVED_IDENTIFIERS_STRICT.contains(ident) {
                    return None;
                }

                excluded_keys.push(*ident);
                bindings.push(BindingPatternTypeObject::SingleName {
                    ident: *ident,
                    property_name: PropertyName::Literal(*ident),
                    default_init: None,
                });
            }
            PropertyDefinition::Property(name, node) => match (name, node) {
                (PropertyName::Literal(name), Node::Identifier(ident)) if *name == ident.sym() => {
                    if strict && *name == Sym::EVAL {
                        return None;
                    }
                    if strict && RESERVED_IDENTIFIERS_STRICT.contains(name) {
                        return None;
                    }

                    excluded_keys.push(*name);
                    bindings.push(BindingPatternTypeObject::SingleName {
                        ident: *name,
                        property_name: PropertyName::Literal(*name),
                        default_init: None,
                    });
                }
                _ => return None,
            },
            PropertyDefinition::SpreadObject(spread) => {
                match spread {
                    Node::Identifier(ident) => {
                        bindings.push(BindingPatternTypeObject::RestProperty {
                            ident: ident.sym(),
                            excluded_keys: excluded_keys.clone(),
                        });
                    }
                    Node::GetConstField(get_const_field) => {
                        bindings.push(BindingPatternTypeObject::RestGetConstField {
                            get_const_field: get_const_field.clone(),
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
        }
    }
    if object.properties().is_empty() {
        bindings.push(BindingPatternTypeObject::Empty);
    }
    Some(DeclarationPattern::Object(DeclarationPatternObject::new(
        bindings, None,
    )))
}

/// Converts an array declaration into an array declaration pattern.
pub(crate) fn array_decl_to_declaration_pattern(
    array: &ArrayDecl,
    strict: bool,
) -> Option<DeclarationPattern> {
    if array.has_trailing_comma_spread() {
        return None;
    }

    let mut bindings = Vec::new();
    for (i, node) in array.as_ref().iter().enumerate() {
        match node {
            Node::Identifier(ident) => {
                if strict && ident.sym() == Sym::ARGUMENTS {
                    return None;
                }

                bindings.push(BindingPatternTypeArray::SingleName {
                    ident: ident.sym(),
                    default_init: None,
                });
            }
            Node::Spread(spread) => {
                match spread.val() {
                    Node::Identifier(ident) => {
                        bindings
                            .push(BindingPatternTypeArray::SingleNameRest { ident: ident.sym() });
                    }
                    Node::GetField(get_field) => {
                        bindings.push(BindingPatternTypeArray::GetFieldRest {
                            get_field: get_field.clone(),
                        });
                    }
                    Node::GetConstField(get_const_field) => {
                        bindings.push(BindingPatternTypeArray::GetConstFieldRest {
                            get_const_field: get_const_field.clone(),
                        });
                    }
                    Node::ArrayDecl(array) => {
                        let pattern = array_decl_to_declaration_pattern(array, strict)?;
                        bindings.push(BindingPatternTypeArray::BindingPatternRest { pattern });
                    }
                    Node::Object(object) => {
                        let pattern = object_decl_to_declaration_pattern(object, strict)?;
                        bindings.push(BindingPatternTypeArray::BindingPatternRest { pattern });
                    }
                    _ => return None,
                }
                if i + 1 != array.as_ref().len() {
                    return None;
                }
            }
            Node::Empty => {
                bindings.push(BindingPatternTypeArray::Elision);
            }
            Node::Assign(assign) => match assign.lhs() {
                AssignTarget::Identifier(ident) => {
                    bindings.push(BindingPatternTypeArray::SingleName {
                        ident: ident.sym(),
                        default_init: Some(assign.rhs().clone()),
                    });
                }
                AssignTarget::GetConstField(get_const_field) => {
                    bindings.push(BindingPatternTypeArray::GetConstField {
                        get_const_field: get_const_field.clone(),
                    });
                }
                AssignTarget::GetField(get_field) => {
                    bindings.push(BindingPatternTypeArray::GetField {
                        get_field: get_field.clone(),
                    });
                }
                AssignTarget::DeclarationPattern(pattern) => {
                    bindings.push(BindingPatternTypeArray::BindingPattern {
                        pattern: pattern.clone(),
                    });
                }
                AssignTarget::GetPrivateField(_) => return None,
            },
            Node::ArrayDecl(array) => {
                let pattern = array_decl_to_declaration_pattern(array, strict)?;
                bindings.push(BindingPatternTypeArray::BindingPattern { pattern });
            }
            Node::Object(object) => {
                let pattern = object_decl_to_declaration_pattern(object, strict)?;
                bindings.push(BindingPatternTypeArray::BindingPattern { pattern });
            }
            Node::GetField(get_field) => {
                bindings.push(BindingPatternTypeArray::GetField {
                    get_field: get_field.clone(),
                });
            }
            Node::GetConstField(get_const_field) => {
                bindings.push(BindingPatternTypeArray::GetConstField {
                    get_const_field: get_const_field.clone(),
                });
            }
            _ => return None,
        }
    }
    Some(DeclarationPattern::Array(DeclarationPatternArray::new(
        bindings, None,
    )))
}
