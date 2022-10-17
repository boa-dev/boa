use crate::syntax::ast::{
    declaration::{LexicalDeclaration, VarDeclaration, Variable},
    expression::Identifier,
    statement::Statement,
    ContainsSymbol, Expression,
};
use boa_interner::{Interner, ToInternedString};

/// The `for` statement creates a loop that consists of three optional expressions.
///
/// A `for` loop repeats until a specified condition evaluates to `false`.
/// The JavaScript for loop is similar to the Java and C for loop.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ForDeclaration
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ForLoop {
    #[cfg_attr(feature = "deser", serde(flatten))]
    inner: Box<InnerForLoop>,
}

impl ForLoop {
    /// Creates a new for loop AST node.
    pub(in crate::syntax) fn new(
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
    pub fn init(&self) -> Option<&ForLoopInitializer> {
        self.inner.init()
    }

    /// Gets the loop condition node.
    pub fn condition(&self) -> Option<&Expression> {
        self.inner.condition()
    }

    /// Gets the final expression node.
    pub fn final_expr(&self) -> Option<&Expression> {
        self.inner.final_expr()
    }

    /// Gets the body of the for loop.
    pub fn body(&self) -> &Statement {
        self.inner.body()
    }

    /// Converts the for loop to a string with the given indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
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

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        let inner = &self.inner;
        matches!(inner.init, Some(ref init) if init.contains_arguments())
            || matches!(inner.condition, Some(ref expr) if expr.contains_arguments())
            || matches!(inner.final_expr, Some(ref expr) if expr.contains_arguments())
            || inner.body.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        let inner = &self.inner;
        matches!(inner.init, Some(ref init) if init.contains(symbol))
            || matches!(inner.condition, Some(ref expr) if expr.contains(symbol))
            || matches!(inner.final_expr, Some(ref expr) if expr.contains(symbol))
            || inner.body.contains(symbol)
    }
}

impl ToInternedString for ForLoop {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<ForLoop> for Statement {
    fn from(for_loop: ForLoop) -> Self {
        Self::ForLoop(for_loop)
    }
}

/// Inner structure to avoid multiple indirections in the heap.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
struct InnerForLoop {
    init: Option<ForLoopInitializer>,
    condition: Option<Expression>,
    final_expr: Option<Expression>,
    body: Statement,
}

impl InnerForLoop {
    /// Creates a new inner for loop.
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
    fn init(&self) -> Option<&ForLoopInitializer> {
        self.init.as_ref()
    }

    /// Gets the loop condition node.
    fn condition(&self) -> Option<&Expression> {
        self.condition.as_ref()
    }

    /// Gets the final expression node.
    fn final_expr(&self) -> Option<&Expression> {
        self.final_expr.as_ref()
    }

    /// Gets the body of the for loop.
    fn body(&self) -> &Statement {
        &self.body
    }
}

#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ForLoopInitializer {
    Expression(Expression),
    Var(VarDeclaration),
    Lexical(LexicalDeclaration),
}

impl ForLoopInitializer {
    /// Return the bound names of a for loop initializer.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-boundnames
    pub(crate) fn bound_names(&self) -> Vec<Identifier> {
        match self {
            ForLoopInitializer::Lexical(lex) => lex
                .variable_list()
                .as_ref()
                .iter()
                .flat_map(Variable::idents)
                .collect(),
            _ => Vec::new(),
        }
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            Self::Var(var) => var.contains_arguments(),
            Self::Lexical(lex) => lex.contains_arguments(),
            Self::Expression(expr) => expr.contains_arguments(),
        }
    }
    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            Self::Var(var) => var.contains(symbol),
            Self::Lexical(lex) => lex.contains(symbol),
            Self::Expression(expr) => expr.contains(symbol),
        }
    }
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
    fn from(expr: Expression) -> Self {
        ForLoopInitializer::Expression(expr)
    }
}

impl From<LexicalDeclaration> for ForLoopInitializer {
    fn from(list: LexicalDeclaration) -> Self {
        ForLoopInitializer::Lexical(list)
    }
}

impl From<VarDeclaration> for ForLoopInitializer {
    fn from(list: VarDeclaration) -> Self {
        ForLoopInitializer::Var(list)
    }
}
