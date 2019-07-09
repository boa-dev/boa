use crate::environment::lexical_environment::{new_function_environment, LexicalEnvironment};
use crate::js::function::{Function, RegularFunction};
use crate::js::object::{INSTANCE_PROTOTYPE, PROTOTYPE};
use crate::js::value::{from_value, to_value, ResultValue, ValueData};
use crate::js::{array, console, function, json, math, object, string};
use crate::syntax::ast::constant::Const;
use crate::syntax::ast::expr::{Expr, ExprDef};
use crate::syntax::ast::op::{BinOp, BitOp, CompOp, LogOp, NumOp, UnaryOp};
use gc::{Gc, GcCell};
use std::borrow::Borrow;

/// An execution engine
pub trait Executor {
    /// Make a new execution engine
    fn new() -> Self;
    /// Run an expression
    fn run(&mut self, expr: &Expr) -> ResultValue;
}

/// A Javascript intepreter
#[derive(Debug)]
pub struct Interpreter {
    /// An object representing the global object
    environment: LexicalEnvironment,
}

impl Executor for Interpreter {
    fn new() -> Self {
        let global = ValueData::new_obj(None);
        object::init(&global);
        console::init(&global);
        math::init(&global);
        array::init(&global);
        function::init(&global);
        json::init(&global);
        string::init(&global);
        Self {
            environment: LexicalEnvironment::new(global.clone()),
        }
    }

