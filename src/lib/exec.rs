use gc::{Gc, GcCell};
use js::function::{Function, RegularFunction};
use js::object::{INSTANCE_PROTOTYPE, PROTOTYPE};
use js::value::{from_value, to_value, ResultValue, Value, ValueData};
use js::{array, console, function, json, math, object, string};
use std::borrow::Borrow;
use std::collections::HashMap;
use syntax::ast::constant::Const;
use syntax::ast::expr::{Expr, ExprDef};
use syntax::ast::op::{BinOp, BitOp, CompOp, LogOp, NumOp, UnaryOp};
/// A variable scope
pub struct Scope {
    /// The value of `this` in the scope
    pub this: Value,
    /// The variables declared in the scope
    pub vars: Value,
}

/// An execution engine
pub trait Executor {
    /// Make a new execution engine
    fn new() -> Self;
    /// Set a global variable called `name` with the value `val`
    fn set_global(&mut self, name: String, val: Value) -> Value;
    /// Resolve the global variable `name`
    fn get_global(&self, name: String) -> Value;
    /// Create a new scope and return it
    fn make_scope(&mut self, this: Value) -> &Scope;
    /// Destroy the current scope
    fn destroy_scope(&mut self) -> Scope;
    /// Run an expression
    fn run(&mut self, expr: &Expr) -> ResultValue;
}

/// A Javascript intepreter
pub struct Interpreter {
    /// An object representing the global object
    global: Value,
    /// The scopes
    pub scopes: Vec<Scope>,
}

impl Interpreter {
    #[inline(always)]
    /// Get the current scope
    pub fn scope(&self) -> &Scope {
        self.scopes.get(self.scopes.len() - 1).unwrap()
    }
}

impl Executor for Interpreter {
    fn new() -> Interpreter {
        let global = ValueData::new_obj(None);
        object::init(global.clone());
        console::init(global.clone());
        math::init(global.clone());
        array::init(global.clone());
        function::init(global.clone());
        json::init(global.clone());
        string::init(global.clone());
        Interpreter {
            global: global.clone(),
            scopes: vec![Scope {
                this: global.clone(),
                vars: global.clone(),
            }],
        }
    }

    fn set_global(&mut self, name: String, val: Value) -> Value {
        self.global.borrow().set_field(name, val)
    }

    fn get_global(&self, name: String) -> Value {
        self.global.borrow().get_field(name)
    }

    fn make_scope(&mut self, this: Value) -> &Scope {
        let scope = Scope {
            this: this,
            vars: ValueData::new_obj(None),
        };
        self.scopes.push(scope);
        &scope
    }

    fn destroy_scope(&mut self) -> Scope {
        self.scopes.pop().unwrap()
    }

