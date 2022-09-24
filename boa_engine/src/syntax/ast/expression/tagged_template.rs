use boa_interner::{Interner, Sym, ToInternedString};

use super::Expression;

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

    pub(crate) fn tag(&self) -> &Expression {
        &self.tag
    }

    pub(crate) fn raws(&self) -> &[Sym] {
        &self.raws
    }

    pub(crate) fn cookeds(&self) -> &[Option<Sym>] {
        &self.cookeds
    }

    pub(crate) fn exprs(&self) -> &[Expression] {
        &self.exprs
    }
}

impl ToInternedString for TaggedTemplate {
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
