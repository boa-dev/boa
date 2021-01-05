//! This module implements the `GcObject` structure.
//!
//! The `GcObject` is a garbage collected Object.

use super::{NativeObject, Object, PROTOTYPE};
use crate::{
    builtins::function::{
        create_unmapped_arguments_object, BuiltInFunction, Function, NativeFunction,
    },
    environment::{
        function_environment_record::BindingStatus, lexical_environment::new_function_environment,
    },
    property::{AccessorDescriptor, Attribute, DataDescriptor, PropertyDescriptor, PropertyKey},
    syntax::ast::node::RcStatementList,
    value::PreferredType,
    Context, Executable, Result, Value,
};
use gc::{Finalize, Gc, GcCell, GcCellRef, GcCellRefMut, Trace};
use serde_json::{map::Map, Value as JSONValue};
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fmt::{self, Debug, Display},
    result::Result as StdResult,
};

/// A wrapper type for an immutably borrowed type T.
pub type Ref<'a, T> = GcCellRef<'a, T>;

/// A wrapper type for a mutably borrowed type T.
pub type RefMut<'a, T> = GcCellRefMut<'a, T>;

/// Garbage collected `Object`.
#[derive(Trace, Finalize, Clone, Default)]
pub struct GcObject(Gc<GcCell<Object>>);

/// The body of a JavaScript function.
///
/// This is needed for the call method since we cannot mutate the function itself since we
/// already borrow it so we get the function body clone it then drop the borrow and run the body
enum FunctionBody {
    BuiltIn(NativeFunction),
    Ordinary(RcStatementList),
}

impl GcObject {
    /// Create a new `GcObject` from a `Object`.
    #[inline]
    pub fn new(object: Object) -> Self {
        Self(Gc::new(GcCell::new(object)))
    }

