use crate::{
    builtins::{
        array,
        function::{create_unmapped_arguments_object, Function, RegularFunction},
        object::{ObjectKind, INSTANCE_PROTOTYPE, PROTOTYPE},
        value::{from_value, to_value, ResultValue, Value, ValueData},
    },
    environment::lexical_environment::{
        new_declarative_environment, new_function_environment, VariableScope,
    },
    realm::Realm,
    syntax::ast::{
        constant::Const,
        expr::{Expr, ExprDef},
        op::{AssignOp, BinOp, BitOp, CompOp, LogOp, NumOp, UnaryOp},
    },
};
use gc::{Gc, GcCell};
use std::{
    borrow::Borrow,
    ops::{Deref, DerefMut},
};

/// An execution engine
pub trait Executor {
    /// Make a new execution engine
    fn new(realm: Realm) -> Self;
    /// Run an expression
    fn run(&mut self, expr: &Expr) -> ResultValue;
}

/// A Javascript intepreter
#[derive(Debug)]
pub struct Interpreter {
    is_return: bool,
    /// realm holds both the global object and the environment
    realm: Realm,
}

fn exec_assign_op(op: &AssignOp, v_a: ValueData, v_b: ValueData) -> Value {
    Gc::new(match *op {
        AssignOp::Add => v_a + v_b,
        AssignOp::Sub => v_a - v_b,
        AssignOp::Mul => v_a * v_b,
        AssignOp::Pow => v_a.as_num_to_power(v_b),
        AssignOp::Div => v_a / v_b,
        AssignOp::Mod => v_a % v_b,
        AssignOp::And => v_a & v_b,
        AssignOp::Or => v_a | v_b,
        AssignOp::Xor => v_a ^ v_b,
        AssignOp::Shl => v_a << v_b,
        AssignOp::Shr => v_a << v_b,
    })
}

impl Executor for Interpreter {
    fn new(realm: Realm) -> Self {
        Interpreter {
            realm,
            is_return: false,
        }
    }

