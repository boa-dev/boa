use boa_interner::{Interner, Sym, ToInternedString};

use crate::syntax::ast::ContainsSymbol;

use super::Expression;

/// A [`TaggedTemplate`][moz] expression, as defined by the [spec].
///
/// `TaggedTemplate`s are a type of template literals that are parsed by a custom function to generate
/// arbitrary objects from the inner strings and expressions.
///
/// [moz]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals#tagged_templates
/// [spec]: https://tc39.es/ecma262/#sec-tagged-templates
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct TaggedTemplate {
    tag: Box<Expression>,
    raws: Box<[Sym]>,
    cookeds: Box<[Option<Sym>]>,
    exprs: Box<[Expression]>,
}

impl TaggedTemplate {
    /// Creates a new tagged template with a tag, the list of raw strings, the cooked strings and
    /// the expressions.
    #[inline]
    pub fn new(
        tag: Expression,
        raws: Box<[Sym]>,
        cookeds: Box<[Option<Sym>]>,
        exprs: Box<[Expression]>,
    ) -> Self {
        Self {
            tag: tag.into(),
            raws,
            cookeds,
            exprs,
        }
    }

    #[inline]
    pub(crate) fn tag(&self) -> &Expression {
        &self.tag
    }

    #[inline]
    pub(crate) fn raws(&self) -> &[Sym] {
        &self.raws
    }

    #[inline]
    pub(crate) fn cookeds(&self) -> &[Option<Sym>] {
        &self.cookeds
    }

    #[inline]
    pub(crate) fn exprs(&self) -> &[Expression] {
        &self.exprs
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.tag.contains_arguments() || self.exprs.iter().any(Expression::contains_arguments)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.tag.contains(symbol) || self.exprs.iter().any(|expr| expr.contains(symbol))
    }
}

impl ToInternedString for TaggedTemplate {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = format!("{}`", self.tag.to_interned_string(interner));
        for (&raw, expr) in self.raws.iter().zip(self.exprs.iter()) {
            buf.push_str(&format!(
                "{}${{{}}}",
                interner.resolve_expect(raw),
                expr.to_interned_string(interner)
            ));
        }
        buf.push('`');

        buf
    }
}

impl From<TaggedTemplate> for Expression {
    #[inline]
    fn from(template: TaggedTemplate) -> Self {
        Self::TaggedTemplate(template)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tagged_template() {
        let scenario = r#"
        function tag(t, ...args) {
           let a = []
           a = a.concat([t[0], t[1], t[2]]);
           a = a.concat([t.raw[0], t.raw[1], t.raw[2]]);
           a = a.concat([args[0], args[1]]);
           return a
        }
        let a = 10;
        tag`result: ${a} \x26 ${a+10}`;
        "#;

        assert_eq!(
            &crate::exec(scenario),
            r#"[ "result: ", " & ", "", "result: ", " \x26 ", "", 10, 20 ]"#
        );
    }
}
