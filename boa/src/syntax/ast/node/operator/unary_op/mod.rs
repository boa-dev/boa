use crate::{
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::{
        node::{Node, NodeKind},
        op,
    },
    Context, Result, Value,
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "vm")]
use crate::{
    profiler::BoaProfiler,
    vm::{compilation::CodeGen, Compiler, Instruction},
};

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
                let num_v_a = self.target().run(context)?.to_number(context)?;
                Value::from(if num_v_a.is_nan() {
                    -1
                } else {
                    // TODO: this is not spec compliant.
                    !(num_v_a as i32)
                })
            }
            op::UnaryOp::Void => {
                self.target().run(context)?;
                Value::undefined()
            }
            op::UnaryOp::Delete => match self.target().kind() {
                NodeKind::GetConstField(ref get_const_field) => Value::boolean(
                    get_const_field
                        .obj()
                        .run(context)?
                        .to_object(context)?
                        .delete(&get_const_field.field().into()),
                ),
                NodeKind::GetField(ref get_field) => {
                    let obj = get_field.obj().run(context)?;
                    let field = &get_field.field().run(context)?;
                    let res = obj
                        .to_object(context)?
                        .delete(&field.to_property_key(context)?);
                    return Ok(Value::boolean(res));
                }
                NodeKind::Identifier(_) => Value::boolean(false),
                NodeKind::ArrayDecl(_)
                | NodeKind::Block(_)
                | NodeKind::Const(_)
                | NodeKind::FunctionDecl(_)
                | NodeKind::FunctionExpr(_)
                | NodeKind::New(_)
                | NodeKind::Object(_)
                | NodeKind::UnaryOp(_) => Value::boolean(true),
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

impl From<UnaryOp> for NodeKind {
    fn from(op: UnaryOp) -> Self {
        Self::UnaryOp(op)
    }
}

#[cfg(feature = "vm")]
impl CodeGen for UnaryOp {
    fn compile(&self, compiler: &mut Compiler) {
        let _timer = BoaProfiler::global().start_event("UnaryOp", "codeGen");
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
