//! Javascript context.

use crate::{
    builtins::{
        self,
        function::{Function, FunctionFlags, NativeFunction},
        iterable::IteratorPrototypes,
        symbol::{Symbol, WellKnownSymbols},
        Console,
    },
    class::{Class, ClassBuilder},
    exec::Interpreter,
    object::{GcObject, Object, ObjectData, PROTOTYPE},
    property::{Property, PropertyKey},
    realm::Realm,
    syntax::{
        ast::{
            node::{
                statement_list::RcStatementList, Call, FormalParameter, Identifier, New,
                StatementList,
            },
            Const, Node,
        },
        Parser,
    },
    value::{PreferredType, RcString, RcSymbol, Type, Value},
    BoaProfiler, Executable, Result,
};
use std::result::Result as StdResult;

/// Javascript context. It is the primary way to interact with the runtime.
///
/// For each `Context` instance a new instance of runtime is created.
/// It means that it is safe to use different contexts in different threads,
/// but each `Context` instance must be used only from a single thread.
#[derive(Debug)]
pub struct Context {
    /// realm holds both the global object and the environment
    realm: Realm,

    /// The current executor.
    executor: Interpreter,

    /// Symbol hash.
    ///
    /// For now this is an incremented u32 number.
    symbol_count: u32,

    /// console object state.
    console: Console,

    /// Cached well known symbols
    well_known_symbols: WellKnownSymbols,

    iterator_prototypes: IteratorPrototypes,
}

impl Default for Context {
    fn default() -> Self {
        let realm = Realm::create();
        let executor = Interpreter::new();
        let (well_known_symbols, symbol_count) = WellKnownSymbols::new();
        let mut context = Self {
            realm,
            executor,
            symbol_count,
            console: Console::default(),
            well_known_symbols,
            iterator_prototypes: IteratorPrototypes::default(),
        };

        // Add new builtIns to Context Realm
        // At a later date this can be removed from here and called explicitly,
        // but for now we almost always want these default builtins
        context.create_intrinsics();
        context.iterator_prototypes = IteratorPrototypes::init(&mut context);
        context
    }
}

impl Context {
    /// Create a new `Context`.
    pub fn new() -> Self {
        Default::default()
    }

    pub fn realm(&self) -> &Realm {
        &self.realm
    }

    pub fn realm_mut(&mut self) -> &mut Realm {
        &mut self.realm
    }

    pub fn executor(&mut self) -> &mut Interpreter {
        &mut self.executor
    }

    /// A helper function for getting a immutable reference to the `console` object.
    pub(crate) fn console(&self) -> &Console {
        &self.console
    }

    /// A helper function for getting a mutable reference to the `console` object.
    pub(crate) fn console_mut(&mut self) -> &mut Console {
        &mut self.console
    }

    /// Sets up the default global objects within Global
    fn create_intrinsics(&mut self) {
        let _timer = BoaProfiler::global().start_event("create_intrinsics", "interpreter");
        // Create intrinsics, add global objects here
        builtins::init(self);
    }

    /// Generates a new `Symbol` internal hash.
    ///
    /// This currently is an incremented value.
    #[inline]
    fn generate_hash(&mut self) -> u32 {
        let hash = self.symbol_count;
        self.symbol_count += 1;
        hash
    }

    /// Construct a new `Symbol` with an optional description.
    #[inline]
    pub fn construct_symbol(&mut self, description: Option<RcString>) -> RcSymbol {
        RcSymbol::from(Symbol::new(self.generate_hash(), description))
    }

    /// Construct an empty object.
    #[inline]
    pub fn construct_object(&self) -> GcObject {
        let object_prototype = self
            .global_object()
            .get_field("Object")
            .get_field(PROTOTYPE);
        GcObject::new(Object::create(object_prototype))
    }

    /// <https://tc39.es/ecma262/#sec-call>
    pub(crate) fn call(&mut self, f: &Value, this: &Value, args: &[Value]) -> Result<Value> {
        match *f {
            Value::Object(ref object) => object.call(this, args, self),
            _ => self.throw_type_error("not a function"),
        }
    }

    /// Return the global object.
    pub fn global_object(&self) -> &Value {
        &self.realm.global_obj
    }

    /// Constructs a `RangeError` with the specified message.
    pub fn construct_range_error<M>(&mut self, message: M) -> Value
    where
        M: Into<String>,
    {
        // Runs a `new RangeError(message)`.
        New::from(Call::new(
            Identifier::from("RangeError"),
            vec![Const::from(message.into()).into()],
        ))
        .run(self)
        .expect_err("RangeError should always throw")
    }

    /// Throws a `RangeError` with the specified message.
    pub fn throw_range_error<M>(&mut self, message: M) -> Result<Value>
    where
        M: Into<String>,
    {
        Err(self.construct_range_error(message))
    }

