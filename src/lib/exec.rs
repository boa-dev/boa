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
    pub realm: Realm,
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
        let mut expr_s = Vec::new();
        let mut val_s: Vec<ResultValue> = Vec::new();

        expr_s.push((expr, false));

        while let Some((expr, can_eval)) = expr_s.pop() {
            let val = match expr.def {
                ExprDef::OpEval(ref v) => {
                    let mut s = Vec::new();

                    for t in v {
                        match t.def {
                            ExprDef::BOp(ref op) => {
                                let y = s.pop().unwrap();
                                let x = s.pop().unwrap();

                                match op {
                                    BinOp::Num(NumOp::Add) => s.push(x + y),
                                    BinOp::Num(NumOp::Sub) => s.push(x - y),
                                    BinOp::Num(NumOp::Mul) => s.push(x * y),
                                    BinOp::Num(NumOp::Div) => s.push(x / y),
                                    _ => (),
                                }
                            }
                            ExprDef::Const(Const::Num(x)) => s.push(x),
                            _ => (),
                        }
                    }

                    Ok(to_value(s.pop().unwrap()))
                }
                ExprDef::BOp(_) | ExprDef::Const(Const::Null) => Ok(to_value(None::<()>)),
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
                            break;
                        }
                        if e == es.last().expect("unable to get last value") {
                            obj = val;
                        }
                    }

                    // pop the block env
                    let _ = self.realm.environment.pop();

                    Ok(obj)
                }
                ExprDef::Local(ref name) => {
                    let val = self.realm.environment.get_binding_value(name);
                    Ok(val)
                }
                ExprDef::GetConstField(ref obj, ref field) => {
                    let val_obj = self.run(obj)?;
                    Ok(val_obj.borrow().get_field_slice(field))
                }
                ExprDef::GetField(ref obj, ref field) => {
                    let val_obj = self.run(obj)?;
                    let val_field = self.run(field)?;
                    Ok(val_obj
                        .borrow()
                        .get_field_slice(&val_field.borrow().to_string()))
                }
                ExprDef::Call(ref callee, ref args) => {
                    let (this, func) = match callee.def {
                        ExprDef::GetConstField(ref obj, ref field) => {
                            let mut obj = self.run(obj)?;
                            if obj.get_type() != "object" || obj.get_type() != "symbol" {
                                obj = self.to_object(&obj).expect("failed to convert to object");
                            }
                            (obj.clone(), obj.borrow().get_field_slice(field))
                        }
                        ExprDef::GetField(ref obj, ref field) => {
                            let obj = self.run(obj)?;
                            let field = self.run(field)?;
                            (
                                obj.clone(),
                                obj.borrow().get_field_slice(&field.borrow().to_string()),
                            )
                        }
                        _ => (self.realm.global_obj.clone(), self.run(&callee.clone())?), // 'this' binding should come from the function's self-contained environment
                    };
                    let mut v_args = Vec::with_capacity(args.len());
                    for arg in args.iter() {
                        if let ExprDef::UnaryOp(UnaryOp::Spread, ref x) = arg.def {
                            let val = self.run(x)?;
                            let mut vals = self.extract_array_properties(&val).unwrap();
                            v_args.append(&mut vals);
                            break; // after spread we don't accept any new arguments
                        }
                        v_args.push(self.run(arg)?);
                    }

                    // execute the function call itself
                    let fnct_result = self.call(&func, &this, v_args);

                    // unset the early return flag
                    self.is_return = false;

                    fnct_result
                }
                ExprDef::WhileLoop(ref cond, ref expr) => {
                    let mut result = Gc::new(ValueData::Undefined);
                    while self.run(cond)?.borrow().is_true() {
                        result = self.run(expr)?;
                    }
                    Ok(result)
                }
                ExprDef::If(ref cond, ref expr, None) => {
                    Ok(if self.run(cond)?.borrow().is_true() {
                        self.run(expr)?
                    } else {
                        Gc::new(ValueData::Undefined)
                    })
                }
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
                        obj.borrow().set_field_slice(&key.clone(), self.run(val)?);
                    }
                    Ok(obj)
                }
                ExprDef::ArrayDecl(ref arr) => {
                    let array = array::new_array(self)?;
                    let mut elements: Vec<Value> = vec![];
                    for elem in arr.iter() {
                        if let ExprDef::UnaryOp(UnaryOp::Spread, ref x) = elem.def {
                            let val = self.run(x)?;
                            let mut vals = self.extract_array_properties(&val).unwrap();
                            elements.append(&mut vals);
                            break; // after spread we don't accept any new arguments
                        }
                        elements.push(self.run(elem)?);
                    }
                    array::add_to_array_object(&array, &elements)?;
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
                    if !can_eval {
                        expr_s.push((expr, true));
                        expr_s.push((a, false));
                        expr_s.push((b, false));
                        continue;
                    }

                    /*
                    let v_r_a = self.run(a)?;
                    let v_r_b = self.run(b)?;
                    let v_a = (*v_r_a).clone();
                    let v_b = (*v_r_b).clone();
                    */
                    let v_r_a = val_s.pop().unwrap()?;
                    let v_r_b = val_s.pop().unwrap()?;
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
                        UnaryOp::Spread => Gc::new(v_a), // for now we can do nothing but return the value as-is
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
                    // turn a `Value` into a `bool`
                    let to_bool =
                        |val| from_value::<bool>(val).expect("Could not convert JS value to bool");
                    Ok(match *op {
                        LogOp::And => to_value(to_bool(self.run(a)?) && to_bool(self.run(b)?)),
                        LogOp::Or => to_value(to_bool(self.run(a)?) || to_bool(self.run(b)?)),
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
                        let v_a = (*v_r_a.borrow().get_field_slice(field)).clone();
                        let v_b = (*self.run(b)?).clone();
                        let value = exec_assign_op(op, v_a, v_b.clone());
                        v_r_a
                            .borrow()
                            .set_field_slice(&field.clone(), value.clone());
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
                        ValueData::Function(ref inner_func) => {
                            match inner_func.clone().into_inner() {
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
                                        let arg_expr =
                                            data.args.get(i).expect("Could not get data argument");
                                        let name = match arg_expr.def {
                                            ExprDef::Local(ref n) => Some(n),
                                            _ => None,
                                        }
                                        .expect("Could not get argument");
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
                            }
                        }
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
                                self.realm.environment.set_mutable_binding(
                                    &name,
                                    val.clone(),
                                    true,
                                );
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
                            val_obj
                                .borrow()
                                .set_field_slice(&field.clone(), val.clone());
                        }
                        ExprDef::GetField(ref obj, ref field) => {
                            let val_obj = self.run(obj)?;
                            let val_field = self.run(field)?;
                            val_obj.borrow().set_field(val_field, val.clone());
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
                        ValueData::Symbol(_) => "symbol",
                        ValueData::Null | ValueData::Object(_) => "object",
                        ValueData::Boolean(_) => "boolean",
                        ValueData::Number(_) | ValueData::Integer(_) => "number",
                        ValueData::String(_) => "string",
                        ValueData::Function(_) => "function",
                    }))
                }
            };

            val_s.push(val);
        }

        val_s.pop().unwrap()
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
                        let arg_expr = data.args.get(i).expect("Could not get data argument");
                        match arg_expr.def {
                            ExprDef::Local(ref name) => {
                                let expr: &Value =
                                    arguments_list.get(i).expect("Could not get argument");
                                self.realm.environment.create_mutable_binding(
                                    name.clone(),
                                    false,
                                    VariableScope::Function,
                                );
                                self.realm
                                    .environment
                                    .initialize_binding(name, expr.clone());
                            }
                            ExprDef::UnaryOp(UnaryOp::Spread, ref expr) => {
                                if let ExprDef::Local(ref name) = expr.def {
                                    let array = array::new_array(self)?;
                                    array::add_to_array_object(&array, &arguments_list[i..])?;

                                    self.realm.environment.create_mutable_binding(
                                        name.clone(),
                                        false,
                                        VariableScope::Function,
                                    );
                                    self.realm.environment.initialize_binding(name, array);
                                } else {
                                    panic!("Unsupported function argument declaration")
                                }
                            }
                            _ => panic!("Unsupported function argument declaration"),
                        }
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
            ValueData::Object(_) | ValueData::Symbol(_) => Ok(value.clone()),
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

    pub fn value_to_rust_number(&mut self, value: &Value) -> f64 {
        match *value.deref().borrow() {
            ValueData::Null => f64::from(0),
            ValueData::Boolean(boolean) => {
                if boolean {
                    f64::from(1)
                } else {
                    f64::from(0)
                }
            }
            ValueData::Number(num) => num,
            ValueData::Integer(num) => f64::from(num),
            ValueData::String(ref string) => string.parse::<f64>().unwrap(),
            ValueData::Object(_) => {
                let prim_value = self.to_primitive(value, Some("number"));
                self.to_string(&prim_value)
                    .to_string()
                    .parse::<f64>()
                    .expect("cannot parse valur to x64")
            }
            _ => {
                // TODO: Make undefined?
                f64::from(0)
            }
        }
    }

    /// `extract_array_properties` converts an array object into a rust vector of Values.   
    /// This is useful for the spread operator, for any other object an `Err` is returned
    fn extract_array_properties(&mut self, value: &Value) -> Result<Vec<Gc<ValueData>>, ()> {
        if let ValueData::Object(ref x) = *value.deref().borrow() {
            // Check if object is array
            if x.deref().borrow().kind == ObjectKind::Array {
                let length: i32 =
                    self.value_to_rust_number(&value.get_field_slice("length")) as i32;
                let values: Vec<Gc<ValueData>> = (0..length)
                    .map(|idx| value.get_field_slice(&idx.to_string()))
                    .collect::<Vec<Value>>();
                return Ok(values);
            }

            return Err(());
        }

        Err(())
    }
}