    /// Immutably borrows the `Object`.
    ///
    /// The borrow lasts until the returned `Ref` exits scope.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn borrow(&self) -> Ref<'_, Object> {
        self.try_borrow().expect("Object already mutably borrowed")
    }

    /// Mutably borrows the Object.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope.
    /// The object cannot be borrowed while this borrow is active.
    ///
    ///# Panics
    /// Panics if the object is currently borrowed.
    #[inline]
    #[track_caller]
    pub fn borrow_mut(&self) -> RefMut<'_, Object> {
        self.try_borrow_mut().expect("Object already borrowed")
    }

    /// Immutably borrows the `Object`, returning an error if the value is currently mutably borrowed.
    ///
    /// The borrow lasts until the returned `GcCellRef` exits scope.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    #[inline]
    pub fn try_borrow(&self) -> StdResult<Ref<'_, Object>, BorrowError> {
        self.0.try_borrow().map_err(|_| BorrowError)
    }

    /// Mutably borrows the object, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts until the returned `GcCellRefMut` exits scope.
    /// The object be borrowed while this borrow is active.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    #[inline]
    pub fn try_borrow_mut(&self) -> StdResult<RefMut<'_, Object>, BorrowMutError> {
        self.0.try_borrow_mut().map_err(|_| BorrowMutError)
    }

    /// Checks if the garbage collected memory is the same.
    #[inline]
    pub fn equals(lhs: &Self, rhs: &Self) -> bool {
        std::ptr::eq(lhs.as_ref(), rhs.as_ref())
    }

    /// Call this object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
    // <https://tc39.es/ecma262/#sec-ecmascript-function-objects-call-thisargument-argumentslist>
    #[track_caller]
    pub fn call(&self, this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let this_function_object = self.clone();
        let f_body = if let Some(function) = self.borrow().as_function() {
            if function.is_callable() {
                match function {
                    Function::BuiltIn(BuiltInFunction(function), _) => {
                        FunctionBody::BuiltIn(*function)
                    }
                    Function::Ordinary {
                        body,
                        params,
                        environment,
                        flags,
                    } => {
                        // Create a new Function environment whose parent is set to the scope of the function declaration (self.environment)
                        // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                        let local_env = new_function_environment(
                            this_function_object,
                            if flags.is_lexical_this_mode() {
                                None
                            } else {
                                Some(this.clone())
                            },
                            Some(environment.clone()),
                            // Arrow functions do not have a this binding https://tc39.es/ecma262/#sec-function-environment-records
                            if flags.is_lexical_this_mode() {
                                BindingStatus::Lexical
                            } else {
                                BindingStatus::Uninitialized
                            },
                        );

                        // Add argument bindings to the function environment
                        for (i, param) in params.iter().enumerate() {
                            // Rest Parameters
                            if param.is_rest_param() {
                                function.add_rest_param(param, i, args, context, &local_env);
                                break;
                            }

                            let value = args.get(i).cloned().unwrap_or_else(Value::undefined);
                            function.add_arguments_to_environment(param, value, &local_env);
                        }

                        // Add arguments object
                        let arguments_obj = create_unmapped_arguments_object(args);
                        local_env
                            .borrow_mut()
                            .create_mutable_binding("arguments".to_string(), false, false)
                            .map_err(|e| e.to_error(context))?;
                        local_env
                            .borrow_mut()
                            .initialize_binding("arguments", arguments_obj)
                            .map_err(|e| e.to_error(context))?;

                        context.realm_mut().environment.push(local_env);

                        FunctionBody::Ordinary(body.clone())
                    }
                }
            } else {
                return context.throw_type_error("function object is not callable");
            }
        } else {
            return context.throw_type_error("not a function");
        };

        match f_body {
            FunctionBody::BuiltIn(func) => func(this, args, context),
            FunctionBody::Ordinary(body) => {
                let result = body.run(context);
                context.realm_mut().environment.pop();

                result
            }
        }
    }

    /// Construct an instance of this object with the specified arguments.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    // <https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget>
    #[track_caller]
    pub fn construct(&self, args: &[Value], context: &mut Context) -> Result<Value> {
        // If the prototype of the constructor is not an object, then use the default object
        // prototype as prototype for the new object
        // see <https://tc39.es/ecma262/#sec-ordinarycreatefromconstructor>
        // see <https://tc39.es/ecma262/#sec-getprototypefromconstructor>
        let proto = self.get(&PROTOTYPE.into(), self.clone().into(), context)?;
        let proto = if proto.is_object() {
            proto
        } else {
            context
                .standard_objects()
                .object_object()
                .prototype()
                .into()
        };
        let this: Value = Object::create(proto).into();

        let this_function_object = self.clone();
        let body = if let Some(function) = self.borrow().as_function() {
            if function.is_constructable() {
                match function {
                    Function::BuiltIn(BuiltInFunction(function), _) => {
                        FunctionBody::BuiltIn(*function)
                    }
                    Function::Ordinary {
                        body,
                        params,
                        environment,
                        flags,
                    } => {
                        // Create a new Function environment who's parent is set to the scope of the function declaration (self.environment)
                        // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                        let local_env = new_function_environment(
                            this_function_object,
                            Some(this.clone()),
                            Some(environment.clone()),
                            // Arrow functions do not have a this binding https://tc39.es/ecma262/#sec-function-environment-records
                            if flags.is_lexical_this_mode() {
                                BindingStatus::Lexical
                            } else {
                                BindingStatus::Uninitialized
                            },
                        );

                        // Add argument bindings to the function environment
                        for (i, param) in params.iter().enumerate() {
                            // Rest Parameters
                            if param.is_rest_param() {
                                function.add_rest_param(param, i, args, context, &local_env);
                                break;
                            }

                            let value = args.get(i).cloned().unwrap_or_else(Value::undefined);
                            function.add_arguments_to_environment(param, value, &local_env);
                        }

                        // Add arguments object
                        let arguments_obj = create_unmapped_arguments_object(args);
                        local_env
                            .borrow_mut()
                            .create_mutable_binding("arguments".to_string(), false, false)
                            .map_err(|e| e.to_error(context))?;
                        local_env
                            .borrow_mut()
                            .initialize_binding("arguments", arguments_obj)
                            .map_err(|e| e.to_error(context))?;

                        context.realm_mut().environment.push(local_env);

                        FunctionBody::Ordinary(body.clone())
                    }
                }
            } else {
                let name = this.get_field("name", context)?.display().to_string();
                return context.throw_type_error(format!("{} is not a constructor", name));
            }
        } else {
            return context.throw_type_error("not a function");
        };

        match body {
            FunctionBody::BuiltIn(function) => {
                function(&this, args, context)?;
                Ok(this)
            }
            FunctionBody::Ordinary(body) => {
                let _ = body.run(context);

                // local_env gets dropped here, its no longer needed
                let binding = context.realm_mut().environment.get_this_binding();
                binding.map_err(|e| e.to_error(context))
            }
        }
    }

    /// Converts an object to a primitive.
    ///
    /// Diverges from the spec to prevent a stack overflow when the object is recursive.
    /// For example,
    /// ```javascript
    /// let a = [1];
    /// a[1] = a;
    /// console.log(a.toString()); // We print "1,"
    /// ```
    /// The spec doesn't mention what to do in this situation, but a naive implementation
    /// would overflow the stack recursively calling `toString()`. We follow v8 and SpiderMonkey
    /// instead by returning a default value for the given `hint` -- either `0.` or `""`.
    /// Example in v8: <https://repl.it/repls/IvoryCircularCertification#index.js>
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinarytoprimitive
    pub(crate) fn ordinary_to_primitive(
        &self,
        context: &mut Context,
        hint: PreferredType,
    ) -> Result<Value> {
        // 1. Assert: Type(O) is Object.
        //      Already is GcObject by type.
        // 2. Assert: Type(hint) is String and its value is either "string" or "number".
        debug_assert!(hint == PreferredType::String || hint == PreferredType::Number);

        // Diverge from the spec here to make sure we aren't going to overflow the stack by converting
        // a recursive structure
        // We can follow v8 & SpiderMonkey's lead and return a default value for the hint in this situation
        // (see https://repl.it/repls/IvoryCircularCertification#index.js)
        let recursion_limiter = RecursionLimiter::new(&self);
        if recursion_limiter.live {
            // we're in a recursive object, bail
            return Ok(match hint {
                PreferredType::Number => Value::from(0),
                PreferredType::String => Value::from(""),
                PreferredType::Default => unreachable!("checked type hint in step 2"),
            });
        }

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
        let this = Value::from(self.clone());
        for name in &method_names {
            // a. Let method be ? Get(O, name).
            let method: Value = this.get_field(*name, context)?;
            // b. If IsCallable(method) is true, then
            if method.is_function() {
                // i. Let result be ? Call(method, O).
                let result = context.call(&method, &this, &[])?;
                // ii. If Type(result) is not Object, return result.
                if !result.is_object() {
                    return Ok(result);
                }
            }
        }

        // 6. Throw a TypeError exception.
        context.throw_type_error("cannot convert object to primitive value")
    }

    /// Converts an object to JSON, checking for reference cycles and throwing a TypeError if one is found
    pub(crate) fn to_json(&self, context: &mut Context) -> Result<JSONValue> {
        let rec_limiter = RecursionLimiter::new(self);
        if rec_limiter.live {
            Err(context.construct_type_error("cyclic object value"))
        } else if self.is_array() {
            let mut keys: Vec<u32> = self.borrow().index_property_keys().cloned().collect();
            keys.sort_unstable();
            let mut arr: Vec<JSONValue> = Vec::with_capacity(keys.len());
            let this = Value::from(self.clone());
            for key in keys {
                let value = this.get_field(key, context)?;
                if value.is_undefined() || value.is_function() || value.is_symbol() {
                    arr.push(JSONValue::Null);
                } else {
                    arr.push(value.to_json(context)?);
                }
            }
            Ok(JSONValue::Array(arr))
        } else {
            let mut new_obj = Map::new();
            let this = Value::from(self.clone());
            for k in self.borrow().keys() {
                let key = k.clone();
                let value = this.get_field(k.to_string(), context)?;
                if !value.is_undefined() && !value.is_function() && !value.is_symbol() {
                    new_obj.insert(key.to_string(), value.to_json(context)?);
                }
            }
            Ok(JSONValue::Object(new_obj))
        }
    }

    /// Convert the object to a `PropertyDescritptor`
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    pub fn to_property_descriptor(&self, context: &mut Context) -> Result<PropertyDescriptor> {
        let mut attribute = Attribute::empty();

        let enumerable_key = PropertyKey::from("enumerable");
        if self.has_property(&enumerable_key)
            && self
                .get(&enumerable_key, self.clone().into(), context)?
                .to_boolean()
        {
            attribute |= Attribute::ENUMERABLE;
        }

        let configurable_key = PropertyKey::from("configurable");
        if self.has_property(&configurable_key)
            && self
                .get(&configurable_key, self.clone().into(), context)?
                .to_boolean()
        {
            attribute |= Attribute::CONFIGURABLE;
        }

        let mut value = None;
        let value_key = PropertyKey::from("value");
        if self.has_property(&value_key) {
            value = Some(self.get(&value_key, self.clone().into(), context)?);
        }

        let mut has_writable = false;
        let writable_key = PropertyKey::from("writable");
        if self.has_property(&writable_key) {
            has_writable = true;
            if self
                .get(&writable_key, self.clone().into(), context)?
                .to_boolean()
            {
                attribute |= Attribute::WRITABLE;
            }
        }

        let mut get = None;
        let get_key = PropertyKey::from("get");
        if self.has_property(&get_key) {
            let getter = self.get(&get_key, self.clone().into(), context)?;
            match getter {
                Value::Object(ref object) if object.is_callable() => {
                    get = Some(object.clone());
                }
                _ => {
                    return Err(
                        context.construct_type_error("Property descriptor getter must be callable")
                    );
                }
            }
        }

        let mut set = None;
        let set_key = PropertyKey::from("set");
        if self.has_property(&set_key) {
            let setter = self.get(&set_key, self.clone().into(), context)?;
            match setter {
                Value::Object(ref object) if object.is_callable() => {
                    set = Some(object.clone());
                }
                _ => {
                    return Err(
                        context.construct_type_error("Property descriptor setter must be callable")
                    );
                }
            };
        }

        if get.is_some() || set.is_some() {
            if value.is_some() || has_writable {
                return Err(context.construct_type_error("Invalid property descriptor. Cannot both specify accessors and a value or writable attribute"));
            }

            Ok(AccessorDescriptor::new(get, set, attribute).into())
        } else {
            Ok(DataDescriptor::new(value.unwrap_or_else(Value::undefined), attribute).into())
        }
    }

    /// Reeturn `true` if it is a native object and the native type is `T`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is<T>(&self) -> bool
    where
        T: NativeObject,
    {
        self.borrow().is::<T>()
    }

    /// Downcast a reference to the object,
    /// if the object is type native object type `T`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn downcast_ref<T>(&self) -> Option<Ref<'_, T>>
    where
        T: NativeObject,
    {
        let object = self.borrow();
        if object.is::<T>() {
            Some(Ref::map(object, |x| x.downcast_ref::<T>().unwrap()))
        } else {
            None
        }
    }

    /// Downcast a mutable reference to the object,
    /// if the object is type native object type `T`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently borrowed.
    #[inline]
    #[track_caller]
    pub fn downcast_mut<T>(&mut self) -> Option<RefMut<'_, T>>
    where
        T: NativeObject,
    {
        let object = self.borrow_mut();
        if object.is::<T>() {
            Some(RefMut::map(object, |x| x.downcast_mut::<T>().unwrap()))
        } else {
            None
        }
    }

    /// Get the prototype of the object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn prototype_instance(&self) -> Value {
        self.borrow().prototype_instance().clone()
    }

    /// Set the prototype of the object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed
    /// or if th prototype is not an object or undefined.
    #[inline]
    #[track_caller]
    pub fn set_prototype_instance(&mut self, prototype: Value) -> bool {
        self.borrow_mut().set_prototype_instance(prototype)
    }

    /// Checks if it an `Array` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_array(&self) -> bool {
        self.borrow().is_array()
    }

    /// Checks if it is an `ArrayIterator` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_array_iterator(&self) -> bool {
        self.borrow().is_array_iterator()
    }

    /// Checks if it is a `Map` object.pub
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_map(&self) -> bool {
        self.borrow().is_map()
    }

    /// Checks if it a `String` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_string(&self) -> bool {
        self.borrow().is_string()
    }

    /// Checks if it a `Function` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_function(&self) -> bool {
        self.borrow().is_function()
    }

    /// Checks if it a Symbol object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_symbol(&self) -> bool {
        self.borrow().is_symbol()
    }

    /// Checks if it an Error object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_error(&self) -> bool {
        self.borrow().is_error()
    }

    /// Checks if it a Boolean object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_boolean(&self) -> bool {
        self.borrow().is_boolean()
    }

    /// Checks if it a `Number` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_number(&self) -> bool {
        self.borrow().is_number()
    }

    /// Checks if it a `BigInt` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_bigint(&self) -> bool {
        self.borrow().is_bigint()
    }

    /// Checks if it a `RegExp` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_regexp(&self) -> bool {
        self.borrow().is_regexp()
    }

    /// Checks if it an ordinary object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_ordinary(&self) -> bool {
        self.borrow().is_ordinary()
    }

    /// Returns `true` if it holds an Rust type that implements `NativeObject`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_native_object(&self) -> bool {
        self.borrow().is_native_object()
    }

    /// Retrieves value of specific property, when the value of the property is expected to be a function.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getmethod
    #[inline]
    pub fn get_method<K>(&self, context: &mut Context, key: K) -> Result<Option<GcObject>>
    where
        K: Into<PropertyKey>,
    {
        let key = key.into();
        let value = self.get(&key, self.clone().into(), context)?;

        if value.is_null_or_undefined() {
            return Ok(None);
        }

        match value.as_object() {
            Some(object) if object.is_callable() => Ok(Some(object)),
            _ => Err(context
                .construct_type_error("value returned for property of object is not a function")),
        }
    }

    /// Determines if `value` inherits from the instance object inheritance path.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinaryhasinstance
    #[inline]
    pub(crate) fn ordinary_has_instance(
        &self,
        context: &mut Context,
        value: &Value,
    ) -> Result<bool> {
        if !self.is_callable() {
            return Ok(false);
        }

        // TODO: If C has a [[BoundTargetFunction]] internal slot, then
        //           Let BC be C.[[BoundTargetFunction]].
        //           Return ? InstanceofOperator(O, BC).

        if let Some(object) = value.as_object() {
            if let Some(prototype) = self
                .get(&"prototype".into(), self.clone().into(), context)?
                .as_object()
            {
                let mut object = object.get_prototype_of();
                while let Some(object_prototype) = object.as_object() {
                    if GcObject::equals(&prototype, &object_prototype) {
                        return Ok(true);
                    }
                    object = object_prototype.get_prototype_of();
                }

                Ok(false)
            } else {
                Err(context
                    .construct_type_error("function has non-object prototype in instanceof check"))
            }
        } else {
            Ok(false)
        }
    }
    #[inline]
    #[track_caller]
    pub fn has_own_property<K>(&self, key: K) -> bool
    where
        K: Into<PropertyKey>,
    {
        let key = key.into();
        self.get_own_property(&key).is_some()
    }

    /// Defines the property or throws a `TypeError` if the operation fails.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-definepropertyorthrow
    #[inline]
    pub(crate) fn define_property_or_throw<K, P>(
        &mut self,
        key: K,
        desc: P,
        context: &mut Context,
    ) -> Result<()>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        let key = key.into();
        let desc = desc.into();

        let success = self.define_own_property(key.clone(), desc, context)?;
        if !success {
            Err(context.construct_type_error(format!("Cannot redefine property: {}", key)))
        } else {
            Ok(())
        }
    }
}

