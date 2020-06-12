//! Execution of the AST, this is where the interpreter actually runs

mod array;
mod block;
mod break_node;
mod conditional;
mod declaration;
mod exception;
mod expression;
mod field;
mod iteration;
mod object;
mod operator;
mod return_smt;
mod spread;
mod statement_list;
mod switch;
#[cfg(test)]
mod tests;
mod throw;
mod try_node;

use crate::{
    builtins::{
        function::{Function as FunctionObject, FunctionBody, ThisMode},
        object::{
            internal_methods_trait::ObjectInternalMethods, Object, ObjectKind, INSTANCE_PROTOTYPE,
            PROTOTYPE,
        },
        property::Property,
        value::{ResultValue, Type, Value, ValueData},
        BigInt, Number,
    },
    realm::Realm,
    syntax::ast::{
        constant::Const,
        node::{FormalParameter, Node, StatementList},
    },
    BoaProfiler,
};
use std::convert::TryFrom;
use std::{borrow::Borrow, ops::Deref};

pub trait Executable {
    /// Runs this executable in the given executor.
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum InterpreterState {
    Executing,
    Return,
    Break(Option<String>),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PreferredType {
    String,
    Number,
    Default,
}

/// A Javascript intepreter
#[derive(Debug)]
pub struct Interpreter {
    current_state: InterpreterState,

    /// realm holds both the global object and the environment
    pub realm: Realm,
}

impl Interpreter {
    /// Creates a new interpreter.
    pub fn new(realm: Realm) -> Self {
        Self {
            current_state: InterpreterState::Executing,
            realm,
        }
    }

    /// Retrieves the `Realm` of this executor.
    pub(crate) fn realm(&self) -> &Realm {
        &self.realm
    }

    /// Retrieves the `Realm` of this executor as a mutable reference.
    pub(crate) fn realm_mut(&mut self) -> &mut Realm {
        &mut self.realm
    }

    /// Utility to create a function Value for Function Declarations, Arrow Functions or Function Expressions
    pub(crate) fn create_function<P, B>(
        &mut self,
        params: P,
        body: B,
        this_mode: ThisMode,
        constructable: bool,
        callable: bool,
    ) -> Value
    where
        P: Into<Box<[FormalParameter]>>,
        B: Into<StatementList>,
    {
        let function_prototype = &self
            .realm
            .environment
            .get_global_object()
            .expect("Could not get the global object")
            .get_field("Function")
            .get_field(PROTOTYPE);

        // Every new function has a prototype property pre-made
        let global_val = &self
            .realm
            .environment
            .get_global_object()
            .expect("Could not get the global object");
        let proto = Value::new_object(Some(global_val));

        let params = params.into();
        let params_len = params.len();
        let func = FunctionObject::new(
            params,
            Some(self.realm.environment.get_current_environment().clone()),
            FunctionBody::Ordinary(body.into()),
            this_mode,
            constructable,
            callable,
        );

        let mut new_func = Object::function();
        new_func.set_func(func);
        let val = Value::from(new_func);
        val.set_internal_slot(INSTANCE_PROTOTYPE, function_prototype.clone());
        val.set_field(PROTOTYPE, proto);
        val.set_field("length", Value::from(params_len));

        val
    }

    /// <https://tc39.es/ecma262/#sec-call>
    pub(crate) fn call(
        &mut self,
        f: &Value,
        this: &mut Value,
        arguments_list: &[Value],
    ) -> ResultValue {
        match *f.data() {
            ValueData::Object(ref obj) => {
                let obj = (**obj).borrow();
                let func = obj.func.as_ref().expect("Expected function");
                func.call(&mut f.clone(), arguments_list, self, this)
            }
            _ => Err(Value::undefined()),
        }
    }

    /// Converts a value into a rust heap allocated string.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_string(&mut self, value: &Value) -> Result<String, Value> {
        match value.data() {
            ValueData::Null => Ok("null".to_owned()),
            ValueData::Undefined => Ok("undefined".to_owned()),
            ValueData::Boolean(boolean) => Ok(boolean.to_string()),
            ValueData::Rational(rational) => Ok(Number::to_native_string(*rational)),
            ValueData::Integer(integer) => Ok(integer.to_string()),
            ValueData::String(string) => Ok(string.clone()),
            ValueData::Symbol(_) => {
                self.throw_type_error("can't convert symbol to string")?;
                unreachable!();
            }
            ValueData::BigInt(ref bigint) => Ok(bigint.to_string()),
            ValueData::Object(_) => {
                let primitive = self.to_primitive(&mut value.clone(), PreferredType::String);
                self.to_string(&primitive)
            }
        }
    }

    /// Helper function.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_bigint(&mut self, value: &Value) -> Result<BigInt, Value> {
        match value.data() {
            ValueData::Null => {
                self.throw_type_error("cannot convert null to a BigInt")?;
                unreachable!();
            }
            ValueData::Undefined => {
                self.throw_type_error("cannot convert undefined to a BigInt")?;
                unreachable!();
            }
            ValueData::String(ref string) => Ok(BigInt::from_string(string, self)?),
            ValueData::Boolean(true) => Ok(BigInt::from(1)),
            ValueData::Boolean(false) => Ok(BigInt::from(0)),
            ValueData::Integer(num) => Ok(BigInt::from(*num)),
            ValueData::Rational(num) => {
                if let Ok(bigint) = BigInt::try_from(*num) {
                    return Ok(bigint);
                }
                self.throw_type_error(format!(
                    "The number {} cannot be converted to a BigInt because it is not an integer",
                    num
                ))?;
                unreachable!();
            }
            ValueData::BigInt(b) => Ok(b.clone()),
            ValueData::Object(_) => {
                let primitive = self.to_primitive(&mut value.clone(), PreferredType::Number);
                self.to_bigint(&primitive)
            }
            ValueData::Symbol(_) => {
                self.throw_type_error("cannot convert Symbol to a BigInt")?;
                unreachable!();
            }
        }
    }

