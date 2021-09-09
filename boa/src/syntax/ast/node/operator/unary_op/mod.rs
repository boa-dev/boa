use crate::{
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::{node::Node, op},
    Context, JsBigInt, JsResult, JsValue,
};
use std::fmt;

use crate::builtins::Number;
use crate::value::Numeric;
#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// A unary operation is an operation with only one operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary_operators
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct UnaryOp {
    op: op::UnaryOp,
    target: Box<Node>,
}

impl UnaryOp {
    /// Creates a new `UnaryOp` AST node.
    pub(in crate::syntax) fn new<V>(op: op::UnaryOp, target: V) -> Self
    where
        V: Into<Node>,
    {
        Self {
            op,
            target: Box::new(target.into()),
        }
    }

    /// Gets the unary operation of the node.
    pub fn op(&self) -> op::UnaryOp {
        self.op
    }

    /// Gets the target of this unary operator.
    pub fn target(&self) -> &Node {
        self.target.as_ref()
    }
}

impl Executable for UnaryOp {
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match self.op() {
            op::UnaryOp::Minus => self.target().run(context)?.neg(context)?,
            op::UnaryOp::Plus => JsValue::new(self.target().run(context)?.to_number(context)?),
            op::UnaryOp::IncrementPost => {
                let x = self.target().run(context)?;
                let ret = x.clone();
                let result = x.to_number(context)? + 1.0;
                context.set_value(self.target(), result.into())?;
                ret
            }
            op::UnaryOp::IncrementPre => {
                let result = self.target().run(context)?.to_number(context)? + 1.0;
                context.set_value(self.target(), result.into())?
            }
            op::UnaryOp::DecrementPost => {
                let x = self.target().run(context)?;
                let ret = x.clone();
                let result = x.to_number(context)? - 1.0;
                context.set_value(self.target(), result.into())?;
                ret
            }
            op::UnaryOp::DecrementPre => {
                let result = self.target().run(context)?.to_number(context)? - 1.0;
                context.set_value(self.target(), result.into())?
            }
            op::UnaryOp::Not => self.target().run(context)?.not(context)?.into(),
            op::UnaryOp::Tilde => {
                let expr = self.target().run(context)?;
                let old_v = expr.to_numeric(context)?;
                match old_v {
                    Numeric::Number(x) => JsValue::new(Number::not(x)),
                    Numeric::BigInt(x) => JsValue::new(JsBigInt::not(&x)),
                }
            }
            op::UnaryOp::Void => {
                self.target().run(context)?;
                JsValue::undefined()
            }
            op::UnaryOp::Delete => match *self.target() {
                Node::GetConstField(ref get_const_field) => {
                    let delete_status = get_const_field
                        .obj()
                        .run(context)?
                        .to_object(context)?
                        .__delete__(&get_const_field.field().into(), context)?;

                    if !delete_status && context.strict() {
                        return context.throw_type_error("Cannot delete property");
                    } else {
                        JsValue::new(delete_status)
                    }
                }
                Node::GetField(ref get_field) => {
                    let obj = get_field.obj().run(context)?;
                    let field = &get_field.field().run(context)?;
                    let res = obj
                        .to_object(context)?
                        .__delete__(&field.to_property_key(context)?, context)?;
                    return Ok(JsValue::new(res));
                }
                Node::Identifier(_) => JsValue::new(false),
                Node::ArrayDecl(_)
                | Node::Block(_)
                | Node::Const(_)
                | Node::FunctionDecl(_)
                | Node::FunctionExpr(_)
                | Node::New(_)
                | Node::Object(_)
                | Node::UnaryOp(_) => JsValue::new(true),
                _ => return context.throw_syntax_error(format!("wrong delete argument {}", self)),
            },
            op::UnaryOp::TypeOf => JsValue::new(self.target().run(context)?.type_of()),
        })
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.op, self.target)
    }
}

impl From<UnaryOp> for Node {
    fn from(op: UnaryOp) -> Self {
        Self::UnaryOp(op)
    }
}
