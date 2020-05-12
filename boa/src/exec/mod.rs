//! Execution of the AST, this is where the interpreter actually runs

#[cfg(test)]
mod tests;

use crate::{
    builtins::{
        array,
        function::{Function as FunctionObject, FunctionBody, ThisMode},
        object::{
            internal_methods_trait::ObjectInternalMethods, Object, ObjectKind, INSTANCE_PROTOTYPE,
            PROTOTYPE,
        },
        property::Property,
        value::{ResultValue, Value, ValueData},
    },
    environment::lexical_environment::{new_declarative_environment, VariableScope},
    realm::Realm,
    syntax::ast::{
        constant::Const,
        node::{MethodDefinitionKind, Node, PropertyDefinition},
        op::{AssignOp, BinOp, BitOp, CompOp, LogOp, NumOp, UnaryOp},
    },
};
use std::{
    borrow::{Borrow, BorrowMut},
    ops::Deref,
};

/// An execution engine
pub trait Executor {
    /// Make a new execution engine
    fn new(realm: Realm) -> Self;
    /// Run an expression
    fn run(&mut self, expr: &Node) -> ResultValue;
}

/// A Javascript intepreter
#[derive(Debug)]
pub struct Interpreter {
    is_return: bool,
    /// realm holds both the global object and the environment
    pub realm: Realm,
}

