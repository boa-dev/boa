//! Operator execution.

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
use std::borrow::BorrowMut;

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
            Node::GetConstField(ref obj, ref field) => {
                let val_obj = obj.run(interpreter)?;
                val_obj.set_field(field, val.clone());
            }
            Node::GetField(ref obj, ref field) => {
                let val_obj = obj.run(interpreter)?;
                let val_field = field.run(interpreter)?;
                val_obj.set_field(val_field, val.clone());
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
                let v_a = self.lhs().run(interpreter)?;
                let v_b = self.rhs().run(interpreter)?;
                Ok(match op {
                    NumOp::Add => v_a + v_b,
                    NumOp::Sub => v_a - v_b,
                    NumOp::Mul => v_a * v_b,
                    NumOp::Exp => v_a.as_num_to_power(v_b),
                    NumOp::Div => v_a / v_b,
                    NumOp::Mod => v_a % v_b,
                })
            }
            op::BinOp::Bit(op) => {
                let v_a = self.lhs().run(interpreter)?;
                let v_b = self.rhs().run(interpreter)?;
                Ok(match op {
                    BitOp::And => v_a & v_b,
                    BitOp::Or => v_a | v_b,
                    BitOp::Xor => v_a ^ v_b,
                    BitOp::Shl => v_a << v_b,
                    BitOp::Shr => v_a >> v_b,
                    // TODO Fix
                    BitOp::UShr => v_a >> v_b,
                })
            }
            op::BinOp::Comp(op) => {
                let mut v_a = self.lhs().run(interpreter)?;
                let mut v_b = self.rhs().run(interpreter)?;
                Ok(Value::from(match op {
                    CompOp::Equal => v_a.equals(v_b.borrow_mut(), interpreter),
                    CompOp::NotEqual => !v_a.equals(v_b.borrow_mut(), interpreter),
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
                                v_b.get_type()
                            ));
                        }
                        let key = interpreter.to_property_key(&mut v_a)?;
                        interpreter.has_property(&mut v_b, &key)
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
                        .get_binding_value(name.as_ref());
                    let v_b = self.rhs().run(interpreter)?;
                    let value = Self::run_assign(op, v_a, v_b);
                    interpreter.realm.environment.set_mutable_binding(
                        name.as_ref(),
                        value.clone(),
                        true,
                    );
                    Ok(value)
                }
                Node::GetConstField(ref obj, ref field) => {
                    let v_r_a = obj.run(interpreter)?;
                    let v_a = v_r_a.get_field(field);
                    let v_b = self.rhs().run(interpreter)?;
                    let value = Self::run_assign(op, v_a, v_b);
                    v_r_a.set_field(&field.clone(), value.clone());
                    Ok(value)
                }
                _ => Ok(Value::undefined()),
            },
        }
    }
}

impl BinOp {
    /// Runs the assignment operators.
    fn run_assign(op: AssignOp, v_a: Value, v_b: Value) -> Value {
        match op {
            AssignOp::Add => v_a + v_b,
            AssignOp::Sub => v_a - v_b,
            AssignOp::Mul => v_a * v_b,
            AssignOp::Exp => v_a.as_num_to_power(v_b),
            AssignOp::Div => v_a / v_b,
            AssignOp::Mod => v_a % v_b,
            AssignOp::And => v_a & v_b,
            AssignOp::Or => v_a | v_b,
            AssignOp::Xor => v_a ^ v_b,
            AssignOp::Shl => v_a << v_b,
            AssignOp::Shr => v_a << v_b,
        }
    }
}

impl Executable for UnaryOp {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let v_a = self.target().run(interpreter)?;

        Ok(match self.op() {
            op::UnaryOp::Minus => -v_a,
            op::UnaryOp::Plus => Value::from(v_a.to_number()),
            op::UnaryOp::IncrementPost => {
                let ret = v_a.clone();
                interpreter.set_value(self.target(), Value::from(v_a.to_number() + 1.0))?;
                ret
            }
            op::UnaryOp::IncrementPre => {
                interpreter.set_value(self.target(), Value::from(v_a.to_number() + 1.0))?
            }
            op::UnaryOp::DecrementPost => {
                let ret = v_a.clone();
                interpreter.set_value(self.target(), Value::from(v_a.to_number() - 1.0))?;
                ret
            }
            op::UnaryOp::DecrementPre => {
                interpreter.set_value(self.target(), Value::from(v_a.to_number() - 1.0))?
            }
            op::UnaryOp::Not => !v_a,
            op::UnaryOp::Tilde => {
                let num_v_a = v_a.to_number();
                // NOTE: possible UB: https://github.com/rust-lang/rust/issues/10184
                Value::from(if num_v_a.is_nan() {
                    -1
                } else {
                    !(num_v_a as i32)
                })
            }
            op::UnaryOp::Void => Value::undefined(),
            op::UnaryOp::Delete => match *self.target() {
                Node::GetConstField(ref obj, ref field) => {
                    Value::boolean(obj.run(interpreter)?.remove_property(field))
                }
                Node::GetField(ref obj, ref field) => Value::boolean(
                    obj.run(interpreter)?
                        .remove_property(&field.run(interpreter)?.to_string()),
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
            op::UnaryOp::TypeOf => Value::from(v_a.get_type()),
        })
    }
}