#[cfg(test)]
mod tests {
    use crate::exec;
    use crate::exec::Executor;
    use crate::forward;
    use crate::realm::Realm;

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
    fn spread_with_arguments() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);

        let scenario = r#"
            const a = [1, "test", 3, 4];
            function foo(...a) {
                return arguments;
            }
            
            var result = foo(...a);
        "#;
        forward(&mut engine, scenario);
        let one = forward(&mut engine, "result[0]");
        assert_eq!(one, String::from("1"));

        let two = forward(&mut engine, "result[1]");
        assert_eq!(two, String::from("test"));

        let three = forward(&mut engine, "result[2]");
        assert_eq!(three, String::from("3"));

        let four = forward(&mut engine, "result[3]");
        assert_eq!(four, String::from("4"));
    }

    #[test]
    fn array_rest_with_arguments() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);

        let scenario = r#"
            var b = [4, 5, 6]
            var a = [1, 2, 3, ...b];
        "#;
        forward(&mut engine, scenario);
        let one = forward(&mut engine, "a");
        assert_eq!(one, String::from("[ 1, 2, 3, 4, 5, 6 ]"));
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

    #[test]
    fn test_early_return() {
        let early_return = r#"
        function early_return() {
            if (true) {
                return true;
            }
            return false;
        }
        early_return()
        "#;
        assert_eq!(exec(early_return), String::from("true"));
        let early_return = r#"
        function nested_fnct() {
            return "nested";
        }
        function outer_fnct() {
            nested_fnct();
            return "outer";
        }
        outer_fnct()
        "#;
        assert_eq!(exec(early_return), String::from("outer"));
    }

    #[test]
    fn test_short_circuit_evaluation() {
        // OR operation
        assert_eq!(exec("true || true"), String::from("true"));
        assert_eq!(exec("true || false"), String::from("true"));
        assert_eq!(exec("false || true"), String::from("true"));
        assert_eq!(exec("false || false"), String::from("false"));

        // the second operand must NOT be evaluated if the first one resolve to `true`.
        let short_circuit_eval = r#"
        function add_one(counter) {
            counter.value += 1;
            return true;
        }
        let counter = { value: 0 };
        let _ = add_one(counter) || add_one(counter);
        counter.value
        "#;
        assert_eq!(exec(short_circuit_eval), String::from("1"));

        // the second operand must be evaluated if the first one resolve to `false`.
        let short_circuit_eval = r#"
        function add_one(counter) {
            counter.value += 1;
            return false;
        }
        let counter = { value: 0 };
        let _ = add_one(counter) || add_one(counter);
        counter.value
        "#;
        assert_eq!(exec(short_circuit_eval), String::from("2"));

        // AND operation
        assert_eq!(exec("true && true"), String::from("true"));
        assert_eq!(exec("true && false"), String::from("false"));
        assert_eq!(exec("false && true"), String::from("false"));
        assert_eq!(exec("false && false"), String::from("false"));

        // the second operand must be evaluated if the first one resolve to `true`.
        let short_circuit_eval = r#"
        function add_one(counter) {
            counter.value += 1;
            return true;
        }
        let counter = { value: 0 };
        let _ = add_one(counter) && add_one(counter);
        counter.value
        "#;
        assert_eq!(exec(short_circuit_eval), String::from("2"));

        // the second operand must NOT be evaluated if the first one resolve to `false`.
        let short_circuit_eval = r#"
        function add_one(counter) {
            counter.value += 1;
            return false;
        }
        let counter = { value: 0 };
        let _ = add_one(counter) && add_one(counter);
        counter.value
        "#;
        assert_eq!(exec(short_circuit_eval), String::from("1"));
    }
}
