//! This module implements the `JsObject` structure.
//!
//! The `JsObject` is a garbage collected Object.

use super::{NativeObject, Object, PROTOTYPE};
use crate::{
    builtins::function::{
        create_unmapped_arguments_object, ClosureFunction, Function, NativeFunction,
    },
    environment::{
        environment_record_trait::EnvironmentRecordTrait,
        function_environment_record::{BindingStatus, FunctionEnvironmentRecord},
        lexical_environment::Environment,
    },
    exec::InterpreterState,
    object::{ObjectData, ObjectKind},
    property::{PropertyDescriptor, PropertyKey},
    syntax::ast::node::RcStatementList,
    value::PreferredType,
    Context, Executable, JsResult, JsValue,
};
use gc::{Finalize, Gc, GcCell, GcCellRef, GcCellRefMut, Trace};
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
pub type RefMut<'a, T, U> = GcCellRefMut<'a, T, U>;

/// Garbage collected `Object`.
#[derive(Trace, Finalize, Clone, Default)]
pub struct JsObject(Gc<GcCell<Object>>);

/// The body of a JavaScript function.
///
/// This is needed for the call method since we cannot mutate the function itself since we
/// already borrow it so we get the function body clone it then drop the borrow and run the body
enum FunctionBody {
    BuiltInFunction(NativeFunction),
    BuiltInConstructor(NativeFunction),
    Closure(Box<dyn ClosureFunction>),
    Ordinary(RcStatementList),
}

