use crate::{
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::{
        node::Node,
        op::{self, AssignOp, BitOp, CompOp, LogOp, NumOp},
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

/// Binary operators requires two operands, one before the operator and one after the operator.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Operators
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct BinOp {
    op: op::BinOp,
    lhs: Box<Node>,
    rhs: Box<Node>,
}

impl BinOp {
    /// Creates a `BinOp` AST node.
    pub(in crate::syntax) fn new<O, L, R>(op: O, lhs: L, rhs: R) -> Self
    where
        O: Into<op::BinOp>,
        L: Into<Node>,
        R: Into<Node>,
    {
        Self {
            op: op.into(),
            lhs: Box::new(lhs.into()),
            rhs: Box::new(rhs.into()),
        }
    }

    /// Gets the binary operation of the node.
    pub fn op(&self) -> op::BinOp {
        self.op
    }

    /// Gets the left hand side of the binary operation.
    pub fn lhs(&self) -> &Node {
        &self.lhs
    }

    /// Gets the right hand side of the binary operation.
    pub fn rhs(&self) -> &Node {
        &self.rhs
    }

    /// Runs the assignment operators.
    fn run_assign(op: AssignOp, x: Value, y: &Node, context: &mut Context) -> Result<Value> {
        match op {
            AssignOp::Add => x.add(&y.run(context)?, context),
            AssignOp::Sub => x.sub(&y.run(context)?, context),
            AssignOp::Mul => x.mul(&y.run(context)?, context),
            AssignOp::Exp => x.pow(&y.run(context)?, context),
            AssignOp::Div => x.div(&y.run(context)?, context),
            AssignOp::Mod => x.rem(&y.run(context)?, context),
            AssignOp::And => x.bitand(&y.run(context)?, context),
            AssignOp::Or => x.bitor(&y.run(context)?, context),
            AssignOp::Xor => x.bitxor(&y.run(context)?, context),
            AssignOp::Shl => x.shl(&y.run(context)?, context),
            AssignOp::Shr => x.shr(&y.run(context)?, context),
            AssignOp::Ushr => x.ushr(&y.run(context)?, context),
            AssignOp::BoolAnd => {
                if x.to_boolean() {
                    Ok(y.run(context)?)
                } else {
                    Ok(x)
                }
            }
            AssignOp::BoolOr => {
                if x.to_boolean() {
                    Ok(x)
                } else {
                    Ok(y.run(context)?)
                }
            }
            AssignOp::Coalesce => {
                if x.is_null_or_undefined() {
                    Ok(y.run(context)?)
                } else {
                    Ok(x)
                }
            }
        }
    }
}

impl Executable for BinOp {
    fn run(&self, context: &mut Context) -> Result<Value> {
        match self.op() {
            op::BinOp::Num(op) => {
                let x = self.lhs().run(context)?;
                let y = self.rhs().run(context)?;
                match op {
                    NumOp::Add => x.add(&y, context),
                    NumOp::Sub => x.sub(&y, context),
                    NumOp::Mul => x.mul(&y, context),
                    NumOp::Exp => x.pow(&y, context),
                    NumOp::Div => x.div(&y, context),
                    NumOp::Mod => x.rem(&y, context),
                }
            }
            op::BinOp::Bit(op) => {
                let x = self.lhs().run(context)?;
                let y = self.rhs().run(context)?;
                match op {
                    BitOp::And => x.bitand(&y, context),
                    BitOp::Or => x.bitor(&y, context),
                    BitOp::Xor => x.bitxor(&y, context),
                    BitOp::Shl => x.shl(&y, context),
                    BitOp::Shr => x.shr(&y, context),
                    BitOp::UShr => x.ushr(&y, context),
                }
            }
            op::BinOp::Comp(op) => {
                let x = self.lhs().run(context)?;
                let y = self.rhs().run(context)?;
                Ok(Value::from(match op {
                    CompOp::Equal => x.equals(&y, context)?,
                    CompOp::NotEqual => !x.equals(&y, context)?,
                    CompOp::StrictEqual => x.strict_equals(&y),
                    CompOp::StrictNotEqual => !x.strict_equals(&y),
                    CompOp::GreaterThan => x.gt(&y, context)?,
                    CompOp::GreaterThanOrEqual => x.ge(&y, context)?,
                    CompOp::LessThan => x.lt(&y, context)?,
                    CompOp::LessThanOrEqual => x.le(&y, context)?,
                    CompOp::In => {
                        if !y.is_object() {
                            return context.throw_type_error(format!(
                                "right-hand side of 'in' should be an object, got {}",
                                y.get_type().as_str()
                            ));
                        }
                        let key = x.to_property_key(context)?;
                        context.has_property(&y, &key)
                    }
                    CompOp::InstanceOf => {
                        if let Some(object) = y.as_object() {
                            let key = context.well_known_symbols().has_instance_symbol();

                            match object.get_method(context, key)? {
                                Some(instance_of_handler) => {
                                    instance_of_handler.call(&y, &[x], context)?.to_boolean()
                                }
                                None if object.is_callable() => {
                                    object.ordinary_has_instance(context, &x)?
                                }
                                None => {
                                    return context.throw_type_error(
                                        "right-hand side of 'instanceof' is not callable",
                                    );
                                }
                            }
                        } else {
                            return context.throw_type_error(format!(
                                "right-hand side of 'instanceof' should be an object, got {}",
                                y.get_type().as_str()
                            ));
                        }
                    }
                }))
            }
            op::BinOp::Log(op) => Ok(match op {
                LogOp::And => {
                    let left = self.lhs().run(context)?;
                    if !left.to_boolean() {
                        left
                    } else {
                        self.rhs().run(context)?
                    }
                }
                LogOp::Or => {
                    let left = self.lhs().run(context)?;
                    if left.to_boolean() {
                        left
                    } else {
                        self.rhs().run(context)?
                    }
                }
                LogOp::Coalesce => {
                    let left = self.lhs.run(context)?;
                    if left.is_null_or_undefined() {
                        self.rhs().run(context)?
                    } else {
                        left
                    }
                }
            }),
            op::BinOp::Assign(op) => match self.lhs() {
                Node::Identifier(ref name) => {
                    let v_a = context
                        .get_binding_value(name.as_ref())
                        .map_err(|e| e.to_error(context))?;

                    let value = Self::run_assign(op, v_a, self.rhs(), context)?;
                    context
                        .set_mutable_binding(name.as_ref(), value.clone(), true)
                        .map_err(|e| e.to_error(context))?;
                    Ok(value)
                }
                Node::GetConstField(ref get_const_field) => {
                    let v_r_a = get_const_field.obj().run(context)?;
                    let v_a = v_r_a.get_field(get_const_field.field(), context)?;
                    let value = Self::run_assign(op, v_a, self.rhs(), context)?;
                    v_r_a.set_field(get_const_field.field(), value.clone(), context)?;
                    Ok(value)
                }
                _ => Ok(Value::undefined()),
            },
            op::BinOp::Comma => {
                self.lhs().run(context)?;
                Ok(self.rhs().run(context)?)
            }
        }
    }
}

#[cfg(feature = "vm")]
impl CodeGen for BinOp {
    fn compile(&self, compiler: &mut Compiler) {
        let _timer = BoaProfiler::global().start_event("binOp", "codeGen");
        match self.op() {
            op::BinOp::Num(op) => {
                self.lhs().compile(compiler);
                self.rhs().compile(compiler);
                match op {
                    NumOp::Add => compiler.add_instruction(Instruction::Add),
                    NumOp::Sub => compiler.add_instruction(Instruction::Sub),
                    NumOp::Mul => compiler.add_instruction(Instruction::Mul),
                    NumOp::Div => compiler.add_instruction(Instruction::Div),
                    NumOp::Exp => compiler.add_instruction(Instruction::Pow),
                    NumOp::Mod => compiler.add_instruction(Instruction::Mod),
                }
            }
            op::BinOp::Bit(op) => {
                self.lhs().compile(compiler);
                self.rhs().compile(compiler);
                match op {
                    BitOp::And => compiler.add_instruction(Instruction::BitAnd),
                    BitOp::Or => compiler.add_instruction(Instruction::BitOr),
                    BitOp::Xor => compiler.add_instruction(Instruction::BitXor),
                    BitOp::Shl => compiler.add_instruction(Instruction::Shl),
                    BitOp::Shr => compiler.add_instruction(Instruction::Shr),
                    BitOp::UShr => compiler.add_instruction(Instruction::UShr),
                }
            }
            op::BinOp::Comp(op) => {
                self.lhs().compile(compiler);
                self.rhs().compile(compiler);
                match op {
                    CompOp::Equal => compiler.add_instruction(Instruction::Eq),
                    CompOp::NotEqual => compiler.add_instruction(Instruction::NotEq),
                    CompOp::StrictEqual => compiler.add_instruction(Instruction::StrictEq),
                    CompOp::StrictNotEqual => compiler.add_instruction(Instruction::StrictNotEq),
                    CompOp::GreaterThan => compiler.add_instruction(Instruction::Gt),
                    CompOp::GreaterThanOrEqual => compiler.add_instruction(Instruction::Ge),
                    CompOp::LessThan => compiler.add_instruction(Instruction::Lt),
                    CompOp::LessThanOrEqual => compiler.add_instruction(Instruction::Le),
                    CompOp::In => compiler.add_instruction(Instruction::In),
                    CompOp::InstanceOf => compiler.add_instruction(Instruction::InstanceOf),
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.lhs, self.op, self.rhs)
    }
}

impl From<BinOp> for Node {
    fn from(op: BinOp) -> Self {
        Self::BinOp(op)
    }
}
