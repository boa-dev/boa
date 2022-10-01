//! Template literal Expression.

use std::borrow::Cow;

use boa_interner::{Interner, Sym, ToInternedString};

use crate::{
    string::ToStringEscaped,
    syntax::ast::{expression::Expression, ContainsSymbol},
};

/// Template literals are string literals allowing embedded expressions.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals
/// [spec]: https://tc39.es/ecma262/#sec-template-literals
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateLiteral {
    elements: Box<[TemplateElement]>,
}

impl From<TemplateLiteral> for Expression {
    fn from(tem: TemplateLiteral) -> Self {
        Self::TemplateLiteral(tem)
    }
}

#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum TemplateElement {
    String(Sym),
    Expr(Expression),
}

impl TemplateLiteral {
    pub fn new(elements: Box<[TemplateElement]>) -> Self {
        Self { elements }
    }

    pub(crate) fn elements(&self) -> &[TemplateElement] {
        &self.elements
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.elements.iter().any(|e| match e {
            TemplateElement::String(_) => false,
            TemplateElement::Expr(expr) => expr.contains_arguments(),
        })
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.elements.iter().any(|e| match e {
            TemplateElement::String(_) => false,
            TemplateElement::Expr(expr) => expr.contains(symbol),
        })
    }
}

impl ToInternedString for TemplateLiteral {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = "`".to_owned();

        for elt in self.elements.iter() {
            match elt {
                TemplateElement::String(s) => buf.push_str(&interner.resolve_expect(*s).join(
                    Cow::Borrowed,
                    |utf16| Cow::Owned(utf16.to_string_escaped()),
                    true,
                )),
                TemplateElement::Expr(n) => {
                    buf.push_str(&format!("${{{}}}", n.to_interned_string(interner)));
                }
            }
        }
        buf.push('`');

        buf
    }
}

#[cfg(test)]
mod tests {
    use crate::exec;

    #[test]
    fn template_literal() {
        let scenario = r#"
        let a = 10;
        `result: ${a} and ${a+10}`;
        "#;

        assert_eq!(&exec(scenario), "\"result: 10 and 20\"");
    }

    #[test]
    fn fmt() {
        crate::syntax::ast::test_formatting(
            r#"
        function tag(t, ...args) {
            let a = [];
            a = a.concat([t[0], t[1], t[2]]);
            a = a.concat([t.raw[0], t.raw[1], t.raw[2]]);
            a = a.concat([args[0], args[1]]);
            return a;
        }
        let a = 10;
        tag`result: ${a} \x26 ${a + 10}`;
        "#,
        );
    }
}
