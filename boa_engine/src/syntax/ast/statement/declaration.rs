//! Declaration nodes
use crate::syntax::ast::{
    expression::{Expression, Identifier},
    join_nodes,
    pattern::Pattern,
    statement::Statement,
    ContainsSymbol,
};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum DeclarationList {
    /// The `const` statements are block-scoped, much like variables defined using the `let`
    /// keyword.
    ///
    /// This declaration creates a constant whose scope can be either global or local to the block
    /// in which it is declared. Global constants do not become properties of the window object,
    /// unlike var variables.
    ///
    /// An initializer for a constant is required. You must specify its value in the same statement
    /// in which it's declared. (This makes sense, given that it can't be changed later.)
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/const
    /// [identifier]: https://developer.mozilla.org/en-US/docs/Glossary/identifier
    /// [expression]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Expressions
    Const(Box<[Declaration]>),

    /// The `let` statement declares a block scope local variable, optionally initializing it to a
    /// value.
    ///
    ///
    /// `let` allows you to declare variables that are limited to a scope of a block statement, or
    /// expression on which it is used, unlike the `var` keyword, which defines a variable
    /// globally, or locally to an entire function regardless of block scope.
    ///
    /// Just like const the `let` does not create properties of the window object when declared
    /// globally (in the top-most scope).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/let
    Let(Box<[Declaration]>),

    /// The `var` statement declares a variable, optionally initializing it to a value.
    ///
    /// var declarations, wherever they occur, are processed before any code is executed. This is
    /// called hoisting, and is discussed further below.
    ///
    /// The scope of a variable declared with var is its current execution context, which is either
    /// the enclosing function or, for variables declared outside any function, global. If you
    /// re-declare a JavaScript variable, it will not lose its value.
    ///
    /// Assigning a value to an undeclared variable implicitly creates it as a global variable (it
    /// becomes a property of the global object) when the assignment is executed.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-VariableStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
    Var(Box<[Declaration]>),
}

impl DeclarationList {
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.as_ref().iter().any(|decl| decl.contains(symbol))
    }

    pub(crate) fn contains_arguments(&self) -> bool {
        self.as_ref().iter().any(Declaration::contains_arguments)
    }
}

impl AsRef<[Declaration]> for DeclarationList {
    fn as_ref(&self) -> &[Declaration] {
        use DeclarationList::{Const, Let, Var};
        match self {
            Var(list) | Const(list) | Let(list) => list,
        }
    }
}

impl ToInternedString for DeclarationList {
    fn to_interned_string(&self, interner: &Interner) -> String {
        if self.as_ref().is_empty() {
            String::new()
        } else {
            use DeclarationList::{Const, Let, Var};
            format!(
                "{} {}",
                match &self {
                    Let(_) => "let",
                    Const(_) => "const",
                    Var(_) => "var",
                },
                join_nodes(interner, self.as_ref())
            )
        }
    }
}

impl From<DeclarationList> for Statement {
    fn from(list: DeclarationList) -> Self {
        Statement::DeclarationList(list)
    }
}

impl From<Declaration> for Box<[Declaration]> {
    fn from(d: Declaration) -> Self {
        Box::new([d])
    }
}

/// Declaration represents a variable declaration of some kind.
///
/// For `let` and `const` declarations this type represents a [`LexicalBinding`][spec1]
///
/// For `var` declarations this type represents a [`VariableDeclaration`][spec2]
///
/// More information:
///  - [ECMAScript reference: 14.3 Declarations and the Variable Statement][spec3]
///
/// [spec1]: https://tc39.es/ecma262/#prod-LexicalBinding
/// [spec2]: https://tc39.es/ecma262/#prod-VariableDeclaration
/// [spec3]:  https://tc39.es/ecma262/#sec-declarations-and-the-variable-statement
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    binding: Binding,
    init: Option<Expression>,
}

impl ToInternedString for Declaration {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = self.binding.to_interned_string(interner);

        if let Some(ref init) = self.init {
            buf.push_str(&format!(" = {}", init.to_interned_string(interner)));
        }
        buf
    }
}