impl JsObject {
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
    pub fn borrow_mut(&self) -> RefMut<'_, Object, Object> {
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
    pub fn try_borrow_mut(&self) -> StdResult<RefMut<'_, Object, Object>, BorrowMutError> {
        self.0.try_borrow_mut().map_err(|_| BorrowMutError)
    }

    /// Checks if the garbage collected memory is the same.
    #[inline]
    pub fn equals(lhs: &Self, rhs: &Self) -> bool {
        std::ptr::eq(lhs.as_ref(), rhs.as_ref())
    }

    /// Internal implementation of [`call`](#method.call) and [`construct`](#method.construct).
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    ///
    /// <https://tc39.es/ecma262/#sec-prepareforordinarycall>
    /// <https://tc39.es/ecma262/#sec-ordinarycallbindthis>
    /// <https://tc39.es/ecma262/#sec-runtime-semantics-evaluatebody>
    /// <https://tc39.es/ecma262/#sec-ordinarycallevaluatebody>
    #[track_caller]
    pub(super) fn call_construct(
        &self,
        this_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
        construct: bool,
    ) -> JsResult<JsValue> {
        let this_function_object = self.clone();
        let mut has_parameter_expressions = false;

        let body = if let Some(function) = self.borrow().as_function() {
            if construct && !function.is_constructable() {
                let name = self
                    .__get__(&"name".into(), self.clone().into(), context)?
                    .display()
                    .to_string();
                return context.throw_type_error(format!("{} is not a constructor", name));
            } else {
                match function {
                    Function::Native {
                        function,
                        constructable,
                    } => {
                        if *constructable || construct {
                            FunctionBody::BuiltInConstructor(function.0)
                        } else {
                            FunctionBody::BuiltInFunction(function.0)
                        }
                    }
                    Function::Closure { function, .. } => FunctionBody::Closure(function.clone()),
                    Function::Ordinary {
                        body,
                        params,
                        environment,
                        flags,
                    } => {
                        let this = if construct {
                            // If the prototype of the constructor is not an object, then use the default object
                            // prototype as prototype for the new object
                            // see <https://tc39.es/ecma262/#sec-ordinarycreatefromconstructor>
                            // see <https://tc39.es/ecma262/#sec-getprototypefromconstructor>
                            let proto = this_target.as_object().unwrap().__get__(
                                &PROTOTYPE.into(),
                                this_target.clone(),
                                context,
                            )?;
                            let proto = if proto.is_object() {
                                proto
                            } else {
                                context
                                    .standard_objects()
                                    .object_object()
                                    .prototype()
                                    .into()
                            };
                            JsValue::new(Object::create(proto))
                        } else {
                            this_target.clone()
                        };

                        // Create a new Function environment whose parent is set to the scope of the function declaration (self.environment)
                        // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                        let local_env = FunctionEnvironmentRecord::new(
                            this_function_object.clone(),
                            if construct || !flags.is_lexical_this_mode() {
                                Some(this.clone())
                            } else {
                                None
                            },
                            Some(environment.clone()),
                            // Arrow functions do not have a this binding https://tc39.es/ecma262/#sec-function-environment-records
                            if flags.is_lexical_this_mode() {
                                BindingStatus::Lexical
                            } else {
                                BindingStatus::Uninitialized
                            },
                            JsValue::undefined(),
                            context,
                        )?;

                        let mut arguments_in_parameter_names = false;

                        for param in params.iter() {
                            has_parameter_expressions =
                                has_parameter_expressions || param.init().is_some();
                            arguments_in_parameter_names =
                                arguments_in_parameter_names || param.name() == "arguments";
                        }

                        // An arguments object is added when all of the following conditions are met
                        // - If not in an arrow function (10.2.11.16)
                        // - If the parameter list does not contain `arguments` (10.2.11.17)
                        // - If there are default parameters or if lexical names and function names do not contain `arguments` (10.2.11.18)
                        //
                        // https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
                        if !flags.is_lexical_this_mode()
                            && !arguments_in_parameter_names
                            && (has_parameter_expressions
                                || (!body.lexically_declared_names().contains("arguments")
                                    && !body.function_declared_names().contains("arguments")))
                        {
                            // Add arguments object
                            let arguments_obj = create_unmapped_arguments_object(args, context)?;
                            local_env.create_mutable_binding("arguments", false, true, context)?;
                            local_env.initialize_binding("arguments", arguments_obj, context)?;
                        }

                        // Turn local_env into Environment so it can be cloned
                        let local_env: Environment = local_env.into();

                        // Push the environment first so that it will be used by default parameters
                        context.push_environment(local_env.clone());

                        // Add argument bindings to the function environment
                        for (i, param) in params.iter().enumerate() {
                            // Rest Parameters
                            if param.is_rest_param() {
                                function.add_rest_param(param, i, args, context, &local_env);
                                break;
                            }

                            let value = match args.get(i).cloned() {
                                None | Some(JsValue::Undefined) => param
                                    .init()
                                    .map(|init| init.run(context).ok())
                                    .flatten()
                                    .unwrap_or_default(),
                                Some(value) => value,
                            };

                            function
                                .add_arguments_to_environment(param, value, &local_env, context);
                        }

                        if has_parameter_expressions {
                            // Create a second environment when default parameter expressions are used
                            // This prevents variables declared in the function body from being
                            // used in default parameter initializers.
                            // https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
                            let second_env = FunctionEnvironmentRecord::new(
                                this_function_object,
                                if construct || !flags.is_lexical_this_mode() {
                                    Some(this)
                                } else {
                                    None
                                },
                                Some(local_env),
                                // Arrow functions do not have a this binding https://tc39.es/ecma262/#sec-function-environment-records
                                if flags.is_lexical_this_mode() {
                                    BindingStatus::Lexical
                                } else {
                                    BindingStatus::Uninitialized
                                },
                                JsValue::undefined(),
                                context,
                            )?;
                            context.push_environment(second_env);
                        }

                        FunctionBody::Ordinary(body.clone())
                    }
                }
            }
        } else {
            return context.throw_type_error("not a function");
        };

        match body {
            FunctionBody::BuiltInConstructor(function) if construct => {
                function(this_target, args, context)
            }
            FunctionBody::BuiltInConstructor(function) => {
                function(&JsValue::undefined(), args, context)
            }
            FunctionBody::BuiltInFunction(function) => function(this_target, args, context),
            FunctionBody::Closure(function) => (function)(this_target, args, context),
            FunctionBody::Ordinary(body) => {
                let result = body.run(context);
                let this = context.get_this_binding();

                if has_parameter_expressions {
                    context.pop_environment();
                }
                context.pop_environment();

                if construct {
                    // https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget
                    // 12. If result.[[Type]] is return, then
                    if context.executor().get_current_state() == &InterpreterState::Return {
                        // a. If Type(result.[[Value]]) is Object, return NormalCompletion(result.[[Value]]).
                        if let Ok(v) = &result {
                            if v.is_object() {
                                return result;
                            }
                        }
                    }

                    // 13. Else, ReturnIfAbrupt(result).
                    result?;

                    // 14. Return ? constructorEnv.GetThisBinding().
                    this
                } else if context.executor().get_current_state() == &InterpreterState::Return {
                    result
                } else {
                    result?;
                    Ok(JsValue::undefined())
                }
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
    ) -> JsResult<JsValue> {
        // 1. Assert: Type(O) is Object.
        //      Already is JsObject by type.
        // 2. Assert: Type(hint) is String and its value is either "string" or "number".
        debug_assert!(hint == PreferredType::String || hint == PreferredType::Number);

        // Diverge from the spec here to make sure we aren't going to overflow the stack by converting
        // a recursive structure
        // We can follow v8 & SpiderMonkey's lead and return a default value for the hint in this situation
        // (see https://repl.it/repls/IvoryCircularCertification#index.js)
        let recursion_limiter = RecursionLimiter::new(self);
        if recursion_limiter.live {
            // we're in a recursive object, bail
            return Ok(match hint {
                PreferredType::Number => JsValue::new(0),
                PreferredType::String => JsValue::new(""),
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
        let this = JsValue::new(self.clone());
        for name in &method_names {
            // a. Let method be ? Get(O, name).
            let method: JsValue = this.get_field(*name, context)?;
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

    /// Return `true` if it is a native object and the native type is `T`.
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
    pub fn downcast_mut<T>(&mut self) -> Option<RefMut<'_, Object, T>>
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
    pub fn prototype_instance(&self) -> JsValue {
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
    pub fn set_prototype_instance(&self, prototype: JsValue) -> bool {
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
        value: &JsValue,
    ) -> JsResult<bool> {
        // 1. If IsCallable(C) is false, return false.
        if !self.is_callable() {
            return Ok(false);
        }

        // TODO: 2. If C has a [[BoundTargetFunction]] internal slot, then
        //         a. Let BC be C.[[BoundTargetFunction]].
        //         b.  Return ? InstanceofOperator(O, BC).

        // 3. If Type(O) is not Object, return false.
        if let Some(object) = value.as_object() {
            // 4. Let P be ? Get(C, "prototype").
            // 5. If Type(P) is not Object, throw a TypeError exception.
            if let Some(prototype) = self.get("prototype", context)?.as_object() {
                // 6. Repeat,
                //      a. Set O to ? O.[[GetPrototypeOf]]().
                //      b. If O is null, return false.
                let mut object = object.__get_prototype_of__(context)?;
                while let Some(object_prototype) = object.as_object() {
                    //     c. If SameValue(P, O) is true, return true.
                    if JsObject::equals(&prototype, &object_prototype) {
                        return Ok(true);
                    }
                    // a. Set O to ? O.[[GetPrototypeOf]]().
                    object = object_prototype.__get_prototype_of__(context)?;
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

    pub fn to_property_descriptor(&self, context: &mut Context) -> JsResult<PropertyDescriptor> {
        // 1 is implemented on the method `to_property_descriptor` of value

        // 2. Let desc be a new Property Descriptor that initially has no fields.
        let mut desc = PropertyDescriptor::builder();

        // 3. Let hasEnumerable be ? HasProperty(Obj, "enumerable").
        // 4. If hasEnumerable is true, then ...
        if self.has_property("enumerable", context)? {
            // a. Let enumerable be ! ToBoolean(? Get(Obj, "enumerable")).
            // b. Set desc.[[Enumerable]] to enumerable.
            desc = desc.enumerable(self.get("enumerable", context)?.to_boolean());
        }

        // 5. Let hasConfigurable be ? HasProperty(Obj, "configurable").
        // 6. If hasConfigurable is true, then ...
        if self.has_property("configurable", context)? {
            // a. Let configurable be ! ToBoolean(? Get(Obj, "configurable")).
            // b. Set desc.[[Configurable]] to configurable.
            desc = desc.configurable(self.get("configurable", context)?.to_boolean());
        }

        // 7. Let hasValue be ? HasProperty(Obj, "value").
        // 8. If hasValue is true, then ...
        if self.has_property("value", context)? {
            // a. Let value be ? Get(Obj, "value").
            // b. Set desc.[[Value]] to value.
            desc = desc.value(self.get("value", context)?);
        }

        // 9. Let hasWritable be ? HasProperty(Obj, ).
        // 10. If hasWritable is true, then ...
        if self.has_property("writable", context)? {
            // a. Let writable be ! ToBoolean(? Get(Obj, "writable")).
            // b. Set desc.[[Writable]] to writable.
            desc = desc.writable(self.get("writable", context)?.to_boolean());
        }

        // 11. Let hasGet be ? HasProperty(Obj, "get").
        // 12. If hasGet is true, then
        let get = if self.has_property("get", context)? {
            // a. Let getter be ? Get(Obj, "get").
            let getter = self.get("get", context)?;
            // b. If IsCallable(getter) is false and getter is not undefined, throw a TypeError exception.
            // todo: extract IsCallable to be callable from Value
            if !getter.is_undefined() && getter.as_object().map_or(true, |o| !o.is_callable()) {
                return Err(
                    context.construct_type_error("Property descriptor getter must be callable")
                );
            }
            // c. Set desc.[[Get]] to getter.
            Some(getter)
        } else {
            None
        };

        // 13. Let hasSet be ? HasProperty(Obj, "set").
        // 14. If hasSet is true, then
        let set = if self.has_property("set", context)? {
            // 14.a. Let setter be ? Get(Obj, "set").
            let setter = self.get("set", context)?;
            // 14.b. If IsCallable(setter) is false and setter is not undefined, throw a TypeError exception.
            // todo: extract IsCallable to be callable from Value
            if !setter.is_undefined() && setter.as_object().map_or(true, |o| !o.is_callable()) {
                return Err(
                    context.construct_type_error("Property descriptor setter must be callable")
                );
            }
            // 14.c. Set desc.[[Set]] to setter.
            Some(setter)
        } else {
            None
        };

        // 15. If desc.[[Get]] is present or desc.[[Set]] is present, then ...
        // a. If desc.[[Value]] is present or desc.[[Writable]] is present, throw a TypeError exception.
        if get.as_ref().or_else(|| set.as_ref()).is_some() && desc.inner().is_data_descriptor() {
            return Err(context.construct_type_error(
                "Invalid property descriptor.\
            Cannot both specify accessors and a value or writable attribute",
            ));
        }

        desc = desc.maybe_get(get).maybe_set(set);

        // 16. Return desc.
        Ok(desc.build())
    }

    /// `7.3.25 CopyDataProperties ( target, source, excludedItems )`
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-copydataproperties
    #[inline]
    pub fn copy_data_properties<K>(
        &mut self,
        source: &JsValue,
        excluded_keys: Vec<K>,
        context: &mut Context,
    ) -> JsResult<()>
    where
        K: Into<PropertyKey>,
    {
        // 1. Assert: Type(target) is Object.
        // 2. Assert: excludedItems is a List of property keys.
        // 3. If source is undefined or null, return target.
        if source.is_null_or_undefined() {
            return Ok(());
        }

        // 4. Let from be ! ToObject(source).
        let from = source
            .to_object(context)
            .expect("function ToObject should never complete abruptly here");

        // 5. Let keys be ? from.[[OwnPropertyKeys]]().
        // 6. For each element nextKey of keys, do
        let excluded_keys: Vec<PropertyKey> = excluded_keys.into_iter().map(|e| e.into()).collect();
        for key in from.__own_property_keys__(context)? {
            // a. Let excluded be false.
            let mut excluded = false;

            // b. For each element e of excludedItems, do
            for e in &excluded_keys {
                // i. If SameValue(e, nextKey) is true, then
                if *e == key {
                    // 1. Set excluded to true.
                    excluded = true;
                    break;
                }
            }
            // c. If excluded is false, then
            if !excluded {
                // i. Let desc be ? from.[[GetOwnProperty]](nextKey).
                let desc = from.__get_own_property__(&key, context)?;

                // ii. If desc is not undefined and desc.[[Enumerable]] is true, then
                if let Some(desc) = desc {
                    if let Some(enumerable) = desc.enumerable() {
                        if enumerable {
                            // 1. Let propValue be ? Get(from, nextKey).
                            let prop_value = from.__get__(&key, from.clone().into(), context)?;

                            // 2. Perform ! CreateDataPropertyOrThrow(target, nextKey, propValue).
                            self.create_data_property_or_throw(key, prop_value, context)
                                .expect(
                                    "CreateDataPropertyOrThrow should never complete abruptly here",
                                );
                        }
                    }
                }
            }
        }

        // 7. Return target.
        Ok(())
    }

    /// Helper function for property insertion.
    #[inline]
    #[track_caller]
    pub(crate) fn insert<K, P>(&self, key: K, property: P) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.borrow_mut().insert(key, property)
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// If a field was already in the object with the same name that a `Some` is returned
    /// with that field, otherwise None is returned.
    #[inline]
    pub fn insert_property<K, P>(&self, key: K, property: P) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.insert(key.into(), property)
    }

    /// It determines if Object is a callable function with a `[[Call]]` internal method.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iscallable
    #[inline]
    #[track_caller]
    pub fn is_callable(&self) -> bool {
        self.borrow().is_callable()
    }

    /// It determines if Object is a function object with a `[[Construct]]` internal method.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isconstructor
    #[inline]
    #[track_caller]
    pub fn is_constructable(&self) -> bool {
        self.borrow().is_constructable()
    }

    /// Returns true if the GcObject is the global for a Realm
    pub fn is_global(&self) -> bool {
        matches!(
            self.borrow().data,
            ObjectData {
                kind: ObjectKind::Global,
                ..
            }
        )
    }
}

impl AsRef<GcCell<Object>> for JsObject {
    #[inline]
    fn as_ref(&self) -> &GcCell<Object> {
        &*self.0
    }
}

/// An error returned by [`JsObject::try_borrow`](struct.JsObject.html#method.try_borrow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowError;

impl Display for BorrowError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Object already mutably borrowed", f)
    }
}

impl Error for BorrowError {}

/// An error returned by [`JsObject::try_borrow_mut`](struct.JsObject.html#method.try_borrow_mut).
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
    ///    b,            // state for b's &JsObject here is None
    ///    b,            // state for b's &JsObject here is Visited
    /// ]);
    /// ```
    Visited,
}

/// Prevents infinite recursion during `Debug::fmt`, `JSON.stringify`, and other conversions.
/// This uses a thread local, so is not safe to use where the object graph will be traversed by
/// multiple threads!
#[derive(Debug)]
pub struct RecursionLimiter {
    /// If this was the first `JsObject` in the tree.
    top_level: bool,
    /// The ptr being kept in the HashSet, so we can delete it when we drop.
    ptr: usize,
    /// If this JsObject has been visited before in the graph, but not in the current branch.
    pub visited: bool,
    /// If this JsObject has been visited in the current branch of the graph.
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
        /// The map of pointers to `JsObject` that have been visited during the current `Debug::fmt` graph,
        /// and the current state of their RecursionLimiter (dropped or live -- see `RecursionValueState`)
        static SEEN: RefCell<HashMap<usize, RecursionValueState>> = RefCell::new(HashMap::new());
    }

    /// Determines if the specified `JsObject` has been visited, and returns a struct that will free it when dropped.
    ///
    /// This is done by maintaining a thread-local hashset containing the pointers of `JsObject` values that have been
    /// visited. The first `JsObject` visited will clear the hashset, while any others will check if they are contained
    /// by the hashset.
    pub fn new(o: &JsObject) -> Self {
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

impl Debug for JsObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let limiter = RecursionLimiter::new(self);

        // Typically, using `!limiter.live` would be good enough here.
        // However, the JS object hierarchy involves quite a bit of repitition, and the sheer amount of data makes
        // understanding the Debug output impossible; limiting the usefulness of it.
        //
        // Instead, we check if the object has appeared before in the entire graph. This means that objects will appear
        // at most once, hopefully making things a bit clearer.
        if !limiter.visited && !limiter.live {
            f.debug_tuple("JsObject").field(&self.0).finish()
        } else {
            f.write_str("{ ... }")
        }
    }
}
