//! Template literal node.

use super::Node;
use crate::{builtins::Array, exec::Executable, value::Type, BoaProfiler, Context, Result, Value};
use gc::{Finalize, Trace};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(test)]
mod tests;

/// Template literals are string literals allowing embedded expressions.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals
/// [spec]: https://tc39.es/ecma262/#sec-template-literals
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct TemplateLit {
    elements: Vec<TemplateElement>,
}

impl TemplateLit {
    pub fn new(elements: Vec<TemplateElement>) -> Self {
        TemplateLit { elements }
    }
}

impl Executable for TemplateLit {
    fn run(&self, context: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("TemplateLiteral", "exec");
        let mut result = String::new();

        for element in self.elements.iter() {
            match element {
                TemplateElement::String(s) => {
                    result.push_str(s);
                }
                TemplateElement::Expr(node) => {
                    let value = node.run(context)?;
                    let s = value.to_string(context)?;
                    result.push_str(&s);
                }
            }
        }
        Ok(result.into())
    }
}

impl fmt::Display for TemplateLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`")?;
        for elt in &self.elements {
            match elt {
                TemplateElement::String(s) => write!(f, "{}", s)?,
                TemplateElement::Expr(n) => write!(f, "${{{}}}", n)?,
            }
        }
        write!(f, "`")
    }
}
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct TaggedTemplate {
    tag: Box<Node>,
    raws: Vec<Box<str>>,
    cookeds: Vec<Box<str>>,
    exprs: Vec<Node>,
}

impl TaggedTemplate {
    pub fn new(tag: Node, raws: Vec<Box<str>>, cookeds: Vec<Box<str>>, exprs: Vec<Node>) -> Self {
        Self {
            tag: Box::new(tag),
            raws,
            cookeds,
            exprs,
        }
    }
}

impl Executable for TaggedTemplate {
    fn run(&self, context: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("TaggedTemplate", "exec");

        let template_object = Array::new_array(context);
        let raw_array = Array::new_array(context);

        for (i, raw) in self.raws.iter().enumerate() {
            raw_array.set_field(i, Value::from(raw), context)?;
        }

        for (i, cooked) in self.cookeds.iter().enumerate() {
            template_object.set_field(i, Value::from(cooked), context)?;
        }
        template_object.set_field("raw", raw_array, context)?;

        let (this, func) = match *self.tag {
            Node::GetConstField(ref get_const_field) => {
                let mut obj = get_const_field.obj().run(context)?;
                if obj.get_type() != Type::Object {
                    obj = Value::Object(obj.to_object(context)?);
                }
                (
                    obj.clone(),
                    obj.get_field(get_const_field.field(), context)?,
                )
            }
            Node::GetField(ref get_field) => {
                let obj = get_field.obj().run(context)?;
                let field = get_field.field().run(context)?;
                (
                    obj.clone(),
                    obj.get_field(field.to_property_key(context)?, context)?,
                )
            }
            _ => (
                context.global_object().clone().into(),
                self.tag.run(context)?,
            ),
        };

        let mut args = Vec::new();
        args.push(template_object);
        for expr in self.exprs.iter() {
            args.push(expr.run(context)?);
        }

        context.call(&func, &this, &args)
    }
}

impl fmt::Display for TaggedTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}`", self.tag)?;
        for (raw, expr) in self.raws.iter().zip(self.exprs.iter()) {
            write!(f, "{}${{{}}}", raw, expr)?;
        }
        write!(f, "`")
    }
}

impl From<TaggedTemplate> for Node {
    fn from(template: TaggedTemplate) -> Self {
        Node::TaggedTemplate(template)
    }
}

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum TemplateElement {
    String(Box<str>),
    Expr(Node),
}
