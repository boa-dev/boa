use crate::{
    exec::Executable,
    syntax::ast::{
        node::Node,
        op::{self, AssignOp, BitOp, CompOp, LogOp, NumOp},
    },
    vm::compilation::CodeGen,
    vm::compilation::Compiler,
    vm::instructions::Instruction,
    Context, Result, Value,
};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Binary operators requires two operands, one before the operator and one after the operator.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Operators
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    fn run_assign(op: AssignOp, x: Value, y: Value, interpreter: &mut Context) -> Result<Value> {
        match op {
            AssignOp::Add => x.add(&y, interpreter),
            AssignOp::Sub => x.sub(&y, interpreter),
            AssignOp::Mul => x.mul(&y, interpreter),
            AssignOp::Exp => x.pow(&y, interpreter),
            AssignOp::Div => x.div(&y, interpreter),
            AssignOp::Mod => x.rem(&y, interpreter),
            AssignOp::And => x.bitand(&y, interpreter),
            AssignOp::Or => x.bitor(&y, interpreter),
            AssignOp::Xor => x.bitxor(&y, interpreter),
            AssignOp::Shl => x.shl(&y, interpreter),
            AssignOp::Shr => x.shr(&y, interpreter),
            AssignOp::Ushr => x.ushr(&y, interpreter),
        }
    }
}

impl Executable for BinOp {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        match self.op() {
            op::BinOp::Num(op) => {
                let x = self.lhs().run(interpreter)?;
                let y = self.rhs().run(interpreter)?;
                match op {
                    NumOp::Add => x.add(&y, interpreter),
                    NumOp::Sub => x.sub(&y, interpreter),
                    NumOp::Mul => x.mul(&y, interpreter),
                    NumOp::Exp => x.pow(&y, interpreter),
                    NumOp::Div => x.div(&y, interpreter),
                    NumOp::Mod => x.rem(&y, interpreter),
                }
            }
            op::BinOp::Bit(op) => {
                let x = self.lhs().run(interpreter)?;
                let y = self.rhs().run(interpreter)?;
                match op {
                    BitOp::And => x.bitand(&y, interpreter),
                    BitOp::Or => x.bitor(&y, interpreter),
                    BitOp::Xor => x.bitxor(&y, interpreter),
                    BitOp::Shl => x.shl(&y, interpreter),
                    BitOp::Shr => x.shr(&y, interpreter),
                    BitOp::UShr => x.ushr(&y, interpreter),
                }
            }
            op::BinOp::Comp(op) => {
                let x = self.lhs().run(interpreter)?;
                let y = self.rhs().run(interpreter)?;
                Ok(Value::from(match op {
                    CompOp::Equal => x.equals(&y, interpreter)?,
                    CompOp::NotEqual => !x.equals(&y, interpreter)?,
                    CompOp::StrictEqual => x.strict_equals(&y),
                    CompOp::StrictNotEqual => !x.strict_equals(&y),
                    CompOp::GreaterThan => x.gt(&y, interpreter)?,
                    CompOp::GreaterThanOrEqual => x.ge(&y, interpreter)?,
                    CompOp::LessThan => x.lt(&y, interpreter)?,
                    CompOp::LessThanOrEqual => x.le(&y, interpreter)?,
                    CompOp::In => {
                        if !y.is_object() {
                            return interpreter.throw_type_error(format!(
                                "right-hand side of 'in' should be an object, got {}",
                                y.get_type().as_str()
                            ));
                        }
                        let key = x.to_property_key(interpreter)?;
                        interpreter.has_property(&y, &key)
                    }
                    CompOp::InstanceOf => {
                        if !y.is_object() {
                            return interpreter.throw_type_error(format!(
                                "right-hand side of 'instanceof' should be an object, got {}",
                                y.get_type().as_str()
                            ));
                        }

                        // spec: https://tc39.es/ecma262/#sec-instanceofoperator
                        todo!("instanceof operator")
                    }
                }))
            }
            op::BinOp::Log(op) => {
                // turn a `Value` into a `bool`
                let to_bool = |value| bool::from(&value);
                Ok(match op {
                    LogOp::And => Value::from(
                        to_bool(self.lhs().run(interpreter)?)
                            && to_bool(self.rhs().run(interpreter)?),
                    ),
                    LogOp::Or => Value::from(
                        to_bool(self.lhs().run(interpreter)?)
                            || to_bool(self.rhs().run(interpreter)?),
                    ),
                })
            }
            op::BinOp::Assign(op) => match self.lhs() {
                Node::Identifier(ref name) => {
                    let v_a = interpreter
                        .realm()
                        .environment
                        .get_binding_value(name.as_ref())
                        .ok_or_else(|| interpreter.construct_reference_error(name.as_ref()))?;
                    let v_b = self.rhs().run(interpreter)?;
                    let value = Self::run_assign(op, v_a, v_b, interpreter)?;
                    interpreter.realm_mut().environment.set_mutable_binding(
                        name.as_ref(),
                        value.clone(),
                        true,
                    );
                    Ok(value)
                }
                Node::GetConstField(ref get_const_field) => {
                    let v_r_a = get_const_field.obj().run(interpreter)?;
                    let v_a = v_r_a.get_field(get_const_field.field());
                    let v_b = self.rhs().run(interpreter)?;
                    let value = Self::run_assign(op, v_a, v_b, interpreter)?;
                    v_r_a.set_field(get_const_field.field(), value.clone());
                    Ok(value)
                }
                _ => Ok(Value::undefined()),
            },
            op::BinOp::Comma => {
                self.lhs().run(interpreter)?;
                Ok(self.rhs().run(interpreter)?)
            }
        }
    }
}

impl CodeGen for BinOp {
    fn compile(&self, compiler: &mut Compiler) {
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
