use super::{access::PropertyAccessField, Expression};
use crate::{
    function::PrivateName,
    join_nodes, try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

/// List of valid operations in an [`Optional`] chain.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum OptionalOperationKind {
    /// A property access (`a?.prop`).
    SimplePropertyAccess {
        /// The field accessed.
        field: PropertyAccessField,
    },
    /// A private property access (`a?.#prop`).
    PrivatePropertyAccess {
        /// The private property accessed.
        field: PrivateName,
    },
    /// A function call (`a?.(arg)`).
    Call {
        /// The args passed to the function call.
        args: Box<[Expression]>,
    },
}

impl VisitWith for OptionalOperationKind {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            Self::SimplePropertyAccess { field } => visitor.visit_property_access_field(field),
            Self::PrivatePropertyAccess { field } => visitor.visit_private_name(field),
            Self::Call { args } => {
                for arg in args.iter() {
                    try_break!(visitor.visit_expression(arg));
                }
                ControlFlow::Continue(())
            }
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            Self::SimplePropertyAccess { field } => visitor.visit_property_access_field_mut(field),
            Self::PrivatePropertyAccess { field } => visitor.visit_private_name_mut(field),
            Self::Call { args } => {
                for arg in args.iter_mut() {
                    try_break!(visitor.visit_expression_mut(arg));
                }
                ControlFlow::Continue(())
            }
        }
    }
}

/// Operation within an [`Optional`] chain.
///
/// An operation within an `Optional` chain can be either shorted or non-shorted. A shorted operation
/// (`?.item`) will force the expression to return `undefined` if the target is `undefined` or `null`.
/// In contrast, a non-shorted operation (`.prop`) will try to access the property, even if the target
/// is `undefined` or `null`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct OptionalOperation {
    kind: OptionalOperationKind,
    shorted: bool,
}

impl OptionalOperation {
    /// Creates a new `OptionalOperation`.
    #[inline]
    #[must_use]
    pub const fn new(kind: OptionalOperationKind, shorted: bool) -> Self {
        Self { kind, shorted }
    }
    /// Gets the kind of operation.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> &OptionalOperationKind {
        &self.kind
    }

    /// Returns `true` if the operation short-circuits the [`Optional`] chain when the target is
    /// `undefined` or `null`.
    #[inline]
    #[must_use]
    pub const fn shorted(&self) -> bool {
        self.shorted
    }
}

impl ToInternedString for OptionalOperation {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = if self.shorted {
            String::from("?.")
        } else {
            if let OptionalOperationKind::SimplePropertyAccess {
                field: PropertyAccessField::Const(name),
            } = &self.kind
            {
                return format!(".{}", interner.resolve_expect(*name));
            }

            if let OptionalOperationKind::PrivatePropertyAccess { field } = &self.kind {
                return format!(".#{}", interner.resolve_expect(field.description()));
            }

            String::new()
        };
        buf.push_str(&match &self.kind {
            OptionalOperationKind::SimplePropertyAccess { field } => match field {
                PropertyAccessField::Const(name) => interner.resolve_expect(*name).to_string(),
                PropertyAccessField::Expr(expr) => {
                    format!("[{}]", expr.to_interned_string(interner))
                }
            },
            OptionalOperationKind::PrivatePropertyAccess { field } => {
                format!("#{}", interner.resolve_expect(field.description()))
            }
            OptionalOperationKind::Call { args } => format!("({})", join_nodes(interner, args)),
        });
        buf
    }
}

impl VisitWith for OptionalOperation {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        visitor.visit_optional_operation_kind(&self.kind)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        visitor.visit_optional_operation_kind_mut(&mut self.kind)
    }
}

/// An optional chain expression, as defined by the [spec].
///
/// [Optional chaining][mdn] allows for short-circuiting property accesses and function calls, which
/// will return `undefined` instead of returning an error if the access target or the call is
/// either `undefined` or `null`.
///
/// An example of optional chaining:
///
/// ```Javascript
/// const adventurer = {
///   name: 'Alice',
///   cat: {
///     name: 'Dinah'
///   }
/// };
///
/// console.log(adventurer.cat?.name); // Dinah
/// console.log(adventurer.dog?.name); // undefined
/// ```
///
/// [spec]: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#prod-OptionalExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Optional_chaining
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Optional {
    target: Box<Expression>,
    chain: Box<[OptionalOperation]>,
}

impl VisitWith for Optional {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_expression(&self.target));
        for op in self.chain.iter() {
            try_break!(visitor.visit_optional_operation(op));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_expression_mut(&mut self.target));
        for op in self.chain.iter_mut() {
            try_break!(visitor.visit_optional_operation_mut(op));
        }
        ControlFlow::Continue(())
    }
}

impl Optional {
    /// Creates a new `Optional` expression.
    #[inline]
    #[must_use]
    pub fn new(target: Expression, chain: Box<[OptionalOperation]>) -> Self {
        Self {
            target: Box::new(target),
            chain,
        }
    }

    /// Gets the target of this `Optional` expression.
    #[inline]
    #[must_use]
    pub fn target(&self) -> &Expression {
        self.target.as_ref()
    }

    /// Gets the chain of accesses and calls that will be applied to the target at runtime.
    #[inline]
    #[must_use]
    pub fn chain(&self) -> &[OptionalOperation] {
        self.chain.as_ref()
    }
}

impl From<Optional> for Expression {
    fn from(opt: Optional) -> Self {
        Self::Optional(opt)
    }
}

impl ToInternedString for Optional {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = self.target.to_interned_string(interner);

        for item in &*self.chain {
            buf.push_str(&item.to_interned_string(interner));
        }

        buf
    }
}