    /// Converts a value to a non-negative integer if it is a valid integer index value.
    ///
    /// See: https://tc39.es/ecma262/#sec-toindex
    #[allow(clippy::wrong_self_convention)]
    pub fn to_index(&mut self, value: &Value) -> Result<usize, Value> {
        if value.is_undefined() {
            return Ok(0);
        }

        let integer_index = self.to_integer(value)?;

        if integer_index < 0 {
            self.throw_range_error("Integer index must be >= 0")?;
            unreachable!();
        }

        if integer_index > 2i64.pow(53) - 1 {
            self.throw_range_error("Integer index must be less than 2**(53) - 1")?;
            unreachable!()
        }

        Ok(integer_index as usize)
    }

    /// Converts a value to an integral 64 bit signed integer.
    ///
    /// See: https://tc39.es/ecma262/#sec-tointeger
    #[allow(clippy::wrong_self_convention)]
    pub fn to_integer(&mut self, value: &Value) -> Result<i64, Value> {
        let number = self.to_number(value)?;

        if number.is_nan() {
            return Ok(0);
        }

        Ok(number as i64)
    }

    /// Converts a value to a double precision floating point.
    ///
    /// See: https://tc39.es/ecma262/#sec-tonumber
    #[allow(clippy::wrong_self_convention)]
    pub fn to_number(&mut self, value: &Value) -> Result<f64, Value> {
        match *value.deref().borrow() {
            ValueData::Null => Ok(0.0),
            ValueData::Undefined => Ok(f64::NAN),
            ValueData::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
            ValueData::String(ref string) => match string.parse::<f64>() {
                Ok(number) => Ok(number),
                Err(_) => Ok(0.0),
            }, // this is probably not 100% correct, see https://tc39.es/ecma262/#sec-tonumber-applied-to-the-string-type
            ValueData::Rational(number) => Ok(number),
            ValueData::Integer(integer) => Ok(f64::from(integer)),
            ValueData::Symbol(_) => {
                self.throw_type_error("argument must not be a symbol")?;
                unreachable!()
            }
            ValueData::BigInt(_) => {
                self.throw_type_error("argument must not be a bigint")?;
                unreachable!()
            }
            ValueData::Object(_) => {
                let prim_value = self.to_primitive(&mut (value.clone()), PreferredType::Number);
                self.to_number(&prim_value)
            }
        }
    }

