use crate::try_break;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{
    declaration::{LexicalDeclaration, VarDeclaration},
    statement::Statement,
    Expression,
};
use boa_interner::{Interner, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;

/// The `for` statement creates a loop that consists of three optional expressions.
///
/// A [`for`][mdn] loop repeats until a specified condition evaluates to `false`.
/// The JavaScript for loop is similar to the Java and C for loop.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ForDeclaration
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ForLoop {
    #[cfg_attr(feature = "serde", serde(flatten))]
    inner: Box<InnerForLoop>,
}

impl ForLoop {
    /// Creates a new for loop AST node.
    #[inline]
    #[must_use]
    pub fn new(
        init: Option<ForLoopInitializer>,
        condition: Option<Expression>,
        final_expr: Option<Expression>,
        body: Statement,
    ) -> Self {
        Self {
            inner: Box::new(InnerForLoop::new(init, condition, final_expr, body)),
        }
    }

    /// Gets the initialization node.
    #[inline]
    #[must_use]
    pub fn init(&self) -> Option<&ForLoopInitializer> {
        self.inner.init()
    }

    /// Gets the loop condition node.
    #[inline]
    #[must_use]
    pub fn condition(&self) -> Option<&Expression> {
        self.inner.condition()
    }

    /// Gets the final expression node.
    #[inline]
    #[must_use]
    pub fn final_expr(&self) -> Option<&Expression> {
        self.inner.final_expr()
    }

    /// Gets the body of the for loop.
    #[inline]
    #[must_use]
    pub fn body(&self) -> &Statement {
        self.inner.body()
    }
}

impl ToIndentedString for ForLoop {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = String::from("for (");
        if let Some(init) = self.init() {
            buf.push_str(&init.to_interned_string(interner));
        }
        buf.push_str("; ");
        if let Some(condition) = self.condition() {
            buf.push_str(&condition.to_interned_string(interner));
        }
        buf.push_str("; ");
        if let Some(final_expr) = self.final_expr() {
            buf.push_str(&final_expr.to_interned_string(interner));
        }
        buf.push_str(&format!(
            ") {}",
            self.inner.body().to_indented_string(interner, indentation)
        ));

        buf
    }
}

impl From<ForLoop> for Statement {
    #[inline]
    fn from(for_loop: ForLoop) -> Self {
        Self::ForLoop(for_loop)
    }
}

impl VisitWith for ForLoop {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(fli) = &self.inner.init {
            try_break!(visitor.visit_for_loop_initializer(fli));
        }
        if let Some(expr) = &self.inner.condition {
            try_break!(visitor.visit_expression(expr));
        }
        if let Some(expr) = &self.inner.final_expr {
            try_break!(visitor.visit_expression(expr));
        }
        visitor.visit_statement(&self.inner.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(fli) = &mut self.inner.init {
            try_break!(visitor.visit_for_loop_initializer_mut(fli));
        }
        if let Some(expr) = &mut self.inner.condition {
            try_break!(visitor.visit_expression_mut(expr));
        }
        if let Some(expr) = &mut self.inner.final_expr {
            try_break!(visitor.visit_expression_mut(expr));
        }
        visitor.visit_statement_mut(&mut self.inner.body)
    }
}

/// Inner structure to avoid multiple indirections in the heap.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
struct InnerForLoop {
    init: Option<ForLoopInitializer>,
    condition: Option<Expression>,
    final_expr: Option<Expression>,
    body: Statement,
}

impl InnerForLoop {
    /// Creates a new inner for loop.
    #[inline]
    fn new(
        init: Option<ForLoopInitializer>,
        condition: Option<Expression>,
        final_expr: Option<Expression>,
        body: Statement,
    ) -> Self {
        Self {
            init,
            condition,
            final_expr,
            body,
        }
    }

    /// Gets the initialization node.
    #[inline]
    fn init(&self) -> Option<&ForLoopInitializer> {
        self.init.as_ref()
    }

    /// Gets the loop condition node.
    #[inline]
    fn condition(&self) -> Option<&Expression> {
        self.condition.as_ref()
    }

    /// Gets the final expression node.
    #[inline]
    fn final_expr(&self) -> Option<&Expression> {
        self.final_expr.as_ref()
    }

    /// Gets the body of the for loop.
    #[inline]
    fn body(&self) -> &Statement {
        &self.body
    }
}

/// A [`ForLoop`] initializer, as defined by the [spec].
///
/// A `ForLoop` initializer differs a lot from an
/// [`IterableLoopInitializer`][super::IterableLoopInitializer], since it can contain any arbitrary
/// expression instead of only accessors and patterns. Additionally, it can also contain many variable
/// declarations instead of only one.
///
/// [spec]: https://tc39.es/ecma262/#prod-ForStatement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum ForLoopInitializer {
    /// An expression initializer.
    Expression(Expression),
    /// A var declaration initializer.
    Var(VarDeclaration),
    /// A lexical declaration initializer.
    Lexical(LexicalDeclaration),
}

impl ToInternedString for ForLoopInitializer {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::Var(var) => var.to_interned_string(interner),
            Self::Lexical(lex) => lex.to_interned_string(interner),
            Self::Expression(expr) => expr.to_interned_string(interner),
        }
    }
}

impl From<Expression> for ForLoopInitializer {
    #[inline]
    fn from(expr: Expression) -> Self {
        ForLoopInitializer::Expression(expr)
    }
}

impl From<LexicalDeclaration> for ForLoopInitializer {
    #[inline]
    fn from(list: LexicalDeclaration) -> Self {
        ForLoopInitializer::Lexical(list)
    }
}

impl From<VarDeclaration> for ForLoopInitializer {
    #[inline]
    fn from(list: VarDeclaration) -> Self {
        ForLoopInitializer::Var(list)
    }
}

impl VisitWith for ForLoopInitializer {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            ForLoopInitializer::Expression(expr) => visitor.visit_expression(expr),
            ForLoopInitializer::Var(vd) => visitor.visit_var_declaration(vd),
            ForLoopInitializer::Lexical(ld) => visitor.visit_lexical_declaration(ld),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            ForLoopInitializer::Expression(expr) => visitor.visit_expression_mut(expr),
            ForLoopInitializer::Var(vd) => visitor.visit_var_declaration_mut(vd),
            ForLoopInitializer::Lexical(ld) => visitor.visit_lexical_declaration_mut(ld),
        }
    }
}
