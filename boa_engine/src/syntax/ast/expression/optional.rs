use boa_interner::{Interner, Sym, ToInternedString};
use std::ops::ControlFlow;

use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
use crate::syntax::ast::{join_nodes, ContainsSymbol};
use crate::try_break;

use super::{access::PropertyAccessField, Expression};

/// List of valid operations in an [`Optional`] chain.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
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
        field: Sym,
    },
    /// A function call (`a?.(arg)`).
    Call {
        /// The args passed to the function call.
        args: Box<[Expression]>,
    },
}

impl OptionalOperationKind {
    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            OptionalOperationKind::SimplePropertyAccess { field } => field.contains_arguments(),
            OptionalOperationKind::PrivatePropertyAccess { .. } => false,
            OptionalOperationKind::Call { args } => args.iter().any(Expression::contains_arguments),
        }
    }
    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            OptionalOperationKind::SimplePropertyAccess { field } => field.contains(symbol),
            OptionalOperationKind::PrivatePropertyAccess { .. } => false,
            OptionalOperationKind::Call { args } => args.iter().any(|e| e.contains(symbol)),
        }
    }
}

/// Operation within an [`Optional`] chain.
///
/// An operation within an `Optional` chain can be either shorted or non-shorted. A shorted operation
/// (`?.item`) will force the expression to return `undefined` if the target is `undefined` or `null`.
/// In contrast, a non-shorted operation (`.prop`) will try to access the property, even if the target
/// is `undefined` or `null`.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct OptionalOperation {
    kind: OptionalOperationKind,
    shorted: bool,
}

impl OptionalOperation {
    /// Creates a new `OptionalOperation`.
    #[inline]
    pub fn new(kind: OptionalOperationKind, shorted: bool) -> Self {
        Self { kind, shorted }
    }
    /// Gets the kind of operation.
    #[inline]
    pub fn kind(&self) -> &OptionalOperationKind {
        &self.kind
    }

    /// Returns `true` if the operation short-circuits the [`Optional`] chain when the target is
    /// `undefined` or `null`.
    #[inline]
    pub fn shorted(&self) -> bool {
        self.shorted
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.kind.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.kind.contains(symbol)
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
                return format!(".#{}", interner.resolve_expect(*field));
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
                format!("#{}", interner.resolve_expect(*field))
            }
            OptionalOperationKind::Call { args } => format!("({})", join_nodes(interner, args)),
        });
        buf
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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
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
    pub fn new(target: Expression, chain: Box<[OptionalOperation]>) -> Self {
        Self {
            target: Box::new(target),
            chain,
        }
    }

    /// Gets the target of this `Optional` expression.
    #[inline]
    pub fn target(&self) -> &Expression {
        self.target.as_ref()
    }

    /// Gets the chain of accesses and calls that will be applied to the target at runtime.
    #[inline]
    pub fn chain(&self) -> &[OptionalOperation] {
        self.chain.as_ref()
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.target.contains_arguments()
            || self.chain.iter().any(OptionalOperation::contains_arguments)
    }
    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.target.contains(symbol) || self.chain.iter().any(|item| item.contains(symbol))
    }
}

impl From<Optional> for Expression {
    fn from(opt: Optional) -> Self {
        Expression::Optional(opt)
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