    #[allow(clippy::match_same_arms)]
    fn run(&mut self, expr: &Expr) -> ResultValue {
        match expr.def {
            ExprDef::Const(Const::Null) => Ok(to_value(None::<()>)),
            ExprDef::Const(Const::Undefined) => Ok(Gc::new(ValueData::Undefined)),
            ExprDef::Const(Const::Num(num)) => Ok(to_value(num)),
            ExprDef::Const(Const::Int(num)) => Ok(to_value(num)),
            // we can't move String from Const into value, because const is a garbage collected value
            // Which means Drop() get's called on Const, but str will be gone at that point.
            // Do Const values need to be garbage collected? We no longer need them once we've generated Values
            ExprDef::Const(Const::String(ref str)) => Ok(to_value(str.to_owned())),
            ExprDef::Const(Const::Bool(val)) => Ok(to_value(val)),
            ExprDef::Block(ref es) => {
                {
                    let env = &mut self.realm.environment;
                    env.push(new_declarative_environment(Some(
                        env.get_current_environment_ref().clone(),
                    )));
                }

                let mut obj = to_value(None::<()>);
                for e in es.iter() {
                    let val = self.run(e)?;
                    // early return
                    if self.is_return {
                        obj = val;
                        self.is_return = false;
                        break;
                    }
                    if e == es.last().expect("unable to get last value") {
                        obj = val;
                    }
                }

                self.realm.environment.pop();
                Ok(obj)
            }
            ExprDef::Local(ref name) => {
                let val = self.realm.environment.get_binding_value(name);
                Ok(val)
            }
            ExprDef::GetConstField(ref obj, ref field) => {
                let val_obj = self.run(obj)?;
                Ok(val_obj.borrow().get_field(field))
            }
            ExprDef::GetField(ref obj, ref field) => {
                let val_obj = self.run(obj)?;
                let val_field = self.run(field)?;
                Ok(val_obj.borrow().get_field(&val_field.borrow().to_string()))
            }
            ExprDef::Call(ref callee, ref args) => {
                let (this, func) = match callee.def {
                    ExprDef::GetConstField(ref obj, ref field) => {
                        let mut obj = self.run(obj)?;
                        if obj.get_type() != "object" {
                            obj = self.to_object(&obj).expect("failed to convert to object");
                        }
                        (obj.clone(), obj.borrow().get_field(field))
                    }
                    ExprDef::GetField(ref obj, ref field) => {
                        let obj = self.run(obj)?;
                        let field = self.run(field)?;
                        (
                            obj.clone(),
                            obj.borrow().get_field(&field.borrow().to_string()),
                        )
                    }
                    _ => (self.realm.global_obj.clone(), self.run(&callee.clone())?), // 'this' binding should come from the function's self-contained environment
                };
                let mut v_args = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    v_args.push(self.run(arg)?);
                }

                self.call(&func, &this, v_args)
            }
            ExprDef::WhileLoop(ref cond, ref expr) => {
                let mut result = Gc::new(ValueData::Undefined);
                while self.run(cond)?.borrow().is_true() {
                    result = self.run(expr)?;
                }
                Ok(result)
            }
            ExprDef::If(ref cond, ref expr, None) => Ok(if self.run(cond)?.borrow().is_true() {
                self.run(expr)?
            } else {
                Gc::new(ValueData::Undefined)
            }),
            ExprDef::If(ref cond, ref expr, Some(ref else_e)) => {
                Ok(if self.run(cond)?.borrow().is_true() {
                    self.run(expr)?
                } else {
                    self.run(else_e)?
                })
            }
            ExprDef::Switch(ref val_e, ref vals, ref default) => {
                let val = self.run(val_e)?.clone();
                let mut result = Gc::new(ValueData::Null);
                let mut matched = false;
                for tup in vals.iter() {
                    let tup: &(Expr, Vec<Expr>) = tup;
                    let cond = &tup.0;
                    let block = &tup.1;
                    if val == self.run(cond)? {
                        matched = true;
                        let last_expr = block.last().expect("Block has no expressions");
                        for expr in block.iter() {
                            let e_result = self.run(expr)?;
                            if expr == last_expr {
                                result = e_result;
                            }
                        }
                    }
                }
                if !matched && default.is_some() {
                    result = self.run(
                        default
                            .as_ref()
                            .expect("Could not get default as reference"),
                    )?;
                }
                Ok(result)
            }
            ExprDef::ObjectDecl(ref map) => {
                let global_val = &self
                    .realm
                    .environment
                    .get_global_object()
                    .expect("Could not get the global object");
                let obj = ValueData::new_obj(Some(global_val));
                for (key, val) in map.iter() {
                    obj.borrow().set_field(key.clone(), self.run(val)?);
                }
                Ok(obj)
            }
            ExprDef::ArrayDecl(ref arr) => {
                let array = array::new_array(self)?;
                let elements: Result<Vec<_>, _> = arr.iter().map(|val| self.run(val)).collect();
                array::add_to_array_object(&array, &elements?)?;
                Ok(array)
            }
            ExprDef::FunctionDecl(ref name, ref args, ref expr) => {
                let function =
                    Function::RegularFunc(RegularFunction::new(*expr.clone(), args.clone()));
                let val = Gc::new(ValueData::Function(Box::new(GcCell::new(function))));
                if name.is_some() {
                    self.realm.environment.create_mutable_binding(
                        name.clone().expect("No name was supplied"),
                        false,
                        VariableScope::Function,
                    );
                    self.realm.environment.initialize_binding(
                        name.as_ref().expect("Could not get name as reference"),
                        val.clone(),
                    )
                }
                Ok(val)
            }
            ExprDef::ArrowFunctionDecl(ref args, ref expr) => {
                let function =
                    Function::RegularFunc(RegularFunction::new(*expr.clone(), args.clone()));
                Ok(Gc::new(ValueData::Function(Box::new(GcCell::new(
                    function,
                )))))
            }
            ExprDef::BinOp(BinOp::Num(ref op), ref a, ref b) => {
                let v_r_a = self.run(a)?;
                let v_r_b = self.run(b)?;
                let v_a = (*v_r_a).clone();
                let v_b = (*v_r_b).clone();
                Ok(Gc::new(match *op {
                    NumOp::Add => v_a + v_b,
                    NumOp::Sub => v_a - v_b,
                    NumOp::Mul => v_a * v_b,
                    NumOp::Pow => v_a.as_num_to_power(v_b),
                    NumOp::Div => v_a / v_b,
                    NumOp::Mod => v_a % v_b,
                }))
            }
            ExprDef::UnaryOp(ref op, ref a) => {
                let v_r_a = self.run(a)?;
                let v_a = (*v_r_a).clone();
                Ok(match *op {
                    UnaryOp::Minus => to_value(-v_a.to_num()),
                    UnaryOp::Plus => to_value(v_a.to_num()),
                    UnaryOp::Not => Gc::new(!v_a),
                    UnaryOp::Tilde => {
                        let num_v_a = v_a.to_num();
                        // NOTE: possible UB: https://github.com/rust-lang/rust/issues/10184
                        to_value(if num_v_a.is_nan() {
                            -1
                        } else {
                            !(num_v_a as i32)
                        })
                    }
                    _ => unreachable!(),
                })
            }
            ExprDef::BinOp(BinOp::Bit(ref op), ref a, ref b) => {
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
            ExprDef::BinOp(BinOp::Comp(ref op), ref a, ref b) => {
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
            ExprDef::BinOp(BinOp::Log(ref op), ref a, ref b) => {
                let v_a =
                    from_value::<bool>(self.run(a)?).expect("Could not convert JS value to bool");
                let v_b =
                    from_value::<bool>(self.run(b)?).expect("Could not convert JS value to bool");
                Ok(match *op {
                    LogOp::And => to_value(v_a && v_b),
                    LogOp::Or => to_value(v_a || v_b),
                })
            }
            ExprDef::BinOp(BinOp::Assign(ref op), ref a, ref b) => match a.def {
                ExprDef::Local(ref name) => {
                    let v_a = (*self.realm.environment.get_binding_value(&name)).clone();
                    let v_b = (*self.run(b)?).clone();
                    let value = exec_assign_op(op, v_a, v_b);
                    self.realm
                        .environment
                        .set_mutable_binding(&name, value.clone(), true);
                    Ok(value)
                }
                ExprDef::GetConstField(ref obj, ref field) => {
                    let v_r_a = self.run(obj)?;
                    let v_a = (*v_r_a.borrow().get_field(field)).clone();
                    let v_b = (*self.run(b)?).clone();
                    let value = exec_assign_op(op, v_a, v_b.clone());
                    v_r_a.borrow().set_field(field.clone(), value.clone());
                    Ok(value)
                }
                _ => Ok(Gc::new(ValueData::Undefined)),
            },
            ExprDef::Construct(ref callee, ref args) => {
                let func_object = self.run(callee)?;
                let mut v_args = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    v_args.push(self.run(arg)?);
                }
                let this = ValueData::new_obj(None);
                // Create a blank object, then set its __proto__ property to the [Constructor].prototype
                this.borrow().set_internal_slot(
                    INSTANCE_PROTOTYPE,
                    func_object.borrow().get_field_slice(PROTOTYPE),
                );

                let construct = func_object.get_internal_slot("construct");

                match *construct {
                    ValueData::Function(ref inner_func) => match inner_func.clone().into_inner() {
                        Function::NativeFunc(ref ntv) => {
                            let func = ntv.data;
                            match func(&this, &v_args, self) {
                                Ok(_) => Ok(this),
                                Err(ref v) => Err(v.clone()),
                            }
                        }
                        Function::RegularFunc(ref data) => {
                            // Create new scope
                            let env = &mut self.realm.environment;
                            env.push(new_function_environment(
                                construct.clone(),
                                this.clone(),
                                Some(env.get_current_environment_ref().clone()),
                            ));

                            for i in 0..data.args.len() {
                                let name = data.args.get(i).expect("Could not get data argument");
                                let expr = v_args.get(i).expect("Could not get argument");
                                env.create_mutable_binding(
                                    name.clone(),
                                    false,
                                    VariableScope::Function,
                                );
                                env.initialize_binding(name, expr.to_owned());
                            }
                            let result = self.run(&data.expr);
                            self.realm.environment.pop();
                            result
                        }
                    },
                    _ => Ok(Gc::new(ValueData::Undefined)),
                }
            }
            ExprDef::Return(ref ret) => {
                let result = match *ret {
                    Some(ref v) => self.run(v),
                    None => Ok(Gc::new(ValueData::Undefined)),
                };
                // Set flag for return
                self.is_return = true;
                result
            }
            ExprDef::Throw(ref ex) => Err(self.run(ex)?),
            ExprDef::Assign(ref ref_e, ref val_e) => {
                let val = self.run(val_e)?;
                match ref_e.def {
                    ExprDef::Local(ref name) => {
                        if self.realm.environment.has_binding(name) {
                            // Binding already exists
                            self.realm
                                .environment
                                .set_mutable_binding(&name, val.clone(), true);
                        } else {
                            self.realm.environment.create_mutable_binding(
                                name.clone(),
                                true,
                                VariableScope::Function,
                            );
                            self.realm.environment.initialize_binding(name, val.clone());
                        }
                    }
                    ExprDef::GetConstField(ref obj, ref field) => {
                        let val_obj = self.run(obj)?;
                        val_obj.borrow().set_field(field.clone(), val.clone());
                    }
                    ExprDef::GetField(ref obj, ref field) => {
                        let val_obj = self.run(obj)?;
                        let val_field = self.run(field)?;
                        val_obj
                            .borrow()
                            .set_field(val_field.to_string(), val.clone());
                    }
                    _ => (),
                }
                Ok(val)
            }
            ExprDef::VarDecl(ref vars) => {
                for var in vars.iter() {
                    let (name, value) = var.clone();
                    let val = match value {
                        Some(v) => self.run(&v)?,
                        None => Gc::new(ValueData::Undefined),
                    };
                    self.realm.environment.create_mutable_binding(
                        name.clone(),
                        false,
                        VariableScope::Function,
                    );
                    self.realm.environment.initialize_binding(&name, val);
                }
                Ok(Gc::new(ValueData::Undefined))
            }
            ExprDef::LetDecl(ref vars) => {
                for var in vars.iter() {
                    let (name, value) = var.clone();
                    let val = match value {
                        Some(v) => self.run(&v)?,
                        None => Gc::new(ValueData::Undefined),
                    };
                    self.realm.environment.create_mutable_binding(
                        name.clone(),
                        false,
                        VariableScope::Block,
                    );
                    self.realm.environment.initialize_binding(&name, val);
                }
                Ok(Gc::new(ValueData::Undefined))
            }
            ExprDef::ConstDecl(ref vars) => {
                for (name, value) in vars.iter() {
                    self.realm.environment.create_immutable_binding(
                        name.clone(),
                        false,
                        VariableScope::Block,
                    );
                    let val = self.run(&value)?;
                    self.realm.environment.initialize_binding(&name, val);
                }
                Ok(Gc::new(ValueData::Undefined))
            }
            ExprDef::TypeOf(ref val_e) => {
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

impl Interpreter {
    /// Get the Interpreter's realm
    pub(crate) fn get_realm(&self) -> &Realm {
        &self.realm
    }

    /// https://tc39.es/ecma262/#sec-call
    pub(crate) fn call(&mut self, f: &Value, v: &Value, arguments_list: Vec<Value>) -> ResultValue {
        // All functions should be objects, and eventually will be.
        // During this transition call will support both native functions and function objects
        match (*f).deref() {
            ValueData::Object(ref obj) => {
                let func: Value = obj.borrow_mut().deref_mut().get_internal_slot("call");
                if !func.is_undefined() {
                    return self.call(&func, v, arguments_list);
                }
                // TODO: error object should be here
                Err(Gc::new(ValueData::Undefined))
            }
            ValueData::Function(ref inner_func) => match *inner_func.deref().borrow() {
                Function::NativeFunc(ref ntv) => {
                    let func = ntv.data;
                    func(v, &arguments_list, self)
                }
                Function::RegularFunc(ref data) => {
                    let env = &mut self.realm.environment;
                    // New target (second argument) is only needed for constructors, just pass undefined
                    let undefined = Gc::new(ValueData::Undefined);
                    env.push(new_function_environment(
                        f.clone(),
                        undefined,
                        Some(env.get_current_environment_ref().clone()),
                    ));
                    for i in 0..data.args.len() {
                        let name = data.args.get(i).expect("Could not get data argument");
                        let expr: &Value = arguments_list.get(i).expect("Could not get argument");
                        self.realm.environment.create_mutable_binding(
                            name.clone(),
                            false,
                            VariableScope::Function,
                        );
                        self.realm
                            .environment
                            .initialize_binding(name, expr.clone());
                    }

                    // Add arguments object
                    let arguments_obj = create_unmapped_arguments_object(arguments_list);
                    self.realm.environment.create_mutable_binding(
                        "arguments".to_string(),
                        false,
                        VariableScope::Function,
                    );
                    self.realm
                        .environment
                        .initialize_binding("arguments", arguments_obj);

                    let result = self.run(&data.expr);
                    self.realm.environment.pop();
                    result
                }
            },
            _ => Err(Gc::new(ValueData::Undefined)),
        }
    }

    /// https://tc39.es/ecma262/#sec-ordinarytoprimitive
    fn ordinary_to_primitive(&mut self, o: &Value, hint: &str) -> Value {
        debug_assert!(o.get_type() == "object");
        debug_assert!(hint == "string" || hint == "number");
        let method_names: Vec<&str> = if hint == "string" {
            vec!["toString", "valueOf"]
        } else {
            vec!["valueOf", "toString"]
        };
        for name in method_names.iter() {
            let method: Value = o.get_field_slice(name);
            if method.is_function() {
                let result = self.call(&method, &o, vec![]);
                match result {
                    Ok(val) => {
                        if val.is_object() {
                            // TODO: throw exception
                            continue;
                        } else {
                            return val;
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        Gc::new(ValueData::Undefined)
    }

    /// The abstract operation ToPrimitive takes an input argument and an optional argument PreferredType.
    /// https://tc39.es/ecma262/#sec-toprimitive
    #[allow(clippy::wrong_self_convention)]
    pub fn to_primitive(&mut self, input: &Value, preferred_type: Option<&str>) -> Value {
        let mut hint: &str;
        match (*input).deref() {
            ValueData::Object(_) => {
                hint = match preferred_type {
                    None => "default",
                    Some(pt) => match pt {
                        "string" => "string",
                        "number" => "number",
                        _ => "default",
                    },
                };

                // Skip d, e we don't support Symbols yet
                // TODO: add when symbols are supported
                if hint == "default" {
                    hint = "number";
                };

                self.ordinary_to_primitive(&input, hint)
            }
            _ => input.clone(),
        }
    }
    /// to_string() converts a value into a String
    /// https://tc39.es/ecma262/#sec-tostring
    #[allow(clippy::wrong_self_convention)]
    pub fn to_string(&mut self, value: &Value) -> Value {
        match *value.deref().borrow() {
            ValueData::Undefined => to_value("undefined"),
            ValueData::Null => to_value("null"),
            ValueData::Boolean(ref boolean) => to_value(boolean.to_string()),
            ValueData::Number(ref num) => to_value(num.to_string()),
            ValueData::Integer(ref num) => to_value(num.to_string()),
            ValueData::String(ref string) => to_value(string.clone()),
            ValueData::Object(_) => {
                let prim_value = self.to_primitive(value, Some("string"));
                self.to_string(&prim_value)
            }
            _ => to_value("function(){...}"),
        }
    }

    /// The abstract operation ToObject converts argument to a value of type Object
    /// https://tc39.es/ecma262/#sec-toobject
    #[allow(clippy::wrong_self_convention)]
    pub fn to_object(&mut self, value: &Value) -> ResultValue {
        match *value.deref().borrow() {
            ValueData::Undefined
            | ValueData::Function(_)
            | ValueData::Integer(_)
            | ValueData::Null => Err(Gc::new(ValueData::Undefined)),
            ValueData::Boolean(_) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("Boolean")
                    .get_field_slice(PROTOTYPE);

                let bool_obj = ValueData::new_obj_from_prototype(proto, ObjectKind::Boolean);
                bool_obj.set_internal_slot("BooleanData", value.clone());
                Ok(bool_obj)
            }
            ValueData::Number(_) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("Number")
                    .get_field_slice(PROTOTYPE);
                let number_obj = ValueData::new_obj_from_prototype(proto, ObjectKind::Number);
                number_obj.set_internal_slot("NumberData", value.clone());
                Ok(number_obj)
            }
            ValueData::String(_) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("String")
                    .get_field_slice(PROTOTYPE);
                let string_obj = ValueData::new_obj_from_prototype(proto, ObjectKind::String);
                string_obj.set_internal_slot("StringData", value.clone());
                Ok(string_obj)
            }
            ValueData::Object(_) => Ok(value.clone()),
        }
    }

    /// value_to_rust_string() converts a value into a rust heap allocated string
    pub fn value_to_rust_string(&mut self, value: &Value) -> String {
        match *value.deref().borrow() {
            ValueData::Null => String::from("null"),
            ValueData::Boolean(ref boolean) => boolean.to_string(),
            ValueData::Number(ref num) => num.to_string(),
            ValueData::Integer(ref num) => num.to_string(),
            ValueData::String(ref string) => string.clone(),
            ValueData::Object(_) => {
                let prim_value = self.to_primitive(value, Some("string"));
                self.to_string(&prim_value).to_string()
            }
            _ => String::from("undefined"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::exec;

    #[test]
    fn empty_let_decl_undefined() {
        let scenario = r#"
        let a;
        a == undefined;
        "#;

        let pass = String::from("true");

        assert_eq!(exec(scenario), pass);
    }

    #[test]
    fn empty_var_decl_undefined() {
        let scenario = r#"
        let b;
        b == undefined;
        "#;

        let pass = String::from("true");

        assert_eq!(exec(scenario), pass);
    }

    #[test]
    fn object_field_set() {
        let scenario = r#"
        let m = {};
        m['key'] = 22;
        m['key']
        "#;
        assert_eq!(exec(scenario), String::from("22"));
    }

    #[test]
    fn array_field_set() {
        let element_changes = r#"
        let m = [1, 2, 3];
        m[1] = 5;
        m[1]
        "#;
        assert_eq!(exec(element_changes), String::from("5"));

        let length_changes = r#"
        let m = [1, 2, 3];
        m[10] = 52;
        m.length
        "#;
        assert_eq!(exec(length_changes), String::from("11"));

        let negative_index_wont_affect_length = r#"
        let m = [1, 2, 3];
        m[-11] = 5;
        m.length
        "#;
        assert_eq!(exec(negative_index_wont_affect_length), String::from("3"));

        let non_num_key_wont_affect_length = r#"
        let m = [1, 2, 3];
        m["magic"] = 5;
        m.length
        "#;
        assert_eq!(exec(non_num_key_wont_affect_length), String::from("3"));
    }

    #[test]
    fn test_tilde_operator() {
        let float = r#"
        let f = -1.2;
        ~f
        "#;
        assert_eq!(exec(float), String::from("0"));

        let numeric = r#"
        let f = 1789;
        ~f
        "#;
        assert_eq!(exec(numeric), String::from("-1790"));

        // TODO: enable test after we have NaN
        // let nan = r#"
        // var m = NaN;
        // ~m
        // "#;
        // assert_eq!(exec(nan), String::from("-1"));

        let object = r#"
        let m = {};
        ~m
        "#;
        assert_eq!(exec(object), String::from("-1"));

        let boolean_true = r#"
        ~true
        "#;
        assert_eq!(exec(boolean_true), String::from("-2"));

        let boolean_false = r#"
        ~false
        "#;
        assert_eq!(exec(boolean_false), String::from("-1"));
    }
}