impl Declaration {
    /// Creates a new variable declaration from a `BindingIdentifier`.
    #[inline]
    pub(in crate::syntax) fn from_identifier(ident: Identifier, init: Option<Expression>) -> Self {
        Self {
            binding: Binding::Identifier(ident),
            init,
        }
    }

    /// Creates a new variable declaration from a `Pattern`.
    #[inline]
    pub(in crate::syntax) fn from_pattern(pattern: Pattern, init: Option<Expression>) -> Self {
        Self {
            binding: Binding::Pattern(pattern),
            init,
        }
    }
    /// Gets the variable declaration binding.
    pub(crate) fn binding(&self) -> &Binding {
        &self.binding
    }

    /// Gets the initialization expression for the variable declaration, if any.
    #[inline]
    pub(crate) fn init(&self) -> Option<&Expression> {
        self.init.as_ref()
    }

    pub(crate) fn contains_arguments(&self) -> bool {
        if let Some(ref node) = self.init {
            if node.contains_arguments() {
                return true;
            }
        }
        self.binding.contains_arguments()
    }

    /// Returns `true` if the variable declaration contains the given token.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        if let Some(ref node) = self.init {
            if node.contains(symbol) {
                return true;
            }
        }
        self.binding.contains(symbol)
    }

    /// Gets the list of declared identifiers.
    pub(crate) fn idents(&self) -> Vec<Sym> {
        self.binding.idents()
    }
}

/// Binding represents either an individual binding or a binding pattern.
///
/// More information:
///  - [ECMAScript reference: 14.3 Declarations and the Variable Statement][spec]
///
/// [spec]:  https://tc39.es/ecma262/#sec-declarations-and-the-variable-statement
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Binding {
    Identifier(Identifier),
    Pattern(Pattern),
}

impl From<Identifier> for Binding {
    fn from(id: Identifier) -> Self {
        Self::Identifier(id)
    }
}

impl From<Pattern> for Binding {
    fn from(pat: Pattern) -> Self {
        Self::Pattern(pat)
    }
}

impl Binding {
    pub(crate) fn contains_arguments(&self) -> bool {
        matches!(self, Binding::Pattern(ref pattern) if pattern.contains_arguments())
    }

    /// Returns `true` if the node contains the given token.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        matches!(self, Binding::Pattern(ref pattern) if pattern.contains(symbol))
    }

    /// Gets the list of declared identifiers.
    pub(crate) fn idents(&self) -> Vec<Sym> {
        match self {
            Binding::Identifier(id) => vec![id.sym()],
            Binding::Pattern(ref pat) => pat.idents(),
        }
    }
}

impl ToInternedString for Binding {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Binding::Identifier(id) => id.to_interned_string(interner),
            Binding::Pattern(ref pattern) => pattern.to_interned_string(interner),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fmt_binding_pattern() {
        crate::syntax::ast::test_formatting(
            r#"
        var { } = {
            o: "1",
        };
        var { o_v1 } = {
            o_v1: "1",
        };
        var { o_v2 = "1" } = {
            o_v2: "2",
        };
        var { a : o_v3 = "1" } = {
            a: "2",
        };
        var { ... o_rest_v1 } = {
            a: "2",
        };
        var { o_v4, o_v5, o_v6 = "1", a : o_v7 = "1", ... o_rest_v2 } = {
            o_v4: "1",
            o_v5: "1",
        };
        var [] = [];
        var [ , ] = [];
        var [ a_v1 ] = [1, 2, 3];
        var [ a_v2, a_v3 ] = [1, 2, 3];
        var [ a_v2, , a_v3 ] = [1, 2, 3];
        var [ ... a_rest_v1 ] = [1, 2, 3];
        var [ a_v4, , ... a_rest_v2 ] = [1, 2, 3];
        var [ { a_v5 } ] = [{
            a_v5: 1,
        }, {
            a_v5: 2,
        }, {
            a_v5: 3,
        }];
        "#,
        );
    }
}