    #[allow(clippy::match_same_arms)]
    fn run(&mut self, expr: &Expr) -> ResultValue {
        match expr.def {
            ExprDef::ConstExpr(Const::Null) => Ok(to_value(None::<()>)),
            ExprDef::ConstExpr(Const::Undefined) => Ok(Gc::new(ValueData::Undefined)),
            ExprDef::ConstExpr(Const::Num(num)) => Ok(to_value(num)),
            ExprDef::ConstExpr(Const::Int(num)) => Ok(to_value(num)),
            // we can't move String from Const into value, because const is a garbage collected value
            // Which means Drop() get's called on Const, but str will be gone at that point.
            // Do Const values need to be garbage collected? We no longer need them once we've generated Values
            ExprDef::ConstExpr(Const::String(ref str)) => Ok(to_value(str.to_owned())),
            ExprDef::ConstExpr(Const::Bool(val)) => Ok(to_value(val)),
            ExprDef::ConstExpr(Const::RegExp(_, _, _)) => Ok(to_value(None::<()>)),
            ExprDef::BlockExpr(ref es) => {
                let mut obj = to_value(None::<()>);
                for e in es.iter() {
                    let val = self.run(e)?;
                    if e == es.last().unwrap() {
                        obj = val;
                    }
                }
                Ok(obj)
            }
            ExprDef::LocalExpr(ref name) => {
                let val = self.environment.get_binding_value(name);
                Ok(val)
            }
            ExprDef::GetConstFieldExpr(ref obj, ref field) => {
                let val_obj = self.run(obj)?;
                Ok(val_obj.borrow().get_field(field))
            }
            ExprDef::GetFieldExpr(ref obj, ref field) => {
                let val_obj = self.run(obj)?;
                let val_field = self.run(field)?;
                Ok(val_obj.borrow().get_field(&val_field.borrow().to_string()))
            }
            ExprDef::CallExpr(ref callee, ref args) => {
                let (this, func) = match callee.def {
                    ExprDef::GetConstFieldExpr(ref obj, ref field) => {
                        let obj = self.run(obj)?;
                        (obj.clone(), obj.borrow().get_field(field))
                    }
                    ExprDef::GetFieldExpr(ref obj, ref field) => {
                        let obj = self.run(obj)?;
                        let field = self.run(field)?;
                        (
                            obj.clone(),
                            obj.borrow().get_field(&field.borrow().to_string()),
                        )
                    }
                    _ => (
                        self.environment.get_global_object().unwrap(),
                        self.run(&callee.clone())?,
                    ), // 'this' binding should come from the function's self-contained environment
                };
                let mut v_args = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    v_args.push(self.run(arg)?);
                }
                match *func {
                    ValueData::Function(ref inner_func) => match *inner_func.as_ref().borrow() {
                        Function::NativeFunc(ref ntv) => {
                            let func = ntv.data;
                            func(&this, v_args, &self)
                        }
                        Function::RegularFunc(ref data) => {
                            let env = &mut self.environment;
                            // New target (second argument) is only needed for constructors, just pass undefined
                            let undefined = Gc::new(ValueData::Undefined);
                            env.push(new_function_environment(
                                func.clone(),
                                undefined,
                                Some(env.get_current_environment_ref().clone()),
                            ));
                            for i in 0..data.args.len() {
                                let name = data.args.get(i).unwrap();
                                let expr = v_args.get(i).unwrap();
                                self.environment.create_mutable_binding(name.clone(), false);
                                self.environment.initialize_binding(name, expr.to_owned());
                            }
                            let result = self.run(&data.expr);
                            self.environment.pop();
                            result
                        }
                    },
                    _ => Err(Gc::new(ValueData::Undefined)),
                }
            }
            ExprDef::WhileLoopExpr(ref cond, ref expr) => {
                let mut result = Gc::new(ValueData::Undefined);
                while self.run(cond)?.borrow().is_true() {
                    result = self.run(expr)?;
                }
                Ok(result)
            }
            ExprDef::IfExpr(ref cond, ref expr, None) => {
                Ok(if self.run(cond)?.borrow().is_true() {
                    self.run(expr)?
                } else {
                    Gc::new(ValueData::Undefined)
                })
            }
            ExprDef::IfExpr(ref cond, ref expr, Some(ref else_e)) => {
                Ok(if self.run(cond)?.borrow().is_true() {
                    self.run(expr)?
                } else {
                    self.run(else_e)?
                })
            }
            ExprDef::SwitchExpr(ref val_e, ref vals, ref default) => {
                let val = self.run(val_e)?.clone();
                let mut result = Gc::new(ValueData::Null);
                let mut matched = false;
                for tup in vals.iter() {
                    let tup: &(Expr, Vec<Expr>) = tup;
                    let cond = &tup.0;
                    let block = &tup.1;
                    if val == self.run(cond)? {
                        matched = true;
                        let last_expr = block.last().unwrap();
                        for expr in block.iter() {
                            let e_result = self.run(expr)?;
                            if expr == last_expr {
                                result = e_result;
                            }
                        }
                    }
                }
                if !matched && default.is_some() {
                    result = self.run(default.as_ref().unwrap())?;
                }
                Ok(result)
            }
            ExprDef::ObjectDeclExpr(ref map) => {
                let global_val = &self.environment.get_global_object().unwrap();
                let obj = ValueData::new_obj(Some(global_val));
                for (key, val) in map.iter() {
                    obj.borrow().set_field(key.clone(), self.run(val)?);
                }
                Ok(obj)
            }
            ExprDef::ArrayDeclExpr(ref arr) => {
                let global_val = &self.environment.get_global_object().unwrap();
                let arr_map = ValueData::new_obj(Some(global_val));
                let mut index: i32 = 0;
                for val in arr.iter() {
                    let val = self.run(val)?;
                    arr_map.borrow().set_field(index.to_string(), val);
                    index += 1;
                }
                arr_map.borrow().set_field_slice(
                    INSTANCE_PROTOTYPE,
                    self.environment
                        .get_binding_value("Array")
                        .borrow()
                        .get_field_slice(PROTOTYPE),
                );
                arr_map.borrow().set_field_slice("length", to_value(index));
                Ok(arr_map)
            }
            ExprDef::FunctionDeclExpr(ref name, ref args, ref expr) => {
                let function =
                    Function::RegularFunc(RegularFunction::new(*expr.clone(), args.clone()));
                let val = Gc::new(ValueData::Function(Box::new(GcCell::new(function))));
                if name.is_some() {
                    self.environment
                        .create_mutable_binding(name.clone().unwrap(), false);
                    self.environment
                        .initialize_binding(name.as_ref().unwrap(), val.clone())
                }
                Ok(val)
            }
            ExprDef::ArrowFunctionDeclExpr(ref args, ref expr) => {
                let function =
                    Function::RegularFunc(RegularFunction::new(*expr.clone(), args.clone()));
                Ok(Gc::new(ValueData::Function(Box::new(GcCell::new(
                    function,
                )))))
            }
            ExprDef::BinOpExpr(BinOp::Num(ref op), ref a, ref b) => {
                let v_r_a = self.run(a)?;
                let v_r_b = self.run(b)?;
                let v_a = (*v_r_a).clone();
                let v_b = (*v_r_b).clone();
                Ok(Gc::new(match *op {
                    NumOp::Add => v_a + v_b,
                    NumOp::Sub => v_a - v_b,
                    NumOp::Mul => v_a * v_b,
                    NumOp::Div => v_a / v_b,
                    NumOp::Mod => v_a % v_b,
                }))
            }
            ExprDef::UnaryOpExpr(ref op, ref a) => {
                let v_r_a = self.run(a)?;
                let v_a = (*v_r_a).clone();
                Ok(match *op {
                    UnaryOp::Minus => to_value(-v_a.to_num()),
                    UnaryOp::Plus => to_value(v_a.to_num()),
                    UnaryOp::Not => Gc::new(!v_a),
                    _ => unreachable!(),
                })
            }
            ExprDef::BinOpExpr(BinOp::Bit(ref op), ref a, ref b) => {
                let v_r_a = self.run(a)?;
                let v_r_b = self.run(b)?;
                let v_a = (*v_r_a).clone();
                let v_b = (*v_r_b).clone();
                Ok(Gc::new(match *op {
                    BitOp::And => v_a & v_b,
                    BitOp::Or => v_a | v_b,
                    BitOp::Xor => v_a ^ v_b,
                    BitOp::Shl => v_a << v_b,
                    BitOp::Shr => v_a >> v_b,
                }))
            }
            ExprDef::BinOpExpr(BinOp::Comp(ref op), ref a, ref b) => {
                let v_r_a = self.run(a)?;
                let v_r_b = self.run(b)?;
                let v_a = v_r_a.borrow();
                let v_b = v_r_b.borrow();
                Ok(to_value(match *op {
                    CompOp::Equal if v_a.is_object() => v_r_a == v_r_b,
                    CompOp::Equal => v_a == v_b,
                    CompOp::NotEqual if v_a.is_object() => v_r_a != v_r_b,
                    CompOp::NotEqual => v_a != v_b,
                    CompOp::StrictEqual if v_a.is_object() => v_r_a == v_r_b,
                    CompOp::StrictEqual => v_a == v_b,
                    CompOp::StrictNotEqual if v_a.is_object() => v_r_a != v_r_b,
                    CompOp::StrictNotEqual => v_a != v_b,
                    CompOp::GreaterThan => v_a.to_num() > v_b.to_num(),
                    CompOp::GreaterThanOrEqual => v_a.to_num() >= v_b.to_num(),
                    CompOp::LessThan => v_a.to_num() < v_b.to_num(),
                    CompOp::LessThanOrEqual => v_a.to_num() <= v_b.to_num(),
                }))
            }
            ExprDef::BinOpExpr(BinOp::Log(ref op), ref a, ref b) => {
                let v_a = from_value::<bool>(self.run(a)?).unwrap();
                let v_b = from_value::<bool>(self.run(b)?).unwrap();
                Ok(match *op {
                    LogOp::And => to_value(v_a && v_b),
                    LogOp::Or => to_value(v_a || v_b),
                })
            }
            ExprDef::ConstructExpr(ref callee, ref args) => {
                let func = self.run(callee)?;
                let mut v_args = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    v_args.push(self.run(arg)?);
                }
                let this = ValueData::new_obj(None);
                // Create a blank object, then set its __proto__ property to the [Constructor].prototype
                this.borrow()
                    .set_field_slice(INSTANCE_PROTOTYPE, func.borrow().get_field_slice(PROTOTYPE));
                match *func {
                    ValueData::Function(ref inner_func) => match inner_func.clone().into_inner() {
                        Function::NativeFunc(ref ntv) => {
                            let func = ntv.data;
                            func(&this, v_args, &self)
                        }
                        Function::RegularFunc(ref data) => {
                            // Create new scope
                            let env = &mut self.environment;
                            env.push(new_function_environment(
                                func.clone(),
                                this.clone(),
                                Some(env.get_current_environment_ref().clone()),
                            ));

                            for i in 0..data.args.len() {
                                let name = data.args.get(i).unwrap();
                                let expr = v_args.get(i).unwrap();
                                env.create_mutable_binding(name.clone(), false);
                                env.initialize_binding(name, expr.to_owned());
                            }
                            let result = self.run(&data.expr);
                            self.environment.pop();
                            result
                        }
                    },
                    _ => Ok(Gc::new(ValueData::Undefined)),
                }
            }
            ExprDef::ReturnExpr(ref ret) => match *ret {
                Some(ref v) => self.run(v),
                None => Ok(Gc::new(ValueData::Undefined)),
            },
            ExprDef::ThrowExpr(ref ex) => Err(self.run(ex)?),
            ExprDef::AssignExpr(ref ref_e, ref val_e) => {
                let val = self.run(val_e)?;
                match ref_e.def {
                    ExprDef::LocalExpr(ref name) => {
                        self.environment.create_mutable_binding(name.clone(), false);
                        self.environment.initialize_binding(name, val.clone());
                    }
                    ExprDef::GetConstFieldExpr(ref obj, ref field) => {
                        let val_obj = self.run(obj)?;
                        val_obj.borrow().set_field(field.clone(), val.clone());
                    }
                    _ => (),
                }
                Ok(val)
            }
            ExprDef::VarDeclExpr(ref vars) => {
                for var in vars.iter() {
                    let (name, value) = var.clone();
                    let val = match value {
                        Some(v) => self.run(&v)?,
                        None => Gc::new(ValueData::Null),
                    };
                    self.environment.create_mutable_binding(name.clone(), false);
                    self.environment.initialize_binding(&name, val);
                }
                Ok(Gc::new(ValueData::Undefined))
            }
            ExprDef::LetDeclExpr(ref vars) => {
                for var in vars.iter() {
                    let (name, value) = var.clone();
                    let val = match value {
                        Some(v) => self.run(&v)?,
                        None => Gc::new(ValueData::Null),
                    };
                    self.environment.create_mutable_binding(name.clone(), false);
                    self.environment.initialize_binding(&name, val);
                }
                Ok(Gc::new(ValueData::Undefined))
            }
            ExprDef::ConstDeclExpr(ref vars) => {
                for (name, value) in vars.iter() {
                    self.environment
                        .create_immutable_binding(name.clone(), false);
                    let val = self.run(&value)?;
                    self.environment.initialize_binding(&name, val);
                }
                Ok(Gc::new(ValueData::Undefined))
            }
            ExprDef::TypeOfExpr(ref val_e) => {
                let val = self.run(val_e)?;
                Ok(to_value(match *val {
                    ValueData::Undefined => "undefined",
                    ValueData::Null | ValueData::Object(_) => "object",
                    ValueData::Boolean(_) => "boolean",
                    ValueData::Number(_) | ValueData::Integer(_) => "number",
                    ValueData::String(_) => "string",
                    ValueData::Function(_) => "function",
                }))
            }
        }
    }
}
