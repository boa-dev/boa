//! Operator execution.
#[cfg(test)]
mod tests;

use super::{Executable, Interpreter};
use crate::{
    builtins::value::{ResultValue, Value},
    environment::lexical_environment::VariableScope,
    syntax::ast::{
        node::{Assign, BinOp, Node, UnaryOp},
        op::{self, AssignOp, BitOp, CompOp, LogOp, NumOp},
    },
    BoaProfiler,
};

impl Executable for Assign {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let _timer = BoaProfiler::global().start_event("Assign", "exec");
        let val = self.rhs().run(interpreter)?;
        match self.lhs() {
            Node::Identifier(ref name) => {
                let environment = &mut interpreter.realm_mut().environment;

                if environment.has_binding(name.as_ref()) {
                    // Binding already exists
                    environment.set_mutable_binding(name.as_ref(), val.clone(), true);
                } else {
                    environment.create_mutable_binding(
                        name.as_ref().to_owned(),
                        true,
                        VariableScope::Function,
                    );
                    environment.initialize_binding(name.as_ref(), val.clone());
                }
            }
            Node::GetConstField(ref get_const_field) => {
                let val_obj = get_const_field.obj().run(interpreter)?;
                val_obj.set_str_field(get_const_field.field(), val.clone());
            }
            Node::GetField(ref get_field) => {
                let val_obj = get_field.obj().run(interpreter)?;
                let val_field = get_field.field().run(interpreter)?;
                val_obj.set_field(val_field, val.clone(), interpreter);
            }
            _ => (),
        }
        Ok(val)
    }
}

impl Executable for BinOp {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
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
                let v_a = self.lhs().run(interpreter)?;
                let v_b = self.rhs().run(interpreter)?;
                Ok(Value::from(match op {
                    CompOp::Equal => v_a.equals(&v_b, interpreter)?,
                    CompOp::NotEqual => !v_a.equals(&v_b, interpreter)?,
                    CompOp::StrictEqual => v_a.strict_equals(&v_b),
                    CompOp::StrictNotEqual => !v_a.strict_equals(&v_b),
                    CompOp::GreaterThan => v_a.to_number() > v_b.to_number(),
                    CompOp::GreaterThanOrEqual => v_a.to_number() >= v_b.to_number(),
                    CompOp::LessThan => v_a.to_number() < v_b.to_number(),
                    CompOp::LessThanOrEqual => v_a.to_number() <= v_b.to_number(),
                    CompOp::In => {
                        if !v_b.is_object() {
                            return interpreter.throw_type_error(format!(
                                "right-hand side of 'in' should be an object, got {}",
                                v_b.get_type().as_str()
                            ));
                        }
                        let key = interpreter.to_property_key(&v_a)?;
                        interpreter.has_property(&v_b, &key)
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
                    interpreter.realm.environment.set_mutable_binding(
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
                    v_r_a.set_str_field(get_const_field.field(), value.clone());
                    Ok(value)
                }
                _ => Ok(Value::undefined()),
            },
        }
    }
}

impl BinOp {
    /// Runs the assignment operators.
    fn run_assign(op: AssignOp, x: Value, y: Value, interpreter: &mut Interpreter) -> ResultValue {
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
        }
    }
}

impl Executable for UnaryOp {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let x = self.target().run(interpreter)?;

        Ok(match self.op() {
            op::UnaryOp::Minus => x.neg(interpreter)?,
            op::UnaryOp::Plus => Value::from(x.to_number()),
            op::UnaryOp::IncrementPost => {
                let ret = x.clone();
                interpreter.set_value(self.target(), Value::from(x.to_number() + 1.0))?;
                ret
            }
            op::UnaryOp::IncrementPre => {
                interpreter.set_value(self.target(), Value::from(x.to_number() + 1.0))?
            }
            op::UnaryOp::DecrementPost => {
                let ret = x.clone();
                interpreter.set_value(self.target(), Value::from(x.to_number() - 1.0))?;
                ret
            }
            op::UnaryOp::DecrementPre => {
                interpreter.set_value(self.target(), Value::from(x.to_number() - 1.0))?
            }
            op::UnaryOp::Not => x.not(interpreter)?,
            op::UnaryOp::Tilde => {
                let num_v_a = x.to_number();
                // NOTE: possible UB: https://github.com/rust-lang/rust/issues/10184
                Value::from(if num_v_a.is_nan() {
                    -1
                } else {
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
                Node::GetField(ref get_field) => Value::boolean(
                    get_field
                        .obj()
                        .run(interpreter)?
                        .remove_property(&get_field.field().run(interpreter)?.to_string()),
                ),
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