    fn run(&mut self, expr: &Expr) -> ResultValue {
        match expr.def {
            ExprDef::ConstExpr(Const::Null) => Ok(to_value(None::<()>)),
            ExprDef::ConstExpr(Const::Undefined) => Ok(Gc::new(ValueData::Undefined)),
            ExprDef::ConstExpr(Const::Num(num)) => Ok(to_value(num)),
            ExprDef::ConstExpr(Const::Int(num)) => Ok(to_value(num)),
            ExprDef::ConstExpr(Const::String(str)) => Ok(to_value(str)),
            ExprDef::ConstExpr(Const::Bool(val)) => Ok(to_value(val)),
            ExprDef::ConstExpr(Const::RegExp(_, _, _)) => Ok(to_value(None::<()>)),
            ExprDef::BlockExpr(ref es) => {
                let mut obj = to_value(None::<()>);
                for e in es.iter() {
                    let val = try!(self.run(e));
                    if e == es.last().unwrap() {
                        obj = val;
                    }
                }
                Ok(obj)
            }
            ExprDef::LocalExpr(ref name) => {
                let mut val = Gc::new(ValueData::Undefined);
                for scope in self.scopes.iter().rev() {
                    let vars = scope.vars.clone();
                    let vars_ptr = vars.borrow();
                    match *vars_ptr.clone() {
                        ValueData::Object(ref obj) => match obj.borrow().get(name) {
                            Some(v) => {
                                val = v.value;
                                break;
                            }
                            None => (),
                        },
                        _ => unreachable!(),
                    }
                }
                Ok(val)
            }
            ExprDef::GetConstFieldExpr(ref obj, ref field) => {
                let val_obj = try!(self.run(obj));
                Ok(val_obj.borrow().get_field(field.clone()))
            }
            ExprDef::GetFieldExpr(ref obj, ref field) => {
                let val_obj = try!(self.run(obj));
                let val_field = try!(self.run(field));
                Ok(val_obj.borrow().get_field(val_field.borrow().to_string()))
            }
            ExprDef::CallExpr(ref callee, ref args) => {
                let (this, func) = match callee.def {
                    ExprDef::GetConstFieldExpr(ref obj, ref field) => {
                        let obj = try!(self.run(obj));
                        (obj, obj.borrow().get_field(field.clone()))
                    }
                    ExprDef::GetFieldExpr(ref obj, ref field) => {
                        let obj = try!(self.run(obj));
                        let field = try!(self.run(field));
                        (obj, obj.borrow().get_field(field.borrow().to_string()))
                    }
                    _ => (self.global.clone(), try!(self.run(&callee.clone()))),
                };
                let mut v_args = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    v_args.push(try!(self.run(arg)));
                }
                match *func {
                    ValueData::Function(ref func) => match *func.borrow() {
                        Function::NativeFunc(ref ntv) => {
                            let func = ntv.data;
                            func(this, try!(self.run(callee)), v_args)
                        }
                        Function::RegularFunc(ref data) => {
                            let scope = self.make_scope(this);
                            let scope_vars_ptr = scope.vars.borrow();
                            for i in 0..data.args.len() {
                                let name = data.args.get(i).unwrap();
                                let expr = v_args.get(i).unwrap();
                                scope_vars_ptr.set_field(name.clone(), *expr);
                            }
                            let result = self.run(&data.expr);
                            self.destroy_scope();
                            result
                        }
                    },
                    _ => Err(Gc::new(ValueData::Undefined)),
                }
            }
            ExprDef::WhileLoopExpr(ref cond, ref expr) => {
                let mut result = Gc::new(ValueData::Undefined);
                while try!(self.run(cond)).borrow().is_true() {
                    result = try!(self.run(expr));
                }
                Ok(result)
            }
            ExprDef::IfExpr(ref cond, ref expr, None) => {
                Ok(if try!(self.run(cond)).borrow().is_true() {
                    try!(self.run(expr))
                } else {
                    Gc::new(ValueData::Undefined)
                })
            }
            ExprDef::IfExpr(ref cond, ref expr, Some(ref else_e)) => {
                Ok(if try!(self.run(cond)).borrow().is_true() {
                    try!(self.run(expr))
                } else {
                    try!(self.run(else_e))
                })
            }
            ExprDef::SwitchExpr(ref val_e, ref vals, ref default) => {
                let val = try!(self.run(val_e)).clone();
                let mut result = Gc::new(ValueData::Null);
                let mut matched = false;
                for tup in vals.iter() {
                    let tup: &(Expr, Vec<Expr>) = tup;
                    let cond = &tup.0;
                    let block = &tup.1;
                    if val == try!(self.run(cond)) {
                        matched = true;
                        let last_expr = block.last().unwrap();
                        for expr in block.iter() {
                            let e_result = try!(self.run(expr));
                            if expr == last_expr {
                                result = e_result;
                            }
                        }
                    }
                }
                if !matched && default.is_some() {
                    result = try!(self.run(default.as_ref().unwrap()));
                }
                Ok(result)
            }
            ExprDef::ObjectDeclExpr(ref map) => {
                let obj = ValueData::new_obj(Some(self.global));
                for (key, val) in map.iter() {
                    obj.borrow().set_field(key.clone(), try!(self.run(val)));
                }
                Ok(obj)
            }
            ExprDef::ArrayDeclExpr(ref arr) => {
                let arr_map = ValueData::new_obj(Some(self.global));
                let mut index: i32 = 0;
                for val in arr.iter() {
                    let val = try!(self.run(val));
                    arr_map.borrow().set_field(index.to_string(), val);
                    index += 1;
                }
                arr_map.borrow().set_field_slice(
                    INSTANCE_PROTOTYPE,
                    self.get_global("Array".to_string())
                        .borrow()
                        .get_field_slice(PROTOTYPE),
                );
                arr_map.borrow().set_field_slice("length", to_value(index));
                Ok(arr_map)
            }
            ExprDef::FunctionDeclExpr(ref name, ref args, ref expr) => {
                let function =
                    Function::RegularFunc(RegularFunction::new(*expr.clone(), args.clone()));
                let val = Gc::new(ValueData::Function(GcCell::new(function)));
                if name.is_some() {
                    self.global.borrow().set_field(name.clone().unwrap(), val);
                }
                Ok(val)
            }
            ExprDef::ArrowFunctionDeclExpr(ref args, ref expr) => {
                let function =
                    Function::RegularFunc(RegularFunction::new(*expr.clone(), args.clone()));
                Ok(Gc::new(ValueData::Function(GcCell::new(function))))
            }
            ExprDef::BinOpExpr(BinOp::Num(ref op), ref a, ref b) => {
                let v_r_a = try!(self.run(a));
                let v_r_b = try!(self.run(b));
                let v_a = *v_r_a;
                let v_b = *v_r_b;
                Ok(Gc::new(match *op {
                    NumOp::Add => v_a + v_b,
                    NumOp::Sub => v_a - v_b,
                    NumOp::Mul => v_a * v_b,
                    NumOp::Div => v_a / v_b,
                    NumOp::Mod => v_a % v_b,
                }))
            }
            ExprDef::UnaryOpExpr(ref op, ref a) => {
                let v_r_a = try!(self.run(a));
                let v_a = *v_r_a;
                Ok(match *op {
                    UnaryOp::Minus => to_value(-v_a.to_num()),
                    UnaryOp::Plus => to_value(v_a.to_num()),
                    UnaryOp::Not => Gc::new(!v_a),
                    _ => unreachable!(),
                })
            }
            ExprDef::BinOpExpr(BinOp::Bit(ref op), ref a, ref b) => {
                let v_r_a = try!(self.run(a));
                let v_r_b = try!(self.run(b));
                let v_a = *v_r_a;
                let v_b = *v_r_b;
                Ok(Gc::new(match *op {
                    BitOp::And => v_a & v_b,
                    BitOp::Or => v_a | v_b,
                    BitOp::Xor => v_a ^ v_b,
                    BitOp::Shl => v_a << v_b,
                    BitOp::Shr => v_a >> v_b,
                }))
            }
            ExprDef::BinOpExpr(BinOp::Comp(ref op), ref a, ref b) => {
                let v_r_a = try!(self.run(a));
                let v_r_b = try!(self.run(b));
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
                let v_a = from_value::<bool>(try!(self.run(a))).unwrap();
                let v_b = from_value::<bool>(try!(self.run(b))).unwrap();
                Ok(match *op {
                    LogOp::And => to_value(v_a && v_b),
                    LogOp::Or => to_value(v_a || v_b),
                })
            }
            ExprDef::ConstructExpr(ref callee, ref args) => {
                let func = try!(self.run(callee));
                let mut v_args = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    v_args.push(try!(self.run(arg)));
                }
                let this = Gc::new(ValueData::Object(GcCell::new(HashMap::new())));
                this.borrow()
                    .set_field_slice(INSTANCE_PROTOTYPE, func.borrow().get_field_slice(PROTOTYPE));
                match *func {
                    ValueData::Function(ref func) => match func.clone().into_inner() {
                        Function::NativeFunc(ref ntv) => {
                            let func = ntv.data;
                            func(this, try!(self.run(callee)), v_args)
                        }
                        Function::RegularFunc(ref data) => {
                            let scope = self.make_scope(this);
                            let scope_vars_ptr = scope.vars.borrow();
                            for i in 0..data.args.len() {
                                let name = data.args.get(i).unwrap();
                                let expr = v_args.get(i).unwrap();
                                scope_vars_ptr.set_field(name.clone(), *expr);
                            }
                            let result = self.run(&data.expr);
                            self.destroy_scope();
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
            ExprDef::ThrowExpr(ref ex) => Err(try!(self.run(ex))),
            ExprDef::AssignExpr(ref ref_e, ref val_e) => {
                let val = try!(self.run(val_e));
                match ref_e.def {
                    ExprDef::LocalExpr(ref name) => {
                        self.scope()
                            .vars
                            .borrow()
                            .set_field(name.clone(), val.clone());
                    }
                    ExprDef::GetConstFieldExpr(ref obj, ref field) => {
                        let val_obj = try!(self.run(obj));
                        val_obj.borrow().set_field(field.clone(), val);
                    }
                    _ => (),
                }
                Ok(val)
            }
            ExprDef::VarDeclExpr(ref vars) => {
                let scope_vars = self.scope().vars;
                let scope_vars_ptr = scope_vars.borrow();
                for var in vars.iter() {
                    let (name, value) = var.clone();
                    let val = match value {
                        Some(v) => try!(self.run(&v)),
                        None => Gc::new(ValueData::Null),
                    };
                    scope_vars_ptr.set_field(name.clone(), val);
                }
                Ok(Gc::new(ValueData::Undefined))
            }
            ExprDef::TypeOfExpr(ref val_e) => {
                let val = try!(self.run(val_e));
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
