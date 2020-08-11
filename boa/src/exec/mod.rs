//! Execution of the AST, this is where the interpreter actually runs

mod array;
mod block;
mod break_node;
mod call;
mod conditional;
mod declaration;
mod exception;
mod field;
mod identifier;
mod iteration;
mod new;
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
    builtins,
    builtins::{
        function::{Function as FunctionObject, FunctionBody, ThisMode},
        number::{f64_to_int32, f64_to_uint32},
        object::{Object, ObjectData, PROTOTYPE},
        property::PropertyKey,
        value::{PreferredType, RcBigInt, RcString, ResultValue, Type, Value},
        BigInt, Console, Number,
    },
    realm::Realm,
    syntax::ast::{
        constant::Const,
        node::{FormalParameter, Node, StatementList},
    },
    BoaProfiler,
};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::ops::Deref;

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

/// A Javascript intepreter
#[derive(Debug)]
pub struct Interpreter {
    /// the current state of the interpreter.
    state: InterpreterState,

    /// realm holds both the global object and the environment
    pub realm: Realm,

    /// This is for generating an unique internal `Symbol` hash.
    symbol_count: u32,

    /// console object state.
    console: Console,
}

impl Interpreter {
    /// Creates a new interpreter.
    pub fn new(realm: Realm) -> Self {
        let mut interpreter = Self {
            state: InterpreterState::Executing,
            realm,
            symbol_count: 0,
            console: Console::default(),
        };

        // Add new builtIns to Interpreter Realm
        // At a later date this can be removed from here and called explicitly, but for now we almost always want these default builtins
        interpreter.create_intrinsics();

        interpreter
    }

    /// Sets up the default global objects within Global
    fn create_intrinsics(&mut self) {
        let _timer = BoaProfiler::global().start_event("create_intrinsics", "interpreter");
        // Create intrinsics, add global objects here
        builtins::init(self);
    }

    /// Retrieves the `Realm` of this executor.
    #[inline]
    pub(crate) fn realm(&self) -> &Realm {
        &self.realm
    }

    /// Retrieves the `Realm` of this executor as a mutable reference.
    #[inline]
    pub(crate) fn realm_mut(&mut self) -> &mut Realm {
        &mut self.realm
    }

    /// Retrieves the global object of the `Realm` of this executor.
    #[inline]
    pub(crate) fn global(&self) -> &Value {
        &self.realm.global_obj
    }

    /// Generates a new `Symbol` internal hash.
    ///
    /// This currently is an incremented value.
    #[inline]
    pub(crate) fn generate_hash(&mut self) -> u32 {
        let hash = self.symbol_count;
        self.symbol_count += 1;
        hash
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
        let function_prototype = self
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

        let new_func = Object::function(func, function_prototype);

        let val = Value::from(new_func);
        val.set_field(PROTOTYPE, proto);
        val.set_field("length", Value::from(params_len));

        val
    }

    /// <https://tc39.es/ecma262/#sec-call>
    pub(crate) fn call(
        &mut self,
        f: &Value,
        this: &Value,
        arguments_list: &[Value],
    ) -> ResultValue {
        match *f {
            Value::Object(ref obj) => {
                let obj = obj.borrow();
                if let ObjectData::Function(ref func) = obj.data {
                    return func.call(f.clone(), this, arguments_list, self);
                }
                self.throw_type_error("not a function")
            }
            _ => self.throw_type_error("not a function"),
        }
    }