fn exec_assign_op(op: &AssignOp, v_a: Value, v_b: Value) -> Value {
    match *op {
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

impl Executor for Interpreter {
    fn new(realm: Realm) -> Self {
        Self {
            realm,
            is_return: false,
        }
    }

    #[allow(clippy::match_same_arms)]
    fn run(&mut self, node: &Node) -> ResultValue {
        match *node {
            Node::Const(Const::Null) => Ok(Value::null()),
            Node::Const(Const::Undefined) => Ok(Value::undefined()),
            Node::Const(Const::Num(num)) => Ok(Value::rational(num)),
            Node::Const(Const::Int(num)) => Ok(Value::integer(num)),
            // we can't move String from Const into value, because const is a garbage collected value
            // Which means Drop() get's called on Const, but str will be gone at that point.
            // Do Const values need to be garbage collected? We no longer need them once we've generated Values
            Node::Const(Const::String(ref value)) => Ok(Value::string(value.to_string())),
            Node::Const(Const::Bool(value)) => Ok(Value::boolean(value)),
            Node::Block(ref es) => {
                {
                    let env = &mut self.realm.environment;
                    env.push(new_declarative_environment(Some(
                        env.get_current_environment_ref().clone(),
                    )));
                }

                let mut obj = Value::null();
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
            Node::Local(ref name) => {
                let val = self.realm.environment.get_binding_value(name);
                Ok(val)
            }
            Node::GetConstField(ref obj, ref field) => {
                let val_obj = self.run(obj)?;
                Ok(val_obj.borrow().get_field_slice(field))
            }
            Node::GetField(ref obj, ref field) => {
                let val_obj = self.run(obj)?;
                let val_field = self.run(field)?;
                Ok(val_obj
                    .borrow()
                    .get_field_slice(&val_field.borrow().to_string()))
            }
            Node::Call(ref callee, ref args) => {
                let (mut this, func) = match callee.deref() {
                    Node::GetConstField(ref obj, ref field) => {
                        let mut obj = self.run(obj)?;
                        if obj.get_type() != "object" || obj.get_type() != "symbol" {
                            obj = self.to_object(&obj).expect("failed to convert to object");
                        }
                        (obj.clone(), obj.borrow().get_field_slice(field))
                    }
                    Node::GetField(ref obj, ref field) => {
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
                    if let Node::Spread(ref x) = arg.deref() {
                        let val = self.run(x)?;
                        let mut vals = self.extract_array_properties(&val).unwrap();
                        v_args.append(&mut vals);
                        break; // after spread we don't accept any new arguments
                    }
                    v_args.push(self.run(arg)?);
                }

                // execute the function call itself
                let fnct_result = self.call(&func, &mut this, &v_args);

                // unset the early return flag
                self.is_return = false;

                fnct_result
            }
            Node::WhileLoop(ref cond, ref expr) => {
                let mut result = Value::undefined();
                while self.run(cond)?.borrow().is_true() {
                    result = self.run(expr)?;
                }
                Ok(result)
            }
            Node::DoWhileLoop(ref body, ref cond) => {
                let mut result = self.run(body)?;
                while self.run(cond)?.borrow().is_true() {
                    result = self.run(body)?;
                }
                Ok(result)
            }
            Node::ForLoop(ref init, ref cond, ref step, ref body) => {
                if let Some(init) = init {
                    self.run(init)?;
                }

                while match cond {
                    Some(cond) => self.run(cond)?.borrow().is_true(),
                    None => true,
                } {
                    self.run(body)?;

                    if let Some(step) = step {
                        self.run(step)?;
                    }
                }

                Ok(Value::undefined())
            }
            Node::If(ref cond, ref expr, None) => Ok(if self.run(cond)?.borrow().is_true() {
                self.run(expr)?
            } else {
                Value::undefined()
            }),
            Node::If(ref cond, ref expr, Some(ref else_e)) => {
                Ok(if self.run(cond)?.borrow().is_true() {
                    self.run(expr)?
                } else {
                    self.run(else_e)?
                })
            }
            Node::Switch(ref val_e, ref vals, ref default) => {
                let val = self.run(val_e)?;
                let mut result = Value::null();
                let mut matched = false;
                for tup in vals.iter() {
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
            Node::Object(ref properties) => {
                let global_val = &self
                    .realm
                    .environment
                    .get_global_object()
                    .expect("Could not get the global object");
                let obj = Value::new_object(Some(global_val));

                // TODO: Implement the rest of the property types.
                for property in properties.iter() {
                    match property {
                        PropertyDefinition::Property(key, value) => {
                            obj.borrow().set_field_slice(&key.clone(), self.run(value)?);
                        }
                        PropertyDefinition::MethodDefinition(kind, name, func) => {
                            if let MethodDefinitionKind::Ordinary = kind {
                                obj.borrow().set_field_slice(&name.clone(), self.run(func)?);
                            } else {
                                // TODO: Implement other types of MethodDefinitionKinds.
                                unimplemented!("other types of property method definitions.");
                            }
                        }
                        i => unimplemented!("{:?} type of property", i),
                    }
                }

                Ok(obj)
            }
            Node::ArrayDecl(ref arr) => {
                let array = array::new_array(self)?;
                let mut elements = Vec::new();
                for elem in arr.iter() {
                    if let Node::Spread(ref x) = elem.deref() {
                        let val = self.run(x)?;
                        let mut vals = self.extract_array_properties(&val).unwrap();
                        elements.append(&mut vals);
                        continue; // Don't push array after spread
                    }
                    elements.push(self.run(elem)?);
                }
                array::add_to_array_object(&array, &elements)?;
                Ok(array)
            }
            // <https://tc39.es/ecma262/#sec-createdynamicfunction>
            Node::FunctionDecl(ref name, ref args, ref expr) => {
                // Todo: Function.prototype doesn't exist yet, so the prototype right now is the Object.prototype
                // let proto = &self
                //     .realm
                //     .environment
                //     .get_global_object()
                //     .expect("Could not get the global object")
                //     .get_field_slice("Object")
                //     .get_field_slice("Prototype");

                let func = FunctionObject::create_ordinary(
                    args.clone(), // TODO: args shouldn't need to be a reference it should be passed by value
                    self.realm.environment.get_current_environment().clone(),
                    FunctionBody::Ordinary(*expr.clone()),
                    ThisMode::NonLexical,
                );

                let mut new_func = Object::function();
                new_func.set_call(func);
                let val = Value::from(new_func);
                val.set_field_slice("length", Value::from(args.len()));

                // Set the name and assign it in the current environment
                val.set_field_slice("name", Value::from(name.clone()));
                self.realm.environment.create_mutable_binding(
                    name.clone(),
                    false,
                    VariableScope::Function,
                );

                self.realm.environment.initialize_binding(name, val.clone());

                Ok(val)
            }
            // <https://tc39.es/ecma262/#sec-createdynamicfunction>
            Node::FunctionExpr(ref name, ref args, ref expr) => {
                // Todo: Function.prototype doesn't exist yet, so the prototype right now is the Object.prototype
                // let proto = &self
                //     .realm
                //     .environment
                //     .get_global_object()
                //     .expect("Could not get the global object")
                //     .get_field_slice("Object")
                //     .get_field_slice("Prototype");

                let func = FunctionObject::create_ordinary(
                    args.clone(), // TODO: args shouldn't need to be a reference it should be passed by value
                    self.realm.environment.get_current_environment().clone(),
                    FunctionBody::Ordinary(*expr.clone()),
                    ThisMode::NonLexical,
                );

                let mut new_func = Object::function();
                new_func.set_call(func);
                let val = Value::from(new_func);
                val.set_field_slice("length", Value::from(args.len()));

                if let Some(name) = name {
                    val.set_field_slice("name", Value::from(name.clone()));
                }

                Ok(val)
            }
            Node::ArrowFunctionDecl(ref args, ref expr) => {
                // Todo: Function.prototype doesn't exist yet, so the prototype right now is the Object.prototype
                // let proto = &self
                //     .realm
                //     .environment
                //     .get_global_object()
                //     .expect("Could not get the global object")
                //     .get_field_slice("Object")
                //     .get_field_slice("Prototype");

                let func = FunctionObject::create_ordinary(
                    args.clone(), // TODO: args shouldn't need to be a reference it should be passed by value
                    self.realm.environment.get_current_environment().clone(),
                    FunctionBody::Ordinary(*expr.clone()),
                    ThisMode::Lexical,
                );

                let mut new_func = Object::function();
                new_func.set_call(func);
                let val = Value::from(new_func);
                val.set_field_slice("length", Value::from(args.len()));

                Ok(val)
            }
            Node::BinOp(BinOp::Num(ref op), ref a, ref b) => {
                let v_a = self.run(a)?;
                let v_b = self.run(b)?;
                Ok(match *op {
                    NumOp::Add => v_a + v_b,
                    NumOp::Sub => v_a - v_b,
                    NumOp::Mul => v_a * v_b,
                    NumOp::Exp => v_a.as_num_to_power(v_b),
                    NumOp::Div => v_a / v_b,
                    NumOp::Mod => v_a % v_b,
                })
            }
            Node::UnaryOp(ref op, ref a) => {
                let v_a = self.run(a)?;
                Ok(match *op {
                    UnaryOp::Minus => Value::from(-v_a.to_number()),
                    UnaryOp::Plus => Value::from(v_a.to_number()),
                    UnaryOp::IncrementPost => {
                        let ret = v_a.clone();
                        self.set_value(a, Value::from(v_a.to_number() + 1.0))?;
                        ret
                    }
                    UnaryOp::IncrementPre => {
                        self.set_value(a, Value::from(v_a.to_number() + 1.0))?
                    }
                    UnaryOp::DecrementPost => {
                        let ret = v_a.clone();
                        self.set_value(a, Value::from(v_a.to_number() - 1.0))?;
                        ret
                    }
                    UnaryOp::DecrementPre => {
                        self.set_value(a, Value::from(v_a.to_number() - 1.0))?
                    }
                    UnaryOp::Not => !v_a,
                    UnaryOp::Tilde => {
                        let num_v_a = v_a.to_number();
                        // NOTE: possible UB: https://github.com/rust-lang/rust/issues/10184
                        Value::from(if num_v_a.is_nan() {
                            -1
                        } else {
                            !(num_v_a as i32)
                        })
                    }
                    UnaryOp::Void => Value::undefined(),
                    UnaryOp::Delete => match a.deref() {
                        Node::GetConstField(ref obj, ref field) => {
                            Value::boolean(self.run(obj)?.remove_property(field))
                        }
                        Node::GetField(ref obj, ref field) => Value::boolean(
                            self.run(obj)?
                                .remove_property(&self.run(field)?.to_string()),
                        ),
                        Node::Local(_) => Value::boolean(false),
                        Node::ArrayDecl(_)
                        | Node::Block(_)
                        | Node::Const(_)
                        | Node::FunctionDecl(_, _, _)
                        | Node::FunctionExpr(_, _, _)
                        | Node::New(_)
                        | Node::Object(_)
                        | Node::TypeOf(_)
                        | Node::UnaryOp(_, _) => Value::boolean(true),
                        _ => panic!("SyntaxError: wrong delete argument {}", node),
                    },
                    _ => unimplemented!(),
                })
            }
            Node::BinOp(BinOp::Bit(ref op), ref a, ref b) => {
                let v_a = self.run(a)?;
                let v_b = self.run(b)?;
                Ok(match *op {
                    BitOp::And => v_a & v_b,
                    BitOp::Or => v_a | v_b,
                    BitOp::Xor => v_a ^ v_b,
                    BitOp::Shl => v_a << v_b,
                    BitOp::Shr => v_a >> v_b,
                    // TODO Fix
                    BitOp::UShr => v_a >> v_b,
                })
            }
            Node::BinOp(BinOp::Comp(ref op), ref a, ref b) => {
                let mut v_r_a = self.run(a)?;
                let mut v_r_b = self.run(b)?;
                let mut v_a = v_r_a.borrow_mut();
                let mut v_b = v_r_b.borrow_mut();
                Ok(Value::from(match *op {
                    CompOp::Equal => v_r_a.equals(v_b, self),
                    CompOp::NotEqual => !v_r_a.equals(v_b, self),
                    CompOp::StrictEqual => v_r_a.strict_equals(v_b),
                    CompOp::StrictNotEqual => !v_r_a.strict_equals(v_b),
                    CompOp::GreaterThan => v_a.to_number() > v_b.to_number(),
                    CompOp::GreaterThanOrEqual => v_a.to_number() >= v_b.to_number(),
                    CompOp::LessThan => v_a.to_number() < v_b.to_number(),
                    CompOp::LessThanOrEqual => v_a.to_number() <= v_b.to_number(),
                    CompOp::In => {
                        if !v_b.is_object() {
                            panic!("TypeError: {} is not an Object.", v_b);
                        }
                        let key = self.to_property_key(&mut v_a);
                        self.has_property(&mut v_b, &key)
                    }
                }))
            }
            Node::BinOp(BinOp::Log(ref op), ref a, ref b) => {
                // turn a `Value` into a `bool`
                let to_bool = |value| bool::from(&value);
                Ok(match *op {
                    LogOp::And => Value::from(to_bool(self.run(a)?) && to_bool(self.run(b)?)),
                    LogOp::Or => Value::from(to_bool(self.run(a)?) || to_bool(self.run(b)?)),
                })
            }
            Node::BinOp(BinOp::Assign(ref op), ref a, ref b) => match a.deref() {
                Node::Local(ref name) => {
                    let v_a = self.realm.environment.get_binding_value(&name);
                    let v_b = self.run(b)?;
                    let value = exec_assign_op(op, v_a, v_b);
                    self.realm
                        .environment
                        .set_mutable_binding(&name, value.clone(), true);
                    Ok(value)
                }
                Node::GetConstField(ref obj, ref field) => {
                    let v_r_a = self.run(obj)?;
                    let v_a = v_r_a.get_field_slice(field);
                    let v_b = self.run(b)?;
                    let value = exec_assign_op(op, v_a, v_b);
                    v_r_a
                        .borrow()
                        .set_field_slice(&field.clone(), value.clone());
                    Ok(value)
                }
                _ => Ok(Value::undefined()),
            },
            Node::New(ref call) => {
                let (callee, args) = match call.as_ref() {
                    Node::Call(callee, args) => (callee, args),
                    _ => unreachable!("Node::New(ref call): 'call' must only be Node::Call type."),
                };

                let func_object = self.run(callee)?;
                let mut v_args = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    v_args.push(self.run(arg)?);
                }
                let mut this = Value::new_object(None);
                // Create a blank object, then set its __proto__ property to the [Constructor].prototype
                this.borrow().set_internal_slot(
                    INSTANCE_PROTOTYPE,
                    func_object.borrow().get_field_slice(PROTOTYPE),
                );

                match *(func_object.borrow()).deref() {
                    ValueData::Object(ref o) => (*o.deref().clone().borrow_mut())
                        .construct
                        .as_ref()
                        .unwrap()
                        .construct(&mut func_object.clone(), &v_args, self, &mut this),
                    _ => Ok(Value::undefined()),
                }
            }
            Node::Return(ref ret) => {
                let result = match *ret {
                    Some(ref v) => self.run(v),
                    None => Ok(Value::undefined()),
                };
                // Set flag for return
                self.is_return = true;
                result
            }
            Node::Throw(ref ex) => Err(self.run(ex)?),
            Node::Assign(ref ref_e, ref val_e) => {
                let val = self.run(val_e)?;
                match ref_e.deref() {
                    Node::Local(ref name) => {
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
                    Node::GetConstField(ref obj, ref field) => {
                        let val_obj = self.run(obj)?;
                        val_obj
                            .borrow()
                            .set_field_slice(&field.clone(), val.clone());
                    }
                    Node::GetField(ref obj, ref field) => {
                        let val_obj = self.run(obj)?;
                        let val_field = self.run(field)?;
                        val_obj.borrow().set_field(val_field, val.clone());
                    }
                    _ => (),
                }
                Ok(val)
            }
            Node::VarDecl(ref vars) => {
                for var in vars.iter() {
                    let (name, value) = var.clone();
                    let val = match value {
                        Some(v) => self.run(&v)?,
                        None => Value::undefined(),
                    };
                    self.realm.environment.create_mutable_binding(
                        name.clone(),
                        false,
                        VariableScope::Function,
                    );
                    self.realm.environment.initialize_binding(&name, val);
                }
                Ok(Value::undefined())
            }
            Node::LetDecl(ref vars) => {
                for var in vars.iter() {
                    let (name, value) = var.clone();
                    let val = match value {
                        Some(v) => self.run(&v)?,
                        None => Value::undefined(),
                    };
                    self.realm.environment.create_mutable_binding(
                        name.clone(),
                        false,
                        VariableScope::Block,
                    );
                    self.realm.environment.initialize_binding(&name, val);
                }
                Ok(Value::undefined())
            }
            Node::ConstDecl(ref vars) => {
                for (name, value) in vars.iter() {
                    self.realm.environment.create_immutable_binding(
                        name.clone(),
                        false,
                        VariableScope::Block,
                    );
                    let val = self.run(&value)?;
                    self.realm.environment.initialize_binding(&name, val);
                }
                Ok(Value::undefined())
            }
            Node::TypeOf(ref val_e) => {
                let val = self.run(val_e)?;
                Ok(Value::from(match *val {
                    ValueData::Undefined => "undefined",
                    ValueData::Symbol(_) => "symbol",
                    ValueData::Null => "object",
                    ValueData::Boolean(_) => "boolean",
                    ValueData::Rational(_) | ValueData::Integer(_) => "number",
                    ValueData::String(_) => "string",
                    ValueData::Object(ref o) => {
                        if o.deref().borrow().is_callable() {
                            "function"
                        } else {
                            "object"
                        }
                    }
                }))
            }
            Node::StatementList(ref list) => {
                {
                    let env = &mut self.realm.environment;
                    env.push(new_declarative_environment(Some(
                        env.get_current_environment_ref().clone(),
                    )));
                }

                let mut obj = Value::null();
                for (i, item) in list.iter().enumerate() {
                    let val = self.run(item)?;
                    // early return
                    if self.is_return {
                        obj = val;
                        break;
                    }
                    if i + 1 == list.len() {
                        obj = val;
                    }
                }

                // pop the block env
                let _ = self.realm.environment.pop();

                Ok(obj)
            }
            Node::Spread(ref node) => {
                // TODO: for now we can do nothing but return the value as-is
                self.run(node)
            }
            ref i => unimplemented!("{}", i),
        }
    }
}

impl Interpreter {
    /// Get the Interpreter's realm
    pub(crate) fn get_realm(&self) -> &Realm {
        &self.realm
    }

    /// https://tc39.es/ecma262/#sec-call
    pub(crate) fn call(
        &mut self,
        f: &Value,
        this: &mut Value,
        arguments_list: &[Value],
    ) -> ResultValue {
        // All functions should be objects, and eventually will be.
        // During this transition call will support both native functions and function objects
        match (*f).deref() {
            ValueData::Object(ref obj) => match (*obj).deref().borrow().call {
                Some(ref func) => func.call(&mut f.clone(), arguments_list, self, this),
                None => panic!("Expected function"),
            },
            _ => Err(Value::undefined()),
        }
    }

    /// https://tc39.es/ecma262/#sec-ordinarytoprimitive
    fn ordinary_to_primitive(&mut self, o: &mut Value, hint: &str) -> Value {
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
                let result = self.call(&method, o, &[]);
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

        Value::undefined()
    }

    /// The abstract operation ToPrimitive takes an input argument and an optional argument PreferredType.
    /// https://tc39.es/ecma262/#sec-toprimitive
    #[allow(clippy::wrong_self_convention)]
    pub fn to_primitive(&mut self, input: &mut Value, preferred_type: Option<&str>) -> Value {
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

                self.ordinary_to_primitive(input, hint)
            }
            _ => input.clone(),
        }
    }
    /// to_string() converts a value into a String
    /// https://tc39.es/ecma262/#sec-tostring
    #[allow(clippy::wrong_self_convention)]
    pub fn to_string(&mut self, value: &Value) -> Value {
        match *value.deref().borrow() {
            ValueData::Undefined => Value::from("undefined"),
            ValueData::Null => Value::from("null"),
            ValueData::Boolean(ref boolean) => Value::from(boolean.to_string()),
            ValueData::Rational(ref num) => Value::from(num.to_string()),
            ValueData::Integer(ref num) => Value::from(num.to_string()),
            ValueData::String(ref string) => Value::from(string.clone()),
            ValueData::Object(_) => {
                let prim_value = self.to_primitive(&mut (value.clone()), Some("string"));
                self.to_string(&prim_value)
            }
            _ => Value::from("function(){...}"),
        }
    }

    /// The abstract operation ToPropertyKey takes argument argument. It converts argument to a value that can be used as a property key.
    /// https://tc39.es/ecma262/#sec-topropertykey
    #[allow(clippy::wrong_self_convention)]
    pub fn to_property_key(&mut self, value: &mut Value) -> Value {
        let key = self.to_primitive(value, Some("string"));
        if key.is_symbol() {
            key
        } else {
            self.to_string(&key)
        }
    }

    /// https://tc39.es/ecma262/#sec-hasproperty
    pub fn has_property(&self, obj: &mut Value, key: &Value) -> bool {
        if let Some(obj) = obj.as_object() {
            if !Property::is_property_key(key) {
                false
            } else {
                obj.has_property(key)
            }
        } else {
            false
        }
    }

    /// The abstract operation ToObject converts argument to a value of type Object
    /// https://tc39.es/ecma262/#sec-toobject
    #[allow(clippy::wrong_self_convention)]
    pub fn to_object(&mut self, value: &Value) -> ResultValue {
        match *value.deref().borrow() {
            ValueData::Undefined | ValueData::Integer(_) | ValueData::Null => {
                Err(Value::undefined())
            }
            ValueData::Boolean(_) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("Boolean")
                    .get_field_slice(PROTOTYPE);

                let bool_obj = Value::new_object_from_prototype(proto, ObjectKind::Boolean);
                bool_obj.set_internal_slot("BooleanData", value.clone());
                Ok(bool_obj)
            }
            ValueData::Rational(_) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("Number")
                    .get_field_slice(PROTOTYPE);
                let number_obj = Value::new_object_from_prototype(proto, ObjectKind::Number);
                number_obj.set_internal_slot("NumberData", value.clone());
                Ok(number_obj)
            }
            ValueData::String(_) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("String")
                    .get_field_slice(PROTOTYPE);
                let string_obj = Value::new_object_from_prototype(proto, ObjectKind::String);
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
            ValueData::Rational(ref num) => num.to_string(),
            ValueData::Integer(ref num) => num.to_string(),
            ValueData::String(ref string) => string.clone(),
            ValueData::Object(_) => {
                let prim_value = self.to_primitive(&mut (value.clone()), Some("string"));
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
            ValueData::Rational(num) => num,
            ValueData::Integer(num) => f64::from(num),
            ValueData::String(ref string) => string.parse::<f64>().unwrap(),
            ValueData::Object(_) => {
                let prim_value = self.to_primitive(&mut (value.clone()), Some("number"));
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
    fn extract_array_properties(&mut self, value: &Value) -> Result<Vec<Value>, ()> {
        if let ValueData::Object(ref x) = *value.deref().borrow() {
            // Check if object is array
            if x.deref().borrow().kind == ObjectKind::Array {
                let length: i32 =
                    self.value_to_rust_number(&value.get_field_slice("length")) as i32;
                let values: Vec<Value> = (0..length)
                    .map(|idx| value.get_field_slice(&idx.to_string()))
                    .collect();
                return Ok(values);
            }

            return Err(());
        }

        Err(())
    }

    fn set_value(&mut self, node: &Node, value: Value) -> ResultValue {
        match node {
            Node::Local(ref name) => {
                self.realm
                    .environment
                    .set_mutable_binding(name, value.clone(), true);
                Ok(value)
            }
            Node::GetConstField(ref obj, ref field) => {
                Ok(self.run(obj)?.set_field_slice(field, value))
            }
            Node::GetField(ref obj, ref field) => {
                Ok(self.run(obj)?.set_field(self.run(field)?, value))
            }
            _ => panic!("TypeError: invalid assignment to {}", node),
        }
    }
}
