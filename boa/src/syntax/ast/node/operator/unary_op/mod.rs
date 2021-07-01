use crate::{exec::Executable, gc::{Finalize, Trace}, syntax::ast::{node::Node, op}, Context, Result, Value, JsBigInt};
use std::fmt;

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
    fn run(&self, context: &mut Context) -> Result<Value> {
        Ok(match self.op() {
            op::UnaryOp::Minus => self.target().run(context)?.neg(context)?,
            op::UnaryOp::Plus => Value::from(self.target().run(context)?.to_number(context)?),
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
                let num_v_a = self.target().run(context)?.to_numeric_number(context)?;
                if num_v_a.is_nan() || num_v_a.is_infinite() {
                    Value::from(-1) // special case for inf or nan
                } else if let Some(num_bigint) = self.target.run(context)?.as_bigint() {

                    Value::from(JsBigInt::not(num_bigint))
                // add bigint support
                } else {
                    let masked = (num_v_a as i64) & 0x00000000ffffffff; // converts float to i32 following spec to ignore MSB using mask
                    Value::from(!(masked as i32)) // Nots i32 conversion and creates value from this
                }
            }
            op::UnaryOp::Void => {
                self.target().run(context)?;
                Value::undefined()
            }
            op::UnaryOp::Delete => match *self.target() {
                Node::GetConstField(ref get_const_field) => Value::boolean(
                    get_const_field
                        .obj()
                        .run(context)?
                        .to_object(context)?
                        .__delete__(&get_const_field.field().into()),
                ),
                Node::GetField(ref get_field) => {
                    let obj = get_field.obj().run(context)?;
                    let field = &get_field.field().run(context)?;
                    let res = obj
                        .to_object(context)?
                        .__delete__(&field.to_property_key(context)?);
                    return Ok(Value::boolean(res));
                }
                Node::Identifier(_) => Value::boolean(false),
                Node::ArrayDecl(_)
                | Node::Block(_)
                | Node::Const(_)
                | Node::FunctionDecl(_)
                | Node::FunctionExpr(_)
                | Node::New(_)
                | Node::Object(_)
                | Node::UnaryOp(_) => Value::boolean(true),
                _ => return context.throw_syntax_error(format!("wrong delete argument {}", self)),
            },
            op::UnaryOp::TypeOf => Value::from(self.target().run(context)?.get_type().as_str()),
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