    /// Converts a value into a rust heap allocated string.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_string(&mut self, value: &Value) -> Result<RcString, Value> {
        match value {
            Value::Null => Ok(RcString::from("null")),
            Value::Undefined => Ok(RcString::from("undefined".to_owned())),
            Value::Boolean(boolean) => Ok(RcString::from(boolean.to_string())),
            Value::Rational(rational) => Ok(RcString::from(Number::to_native_string(*rational))),
            Value::Integer(integer) => Ok(RcString::from(integer.to_string())),
            Value::String(string) => Ok(string.clone()),
            Value::Symbol(_) => Err(self.construct_type_error("can't convert symbol to string")),
            Value::BigInt(ref bigint) => Ok(RcString::from(bigint.to_string())),
            Value::Object(_) => {
                let primitive = value.to_primitive(self, PreferredType::String)?;
                self.to_string(&primitive)
            }
        }
    }

    /// Helper function.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_bigint(&mut self, value: &Value) -> Result<RcBigInt, Value> {
        match value {
            Value::Null => Err(self.construct_type_error("cannot convert null to a BigInt")),
            Value::Undefined => {
                Err(self.construct_type_error("cannot convert undefined to a BigInt"))
            }
            Value::String(ref string) => Ok(RcBigInt::from(BigInt::from_string(string, self)?)),
            Value::Boolean(true) => Ok(RcBigInt::from(BigInt::from(1))),
            Value::Boolean(false) => Ok(RcBigInt::from(BigInt::from(0))),
            Value::Integer(num) => Ok(RcBigInt::from(BigInt::from(*num))),
            Value::Rational(num) => {
                if let Ok(bigint) = BigInt::try_from(*num) {
                    return Ok(RcBigInt::from(bigint));
                }
                Err(self.construct_type_error(format!(
                    "The number {} cannot be converted to a BigInt because it is not an integer",
                    num
                )))
            }
            Value::BigInt(b) => Ok(b.clone()),
            Value::Object(_) => {
                let primitive = value.to_primitive(self, PreferredType::Number)?;
                self.to_bigint(&primitive)
            }
            Value::Symbol(_) => Err(self.construct_type_error("cannot convert Symbol to a BigInt")),
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

        if integer_index < 0.0 {
            return Err(self.construct_range_error("Integer index must be >= 0"));
        }

        if integer_index > Number::MAX_SAFE_INTEGER {
            return Err(self.construct_range_error("Integer index must be less than 2**(53) - 1"));
        }

        Ok(integer_index as usize)
    }

    /// Converts a value to an integral Number value.
    ///
    /// See: https://tc39.es/ecma262/#sec-tointeger
    #[allow(clippy::wrong_self_convention)]
    pub fn to_integer(&mut self, value: &Value) -> Result<f64, Value> {
        // 1. Let number be ? ToNumber(argument).
        let number = self.to_number(value)?;

        // 2. If number is +∞ or -∞, return number.
        if !number.is_finite() {
            // 3. If number is NaN, +0, or -0, return +0.
            if number.is_nan() {
                return Ok(0.0);
            }
            return Ok(number);
        }

        // 4. Let integer be the Number value that is the same sign as number and whose magnitude is floor(abs(number)).
        // 5. If integer is -0, return +0.
        // 6. Return integer.
        Ok(number.trunc() + 0.0) // We add 0.0 to convert -0.0 to +0.0
    }

    /// Converts a value to an integral 32 bit signed integer.
    ///
    /// See: https://tc39.es/ecma262/#sec-toint32
    #[allow(clippy::wrong_self_convention)]
    pub fn to_int32(&mut self, value: &Value) -> Result<i32, Value> {
        // This is the fast path, if the value is Integer we can just return it.
        if let Value::Integer(number) = *value {
            return Ok(number);
        }
        let number = self.to_number(value)?;

        Ok(f64_to_int32(number))
    }

    /// Converts a value to an integral 32 bit unsigned integer.
    ///
    /// See: https://tc39.es/ecma262/#sec-toint32
    #[allow(clippy::wrong_self_convention)]
    pub fn to_uint32(&mut self, value: &Value) -> Result<u32, Value> {
        // This is the fast path, if the value is Integer we can just return it.
        if let Value::Integer(number) = *value {
            return Ok(number as u32);
        }
        let number = self.to_number(value)?;

        Ok(f64_to_uint32(number))
    }

    /// Converts argument to an integer suitable for use as the length of an array-like object.
    ///
    /// See: https://tc39.es/ecma262/#sec-tolength
    #[allow(clippy::wrong_self_convention)]
    pub fn to_length(&mut self, value: &Value) -> Result<usize, Value> {
        // 1. Let len be ? ToInteger(argument).
        let len = self.to_integer(value)?;

        // 2. If len ≤ +0, return +0.
        if len < 0.0 {
            return Ok(0);
        }

        // 3. Return min(len, 2^53 - 1).
        Ok(len.min(Number::MAX_SAFE_INTEGER) as usize)
    }

    /// Converts a value to a double precision floating point.
    ///
    /// See: https://tc39.es/ecma262/#sec-tonumber
    #[allow(clippy::wrong_self_convention)]
    pub fn to_number(&mut self, value: &Value) -> Result<f64, Value> {
        match *value {
            Value::Null => Ok(0.0),
            Value::Undefined => Ok(f64::NAN),
            Value::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
            // TODO: this is probably not 100% correct, see https://tc39.es/ecma262/#sec-tonumber-applied-to-the-string-type
            Value::String(ref string) => Ok(string.parse().unwrap_or(f64::NAN)),
            Value::Rational(number) => Ok(number),
            Value::Integer(integer) => Ok(f64::from(integer)),
            Value::Symbol(_) => Err(self.construct_type_error("argument must not be a symbol")),
            Value::BigInt(_) => Err(self.construct_type_error("argument must not be a bigint")),
            Value::Object(_) => {
                let primitive = value.to_primitive(self, PreferredType::Number)?;
                self.to_number(&primitive)
            }
        }
    }

    /// It returns value converted to a numeric value of type Number or BigInt.
    ///
    /// See: https://tc39.es/ecma262/#sec-tonumeric
    #[allow(clippy::wrong_self_convention)]
    pub fn to_numeric(&mut self, value: &Value) -> ResultValue {
        let primitive = value.to_primitive(self, PreferredType::Number)?;
        if primitive.is_bigint() {
            return Ok(primitive);
        }
        Ok(Value::from(self.to_number(&primitive)?))
    }

    /// This is a more specialized version of `to_numeric`.
    ///
    /// It returns value converted to a numeric value of type `Number`.
    ///
    /// See: https://tc39.es/ecma262/#sec-tonumeric
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_numeric_number(&mut self, value: &Value) -> Result<f64, Value> {
        let primitive = value.to_primitive(self, PreferredType::Number)?;
        if let Some(ref bigint) = primitive.as_bigint() {
            return Ok(bigint.to_f64());
        }
        Ok(self.to_number(&primitive)?)
    }

    /// Converts an array object into a rust vector of values.
    ///
    /// This is useful for the spread operator, for any other object an `Err` is returned
    pub(crate) fn extract_array_properties(&mut self, value: &Value) -> Result<Vec<Value>, ()> {
        if let Value::Object(ref x) = value {
            // Check if object is array
            if let ObjectData::Array = x.deref().borrow().data {
                let length = i32::from(&value.get_field("length"));
                let values = (0..length)
                    .map(|idx| value.get_field(idx.to_string()))
                    .collect();
                return Ok(values);
            }
            // Check if object is a Map
            else if let ObjectData::Map(ref map) = x.deref().borrow().data {
                let values = map
                    .borrow()
                    .iter()
                    .map(|(key, value)| {
                        // Construct a new array containing the key-value pair
                        let array = Value::new_object(Some(
                            &self
                                .realm()
                                .environment
                                .get_global_object()
                                .expect("Could not get global object"),
                        ));
                        array.set_data(ObjectData::Array);
                        array.as_object_mut().expect("object").set_prototype(
                            self.realm()
                                .environment
                                .get_binding_value("Array")
                                .expect("Array was not initialized")
                                .borrow()
                                .get_field(PROTOTYPE),
                        );
                        array.set_field("0", key);
                        array.set_field("1", value);
                        array.set_field("length", Value::from(2));
                        array
                    })
                    .collect();
                return Ok(values);
            }

            return Err(());
        }

        Err(())
    }

    /// Converts an object to a primitive.
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinarytoprimitive
    pub(crate) fn ordinary_to_primitive(&mut self, o: &Value, hint: PreferredType) -> ResultValue {
        // 1. Assert: Type(O) is Object.
        debug_assert!(o.get_type() == Type::Object);
        // 2. Assert: Type(hint) is String and its value is either "string" or "number".
        debug_assert!(hint == PreferredType::String || hint == PreferredType::Number);

        // 3. If hint is "string", then
        //    a. Let methodNames be « "toString", "valueOf" ».
        // 4. Else,
        //    a. Let methodNames be « "valueOf", "toString" ».
        let method_names = if hint == PreferredType::String {
            ["toString", "valueOf"]
        } else {
            ["valueOf", "toString"]
        };

        // 5. For each name in methodNames in List order, do
        for name in &method_names {
            // a. Let method be ? Get(O, name).
            let method: Value = o.get_field(*name);
            // b. If IsCallable(method) is true, then
            if method.is_function() {
                // i. Let result be ? Call(method, O).
                let result = self.call(&method, &o, &[])?;
                // ii. If Type(result) is not Object, return result.
                if !result.is_object() {
                    return Ok(result);
                }
            }
        }

        // 6. Throw a TypeError exception.
        self.throw_type_error("cannot convert object to primitive value")
    }

    /// The abstract operation ToPropertyKey takes argument argument. It converts argument to a value that can be used as a property key.
    ///
    /// https://tc39.es/ecma262/#sec-topropertykey
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_property_key(&mut self, value: &Value) -> Result<PropertyKey, Value> {
        let key = value.to_primitive(self, PreferredType::String)?;
        if let Value::Symbol(ref symbol) = key {
            Ok(PropertyKey::from(symbol.clone()))
        } else {
            let string = self.to_string(&key)?;
            Ok(PropertyKey::from(string))
        }
    }

    /// https://tc39.es/ecma262/#sec-hasproperty
    pub(crate) fn has_property(&self, obj: &Value, key: &PropertyKey) -> bool {
        if let Some(obj) = obj.as_object() {
            obj.has_property(key)
        } else {
            false
        }
    }

    /// The abstract operation ToObject converts argument to a value of type Object
    /// https://tc39.es/ecma262/#sec-toobject
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_object(&mut self, value: &Value) -> ResultValue {
        match value {
            Value::Undefined | Value::Null => {
                self.throw_type_error("cannot convert 'null' or 'undefined' to object")
            }
            Value::Boolean(boolean) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("Boolean")
                    .expect("Boolean was not initialized")
                    .get_field(PROTOTYPE);

                Ok(Value::new_object_from_prototype(
                    proto,
                    ObjectData::Boolean(*boolean),
                ))
            }
            Value::Integer(integer) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("Number")
                    .expect("Number was not initialized")
                    .get_field(PROTOTYPE);
                Ok(Value::new_object_from_prototype(
                    proto,
                    ObjectData::Number(f64::from(*integer)),
                ))
            }
            Value::Rational(rational) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("Number")
                    .expect("Number was not initialized")
                    .get_field(PROTOTYPE);

                Ok(Value::new_object_from_prototype(
                    proto,
                    ObjectData::Number(*rational),
                ))
            }
            Value::String(ref string) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("String")
                    .expect("String was not initialized")
                    .get_field(PROTOTYPE);

                Ok(Value::new_object_from_prototype(
                    proto,
                    ObjectData::String(string.clone()),
                ))
            }
            Value::Symbol(ref symbol) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("Symbol")
                    .expect("Symbol was not initialized")
                    .get_field(PROTOTYPE);

                Ok(Value::new_object_from_prototype(
                    proto,
                    ObjectData::Symbol(symbol.clone()),
                ))
            }
            Value::BigInt(ref bigint) => {
                let proto = self
                    .realm
                    .environment
                    .get_binding_value("BigInt")
                    .expect("BigInt was not initialized")
                    .get_field(PROTOTYPE);
                let bigint_obj =
                    Value::new_object_from_prototype(proto, ObjectData::BigInt(bigint.clone()));
                Ok(bigint_obj)
            }
            Value::Object(_) => Ok(value.clone()),
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
            Node::GetField(ref get_field) => {
                let field = get_field.field().run(self)?;
                let key = self.to_property_key(&field)?;
                Ok(get_field.obj().run(self)?.set_field(key, value))
            }
            _ => panic!("TypeError: invalid assignment to {}", node),
        }
    }

    #[inline]
    pub(crate) fn set_current_state(&mut self, new_state: InterpreterState) {
        self.state = new_state
    }

    #[inline]
    pub(crate) fn get_current_state(&self) -> &InterpreterState {
        &self.state
    }

    /// Check if the `Value` can be converted to an `Object`
    ///
    /// The abstract operation `RequireObjectCoercible` takes argument argument.
    /// It throws an error if argument is a value that cannot be converted to an Object using `ToObject`.
    /// It is defined by [Table 15][table]
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [table]: https://tc39.es/ecma262/#table-14
    /// [spec]: https://tc39.es/ecma262/#sec-requireobjectcoercible
    #[inline]
    pub fn require_object_coercible<'a>(&mut self, value: &'a Value) -> Result<&'a Value, Value> {
        if value.is_null_or_undefined() {
            Err(self.construct_type_error("cannot convert null or undefined to Object"))
        } else {
            Ok(value)
        }
    }

    /// A helper function for getting a immutable reference to the `console` object.
    pub(crate) fn console(&self) -> &Console {
        &self.console
    }

    /// A helper function for getting a mutable reference to the `console` object.
    pub(crate) fn console_mut(&mut self) -> &mut Console {
        &mut self.console
    }
}

impl Executable for Node {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let _timer = BoaProfiler::global().start_event("Executable", "exec");
        match *self {
            Node::Const(Const::Null) => Ok(Value::null()),
            Node::Const(Const::Num(num)) => Ok(Value::rational(num)),
            Node::Const(Const::Int(num)) => Ok(Value::integer(num)),
            Node::Const(Const::BigInt(ref num)) => Ok(Value::from(num.clone())),
            // we can't move String from Const into value, because const is a garbage collected value
            // Which means Drop() get's called on Const, but str will be gone at that point.
            // Do Const values need to be garbage collected? We no longer need them once we've generated Values
            Node::Const(Const::String(ref value)) => Ok(Value::string(value.to_string())),
            Node::Const(Const::Bool(value)) => Ok(Value::boolean(value)),
            Node::Block(ref block) => block.run(interpreter),
            Node::Identifier(ref identifier) => identifier.run(interpreter),
            Node::GetConstField(ref get_const_field_node) => get_const_field_node.run(interpreter),
            Node::GetField(ref get_field) => get_field.run(interpreter),
            Node::Call(ref call) => call.run(interpreter),
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
            Node::FunctionExpr(ref function_expr) => function_expr.run(interpreter),
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