    /// Constructs a `TypeError` with the specified message.
    pub fn construct_type_error<M>(&mut self, message: M) -> Value
    where
        M: Into<String>,
    {
        // Runs a `new TypeError(message)`.
        New::from(Call::new(
            Identifier::from("TypeError"),
            vec![Const::from(message.into()).into()],
        ))
        .run(self)
        .expect_err("TypeError should always throw")
    }

    /// Throws a `TypeError` with the specified message.
    pub fn throw_type_error<M>(&mut self, message: M) -> Result<Value>
    where
        M: Into<String>,
    {
        Err(self.construct_type_error(message))
    }

    /// Constructs a `ReferenceError` with the specified message.
    pub fn construct_reference_error<M>(&mut self, message: M) -> Value
    where
        M: Into<String>,
    {
        New::from(Call::new(
            Identifier::from("ReferenceError"),
            vec![Const::from(message.into() + " is not defined").into()],
        ))
        .run(self)
        .expect_err("ReferenceError should always throw")
    }

    /// Throws a `ReferenceError` with the specified message.
    pub fn throw_reference_error<M>(&mut self, message: M) -> Result<Value>
    where
        M: Into<String>,
    {
        Err(self.construct_reference_error(message))
    }

    /// Constructs a `SyntaxError` with the specified message.
    pub fn construct_syntax_error<M>(&mut self, message: M) -> Value
    where
        M: Into<String>,
    {
        New::from(Call::new(
            Identifier::from("SyntaxError"),
            vec![Const::from(message.into()).into()],
        ))
        .run(self)
        .expect_err("SyntaxError should always throw")
    }

    /// Throws a `SyntaxError` with the specified message.
    pub fn throw_syntax_error<M>(&mut self, message: M) -> Result<Value>
    where
        M: Into<String>,
    {
        Err(self.construct_syntax_error(message))
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
        let function_prototype = self
            .global_object()
            .get_field("Function")
            .get_field(PROTOTYPE);

        // Every new function has a prototype property pre-made
        let proto = Value::new_object(Some(self.global_object()));

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

    /// Create a new builin function.
    pub fn create_builtin_function(
        &mut self,
        name: &str,
        length: usize,
        body: NativeFunction,
    ) -> Result<GcObject> {
        let function_prototype = self
            .global_object()
            .get_field("Function")
            .get_field(PROTOTYPE);

        // Every new function has a prototype property pre-made
        let proto = Value::new_object(Some(self.global_object()));
        let mut function = Object::function(
            Function::BuiltIn(body.into(), FunctionFlags::CALLABLE),
            function_prototype,
        );
        function.set(PROTOTYPE.into(), proto);
        function.set("length".into(), length.into());
        function.set("name".into(), name.into());

        Ok(GcObject::new(function))
    }

    /// Register a global function.
    pub fn register_global_function(
        &mut self,
        name: &str,
        length: usize,
        body: NativeFunction,
    ) -> Result<()> {
        let function = self.create_builtin_function(name, length, body)?;
        self.global_object().set_field(name, function);
        Ok(())
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
                        array
                            .as_object_mut()
                            .expect("object")
                            .set_prototype_instance(
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

    pub(crate) fn set_value(&mut self, node: &Node, value: Value) -> Result<Value> {
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
        self.global_object()
            .as_object_mut()
            .unwrap()
            .insert_property(T::NAME, property);
        Ok(())
    }

    fn parser_expr(src: &str) -> StdResult<StatementList, String> {
        Parser::new(src.as_bytes())
            .parse_all()
            .map_err(|e| e.to_string())
    }

    /// Evaluates the given code.
    ///
    /// # Examples
    /// ```
    ///# use boa::Context;
    /// let mut context = Context::new();
    ///
    /// let value = context.eval("1 + 3").unwrap();
    ///
    /// assert!(value.is_number());
    /// assert_eq!(value.as_number().unwrap(), 4.0);
    /// ```
    #[allow(clippy::unit_arg, clippy::drop_copy)]
    pub fn eval(&mut self, src: &str) -> Result<Value> {
        let main_timer = BoaProfiler::global().start_event("Main", "Main");

        let result = match Self::parser_expr(src) {
            Ok(expr) => expr.run(self),
            Err(e) => self.throw_syntax_error(e),
        };

        // The main_timer needs to be dropped before the BoaProfiler is.
        drop(main_timer);
        BoaProfiler::global().drop();

        result
    }

    /// Returns a structure that contains the JavaScript well known symbols.
    ///
    /// # Examples
    /// ```
    ///# use boa::Context;
    /// let mut context = Context::new();
    ///
    /// let iterator = context.well_known_symbols().iterator_symbol();
    /// assert_eq!(iterator.description(), Some("Symbol.iterator"));
    /// ```
    /// This is equivalent to `let iterator = Symbol.iterator` in JavaScript.
    #[inline]
    pub fn well_known_symbols(&self) -> &WellKnownSymbols {
        &self.well_known_symbols
    }

    #[inline]
    pub fn iterator_prototypes(&self) -> &IteratorPrototypes {
        &self.iterator_prototypes
    }
}