    /// Converts an array object into a rust vector of values.
    ///
    /// This is useful for the spread operator, for any other object an `Err` is returned
    pub(crate) fn extract_array_properties(&mut self, value: &Value) -> Result<Vec<Value>, ()> {
        if let ValueData::Object(ref x) = *value.deref().borrow() {
            // Check if object is array
            if x.deref().borrow().kind == ObjectKind::Array {
                let length: i32 = self.value_to_rust_number(&value.get_field("length")) as i32;
                let values: Vec<Value> = (0..length)
                    .map(|idx| value.get_field(idx.to_string()))
                    .collect();
                return Ok(values);
            }

            return Err(());
        }

        Err(())
    }

    /// <https://tc39.es/ecma262/#sec-ordinarytoprimitive>
    pub(crate) fn ordinary_to_primitive(&mut self, o: &mut Value, hint: PreferredType) -> Value {
        debug_assert!(o.get_type() == Type::Object);
        debug_assert!(hint == PreferredType::String || hint == PreferredType::Number);
        let method_names: Vec<&str> = if hint == PreferredType::String {
            vec!["toString", "valueOf"]
        } else {
            vec!["valueOf", "toString"]
        };
        for name in method_names.iter() {
            let method: Value = o.get_field(*name);
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
    /// <https://tc39.es/ecma262/#sec-toprimitive>
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_primitive(
        &mut self,
        input: &mut Value,
        preferred_type: PreferredType,
    ) -> Value {
        let mut hint: PreferredType;
        match (*input).deref() {
            ValueData::Object(_) => {
                hint = preferred_type;

                // Skip d, e we don't support Symbols yet
                // TODO: add when symbols are supported
                if hint == PreferredType::Default {
                    hint = PreferredType::Number;
                };

                self.ordinary_to_primitive(input, hint)
            }
            _ => input.clone(),
        }
    }

    /// The abstract operation ToPropertyKey takes argument argument. It converts argument to a value that can be used as a property key.
    ///
    /// https://tc39.es/ecma262/#sec-topropertykey
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_property_key(&mut self, value: &mut Value) -> ResultValue {
        let key = self.to_primitive(value, PreferredType::String);
        if key.is_symbol() {
            Ok(key)
        } else {
            self.to_string(&key).map(Value::from)
        }
    }

    /// https://tc39.es/ecma262/#sec-hasproperty
    pub(crate) fn has_property(&self, obj: &mut Value, key: &Value) -> bool {
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
    pub(crate) fn to_object(&mut self, value: &Value) -> ResultValue {
        match value.data() {
            ValueData::Undefined | ValueData::Null => Err(Value::undefined()),
            ValueData::Integer(_) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("Number")
                    .get_field(PROTOTYPE);
                let number_obj = Value::new_object_from_prototype(proto, ObjectKind::Number);
                number_obj.set_internal_slot("NumberData", value.clone());
                Ok(number_obj)
            }
            ValueData::Boolean(_) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("Boolean")
                    .get_field(PROTOTYPE);

                let bool_obj = Value::new_object_from_prototype(proto, ObjectKind::Boolean);
                bool_obj.set_internal_slot("BooleanData", value.clone());
                Ok(bool_obj)
            }
            ValueData::Rational(_) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("Number")
                    .get_field(PROTOTYPE);
                let number_obj = Value::new_object_from_prototype(proto, ObjectKind::Number);
                number_obj.set_internal_slot("NumberData", value.clone());
                Ok(number_obj)
            }
            ValueData::String(_) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("String")
                    .get_field(PROTOTYPE);
                let string_obj = Value::new_object_from_prototype(proto, ObjectKind::String);
                string_obj.set_internal_slot("StringData", value.clone());
                Ok(string_obj)
            }
            ValueData::Object(_) | ValueData::Symbol(_) => Ok(value.clone()),
            ValueData::BigInt(_) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("BigInt")
                    .get_field(PROTOTYPE);
                let bigint_obj = Value::new_object_from_prototype(proto, ObjectKind::BigInt);
                bigint_obj.set_internal_slot("BigIntData", value.clone());
                Ok(bigint_obj)
            }
        }
    }

    pub(crate) fn value_to_rust_number(&mut self, value: &Value) -> f64 {
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
            ValueData::BigInt(ref bigint) => bigint.to_f64(),
            ValueData::Object(_) => {
                let prim_value = self.to_primitive(&mut (value.clone()), PreferredType::Number);
                self.to_string(&prim_value)
                    .expect("cannot convert value to string")
                    .parse::<f64>()
                    .expect("cannot parse value to f64")
            }
            ValueData::Undefined => f64::NAN,
            _ => {
                // TODO: Make undefined?
                f64::from(0)
            }
        }
    }

    fn set_value(&mut self, node: &Node, value: Value) -> ResultValue {
        match node {
            Node::Identifier(ref name) => {
                self.realm
                    .environment
                    .set_mutable_binding(name.as_ref(), value.clone(), true);
                Ok(value)
            }
            Node::GetConstField(ref get_const_field_node) => Ok(get_const_field_node
                .obj()
                .run(self)?
                .set_field(get_const_field_node.field(), value)),
            Node::GetField(ref get_field) => Ok(get_field
                .obj()
                .run(self)?
                .set_field(get_field.field().run(self)?, value)),
            _ => panic!("TypeError: invalid assignment to {}", node),
        }
    }

    pub(crate) fn set_current_state(&mut self, new_state: InterpreterState) {
        self.current_state = new_state
    }

    pub(crate) fn get_current_state(&self) -> &InterpreterState {
        &self.current_state
    }
}

