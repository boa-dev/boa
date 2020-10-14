use crate::{
    exec::Executable,
    syntax::ast::{node::Node, op},
    vm::compilation::CodeGen,
    vm::Compiler,
    vm::Instruction,
    Context, Result, Value,
};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A unary operation is an operation with only one operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary_operators
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        let x = self.target().run(interpreter)?;

        Ok(match self.op() {
            op::UnaryOp::Minus => x.neg(interpreter)?,
            op::UnaryOp::Plus => Value::from(x.to_number(interpreter)?),
            op::UnaryOp::IncrementPost => {
                let ret = x.clone();
                let result = x.to_number(interpreter)? + 1.0;
                interpreter.set_value(self.target(), result.into())?;
                ret
            }
            op::UnaryOp::IncrementPre => {
                let result = x.to_number(interpreter)? + 1.0;
                interpreter.set_value(self.target(), result.into())?
            }
            op::UnaryOp::DecrementPost => {
                let ret = x.clone();
                let result = x.to_number(interpreter)? - 1.0;
                interpreter.set_value(self.target(), result.into())?;
                ret
            }
            op::UnaryOp::DecrementPre => {
                let result = x.to_number(interpreter)? - 1.0;
                interpreter.set_value(self.target(), result.into())?
            }
            op::UnaryOp::Not => x.not(interpreter)?.into(),
            op::UnaryOp::Tilde => {
                let num_v_a = x.to_number(interpreter)?;
                Value::from(if num_v_a.is_nan() {
                    -1
                } else {
                    // TODO: this is not spec compliant.
                    !(num_v_a as i32)
                })
            }
            op::UnaryOp::Void => Value::undefined(),
            op::UnaryOp::Delete => match *self.target() {
                Node::GetConstField(ref get_const_field) => Value::boolean(
                    get_const_field
                        .obj()
                        .run(interpreter)?
                        .remove_property(get_const_field.field()),
                ),
                Node::GetField(ref get_field) => {
                    let obj = get_field.obj().run(interpreter)?;
                    let field = &get_field.field().run(interpreter)?;
                    let res = obj.remove_property(field.to_string(interpreter)?.as_str());
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
                _ => panic!("SyntaxError: wrong delete argument {}", self),
            },
            op::UnaryOp::TypeOf => Value::from(x.get_type().as_str()),
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

impl CodeGen for UnaryOp {
    fn compile(&self, compiler: &mut Compiler) {
        self.target().compile(compiler);
        match self.op {
            op::UnaryOp::Void => compiler.add_instruction(Instruction::Void),
            op::UnaryOp::Plus => compiler.add_instruction(Instruction::Pos),
            op::UnaryOp::Minus => compiler.add_instruction(Instruction::Neg),
            op::UnaryOp::TypeOf => compiler.add_instruction(Instruction::TypeOf),
            op::UnaryOp::Not => compiler.add_instruction(Instruction::Not),
            op::UnaryOp::Tilde => compiler.add_instruction(Instruction::BitNot),
            op::UnaryOp::IncrementPost => {}
            op::UnaryOp::IncrementPre => {}
            op::UnaryOp::DecrementPost => {}
            op::UnaryOp::DecrementPre => {}
            op::UnaryOp::Delete => {}
        }
    }
}