impl AsRef<GcCell<Object>> for GcObject {
    #[inline]
    fn as_ref(&self) -> &GcCell<Object> {
        &*self.0
    }
}

/// An error returned by [`GcObject::try_borrow`](struct.GcObject.html#method.try_borrow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowError;

impl Display for BorrowError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Object already mutably borrowed", f)
    }
}

impl Error for BorrowError {}

/// An error returned by [`GcObject::try_borrow_mut`](struct.GcObject.html#method.try_borrow_mut).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowMutError;

impl Display for BorrowMutError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Object already borrowed", f)
    }
}

impl Error for BorrowMutError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RecursionValueState {
    /// This value is "live": there's an active RecursionLimiter that hasn't been dropped.
    Live,
    /// This value has been seen before, but the recursion limiter has been dropped.
    /// For example:
    /// ```javascript
    /// let b = [];
    /// JSON.stringify([ // Create a recursion limiter for the root here
    ///    b,            // state for b's &GcObject here is None
    ///    b,            // state for b's &GcObject here is Visited
    /// ]);
    /// ```
    Visited,
}

/// Prevents infinite recursion during `Debug::fmt`, `JSON.stringify`, and other conversions.
/// This uses a thread local, so is not safe to use where the object graph will be traversed by
/// multiple threads!
#[derive(Debug)]
pub struct RecursionLimiter {
    /// If this was the first `GcObject` in the tree.
    top_level: bool,
    /// The ptr being kept in the HashSet, so we can delete it when we drop.
    ptr: usize,
    /// If this GcObject has been visited before in the graph, but not in the current branch.
    pub visited: bool,
    /// If this GcObject has been visited in the current branch of the graph.
    pub live: bool,
}

