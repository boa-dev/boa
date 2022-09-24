use crate::syntax::ast::{expression::Expression, statement::Statement};
use boa_interner::{Interner, Sym, ToInternedString};

/// The `return` statement ends function execution and specifies a value to be returned to the
/// function caller.
///
/// Syntax: `return [expression];`
///
/// `expression`:
///  > The expression whose value is to be returned. If omitted, `undefined` is returned
///  > instead.
///
/// When a `return` statement is used in a function body, the execution of the function is
/// stopped. If specified, a given value is returned to the function caller.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ReturnStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/return
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Return {
    expr: Option<Expression>,
    label: Option<Sym>,
}

impl Return {
    pub fn label(&self) -> Option<Sym> {
        self.label
    }

    pub fn expr(&self) -> Option<&Expression> {
        self.expr.as_ref()
    }

    /// Creates a `Return` AST node.
    pub fn new(expr: Option<Expression>, label: Option<Sym>) -> Self {
        Self { expr, label }
    }
}

impl From<Return> for Statement {
    fn from(return_smt: Return) -> Self {
        Self::Return(return_smt)
    }
}

impl ToInternedString for Return {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self.expr() {
            Some(ex) => format!("return {}", ex.to_interned_string(interner)),
            None => "return".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fmt() {
        crate::syntax::ast::test_formatting(
            r#"
        function say_hello(msg) {
            if (msg === "") {
                return 0;
            }
            console.log("hello " + msg);
            return;
        }
        say_hello("");
        say_hello("world");
        "#,
        );
    }
}