impl Executable for Node {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let _timer = BoaProfiler::global().start_event("Executable", "exec");
        match *self {
            Node::Const(Const::Null) => Ok(Value::null()),
            Node::Const(Const::Undefined) => Ok(Value::undefined()),
            Node::Const(Const::Num(num)) => Ok(Value::rational(num)),
            Node::Const(Const::Int(num)) => Ok(Value::integer(num)),
            Node::Const(Const::BigInt(ref num)) => Ok(Value::from(num.clone())),
            // we can't move String from Const into value, because const is a garbage collected value
            // Which means Drop() get's called on Const, but str will be gone at that point.
            // Do Const values need to be garbage collected? We no longer need them once we've generated Values
            Node::Const(Const::String(ref value)) => Ok(Value::string(value.to_string())),
            Node::Const(Const::Bool(value)) => Ok(Value::boolean(value)),
            Node::Block(ref block) => block.run(interpreter),
            Node::Identifier(ref name) => {
                let val = interpreter
                    .realm()
                    .environment
                    .get_binding_value(name.as_ref());
                Ok(val)
            }
            Node::GetConstField(ref get_const_field_node) => get_const_field_node.run(interpreter),
            Node::GetField(ref get_field) => get_field.run(interpreter),
            Node::Call(ref expr) => expr.run(interpreter),
            Node::WhileLoop(ref while_loop) => while_loop.run(interpreter),
            Node::DoWhileLoop(ref do_while) => do_while.run(interpreter),
            Node::ForLoop(ref for_loop) => for_loop.run(interpreter),
            Node::If(ref if_smt) => if_smt.run(interpreter),
            Node::Switch(ref switch) => switch.run(interpreter),
            Node::Object(ref obj) => obj.run(interpreter),
            Node::ArrayDecl(ref arr) => arr.run(interpreter),
            // <https://tc39.es/ecma262/#sec-createdynamicfunction>
            Node::FunctionDecl(ref decl) => decl.run(interpreter),
            // <https://tc39.es/ecma262/#sec-createdynamicfunction>
            Node::FunctionExpr(ref expr) => expr.run(interpreter),
            Node::ArrowFunctionDecl(ref decl) => decl.run(interpreter),
            Node::BinOp(ref op) => op.run(interpreter),
            Node::UnaryOp(ref op) => op.run(interpreter),
            Node::New(ref call) => call.run(interpreter),
            Node::Return(ref ret) => ret.run(interpreter),
            Node::Throw(ref throw) => throw.run(interpreter),
            Node::Assign(ref op) => op.run(interpreter),
            Node::VarDeclList(ref decl) => decl.run(interpreter),
            Node::LetDeclList(ref decl) => decl.run(interpreter),
            Node::ConstDeclList(ref decl) => decl.run(interpreter),
            Node::Spread(ref spread) => spread.run(interpreter),
            Node::This => {
                // Will either return `this` binding or undefined
                Ok(interpreter.realm().environment.get_this_binding())
            }
            Node::Try(ref try_node) => try_node.run(interpreter),
            Node::Break(ref break_node) => break_node.run(interpreter),
            ref i => unimplemented!("{:?}", i),
        }
    }
}
