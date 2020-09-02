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
        function::{Function, FunctionFlags, NativeFunction},
        object::{GcObject, Object, ObjectData, PROTOTYPE},
        property::{Property, PropertyKey},
        value::{PreferredType, RcString, RcSymbol, Type, Value},
        Console, Symbol,
    },
    class::{Class, ClassBuilder},
    realm::Realm,
    syntax::ast::{
        constant::Const,
        node::{FormalParameter, Node, RcStatementList, StatementList},
    },
    BoaProfiler, Result,
};
use std::result::Result as StdResult;

pub trait Executable {
    /// Runs this executable in the given executor.
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum InterpreterState {
    Executing,
    Return,
    Break(Option<String>),
    Continue(Option<String>),
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
    pub fn global(&self) -> &Value {
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
        flags: FunctionFlags,
    ) -> Value
    where
        P: Into<Box<[FormalParameter]>>,
        B: Into<StatementList>,
    {
        let function_prototype = self.global().get_field("Function").get_field(PROTOTYPE);

        // Every new function has a prototype property pre-made
        let proto = Value::new_object(Some(self.global()));

        let params = params.into();
        let params_len = params.len();
        let func = Function::Ordinary {
            flags,
            body: RcStatementList::from(body.into()),
            params,
            environment: self.realm.environment.get_current_environment().clone(),
        };

        let new_func = Object::function(func, function_prototype);

        let val = Value::from(new_func);

        // Set constructor field to the newly created Value (function object)
        proto.set_field("constructor", val.clone());

        val.set_field(PROTOTYPE, proto);
        val.set_field("length", Value::from(params_len));

        val
    }

    /// Utility to create a function Value for Function Declarations, Arrow Functions or Function Expressions
    pub fn create_builtin_function(
        &mut self,
        name: &str,
        length: usize,
        body: NativeFunction,
    ) -> Result<GcObject> {
        let function_prototype = self.global().get_field("Function").get_field(PROTOTYPE);

        // Every new function has a prototype property pre-made
        let proto = Value::new_object(Some(self.global()));
        let mut function = Object::function(
            Function::BuiltIn(body.into(), FunctionFlags::CALLABLE),
            function_prototype,
        );
        function.set(PROTOTYPE.into(), proto);
        function.set("length".into(), length.into());
        function.set("name".into(), name.into());

        Ok(GcObject::new(function))
    }

    pub fn register_global_function(
        &mut self,
        name: &str,
        length: usize,
        body: NativeFunction,
    ) -> Result<()> {
        let function = self.create_builtin_function(name, length, body)?;
        self.global().set_field(name, function);
        Ok(())
    }

    /// <https://tc39.es/ecma262/#sec-call>
    pub(crate) fn call(&mut self, f: &Value, this: &Value, args: &[Value]) -> Result<Value> {
        match *f {
            Value::Object(ref object) => object.call(this, args, self),
            _ => self.throw_type_error("not a function"),
        }
    }

    /// Converts an array object into a rust vector of values.
    ///
    /// This is useful for the spread operator, for any other object an `Err` is returned
    pub(crate) fn extract_array_properties(&mut self, value: &Value) -> StdResult<Vec<Value>, ()> {
        if let Value::Object(ref x) = value {
            // Check if object is array
            if let ObjectData::Array = x.borrow().data {
                let length = value.get_field("length").as_number().unwrap() as i32;
                let values = (0..length)
                    .map(|idx| value.get_field(idx.to_string()))
                    .collect();
                return Ok(values);
            }
            // Check if object is a Map
            else if let ObjectData::Map(ref map) = x.borrow().data {
                let values = map
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
    pub(crate) fn ordinary_to_primitive(
        &mut self,
        o: &Value,
        hint: PreferredType,
    ) -> Result<Value> {
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

    /// https://tc39.es/ecma262/#sec-hasproperty
    pub(crate) fn has_property(&self, obj: &Value, key: &PropertyKey) -> bool {
        if let Some(obj) = obj.as_object() {
            obj.has_property(key)
        } else {
            false
        }
    }

    fn set_value(&mut self, node: &Node, value: Value) -> Result<Value> {
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
                let key = field.to_property_key(self)?;
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

    /// A helper function for getting a immutable reference to the `console` object.
    pub(crate) fn console(&self) -> &Console {
        &self.console
    }

    /// A helper function for getting a mutable reference to the `console` object.
    pub(crate) fn console_mut(&mut self) -> &mut Console {
        &mut self.console
    }

    /// Construct a new `Symbol` with an optional description.
    #[inline]
    pub fn construct_symbol(&mut self, description: Option<RcString>) -> RcSymbol {
        RcSymbol::from(Symbol::new(self.generate_hash(), description))
    }

    /// Construct an empty object.
    #[inline]
    pub fn construct_object(&self) -> GcObject {
        let object_prototype = self.global().get_field("Object").get_field(PROTOTYPE);
        GcObject::new(Object::create(object_prototype))
    }

    /// Register a global class of type `T`, where `T` implemets `Class`.
    ///
    /// # Example
    /// ```ignore
    /// #[derive(Debug, Trace, Finalize)]
    /// struct MyClass;
    ///
    /// impl Class for MyClass {
    ///    // ...
    /// }
    ///
    /// context.register_global_class::<MyClass>();
    /// ```
    pub fn register_global_class<T>(&mut self) -> Result<()>
    where
        T: Class,
    {
        let mut class_builder = ClassBuilder::new::<T>(self);
        T::init(&mut class_builder)?;

        let class = class_builder.build();
        let property = Property::data_descriptor(class.into(), T::ATTRIBUTE);
        self.global()
            .as_object_mut()
            .unwrap()
            .insert_property(T::NAME, property);
        Ok(())
    }
}

impl Executable for Node {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("Executable", "exec");
        match *self {
            Node::Const(Const::Null) => Ok(Value::null()),
            Node::Const(Const::Num(num)) => Ok(Value::rational(num)),
            Node::Const(Const::Int(num)) => Ok(Value::integer(num)),
            Node::Const(Const::BigInt(ref num)) => Ok(Value::from(num.clone())),
            Node::Const(Const::Undefined) => Ok(Value::Undefined),
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
            Node::ConditionalOp(ref op) => op.run(interpreter),
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
            Node::Continue(ref continue_node) => continue_node.run(interpreter),
        }
    }
}