impl Drop for RecursionLimiter {
    fn drop(&mut self) {
        if self.top_level {
            // When the top level of the graph is dropped, we can free the entire map for the next traversal.
            Self::SEEN.with(|hm| hm.borrow_mut().clear());
        } else if !self.live {
            // This was the first RL for this object to become live, so it's no longer live now that it's dropped.
            Self::SEEN.with(|hm| {
                hm.borrow_mut()
                    .insert(self.ptr, RecursionValueState::Visited)
            });
        }
    }
}

impl RecursionLimiter {
    thread_local! {
        /// The map of pointers to `GcObject` that have been visited during the current `Debug::fmt` graph,
        /// and the current state of their RecursionLimiter (dropped or live -- see `RecursionValueState`)
        static SEEN: RefCell<HashMap<usize, RecursionValueState>> = RefCell::new(HashMap::new());
    }

    /// Determines if the specified `GcObject` has been visited, and returns a struct that will free it when dropped.
    ///
    /// This is done by maintaining a thread-local hashset containing the pointers of `GcObject` values that have been
    /// visited. The first `GcObject` visited will clear the hashset, while any others will check if they are contained
    /// by the hashset.
    pub fn new(o: &GcObject) -> Self {
        // We shouldn't have to worry too much about this being moved during Debug::fmt.
        let ptr = (o.as_ref() as *const _) as usize;
        let (top_level, visited, live) = Self::SEEN.with(|hm| {
            let mut hm = hm.borrow_mut();
            let top_level = hm.is_empty();
            let old_state = hm.insert(ptr, RecursionValueState::Live);

            (
                top_level,
                old_state == Some(RecursionValueState::Visited),
                old_state == Some(RecursionValueState::Live),
            )
        });

        Self {
            top_level,
            ptr,
            visited,
            live,
        }
    }
}

impl Debug for GcObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let limiter = RecursionLimiter::new(&self);

        // Typically, using `!limiter.live` would be good enough here.
        // However, the JS object hierarchy involves quite a bit of repitition, and the sheer amount of data makes
        // understanding the Debug output impossible; limiting the usefulness of it.
        //
        // Instead, we check if the object has appeared before in the entire graph. This means that objects will appear
        // at most once, hopefully making things a bit clearer.
        if !limiter.visited && !limiter.live {
            f.debug_tuple("GcObject").field(&self.0).finish()
        } else {
            f.write_str("{ ... }")
        }
    }
}
