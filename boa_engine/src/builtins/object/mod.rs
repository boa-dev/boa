//! This module implements the global `Object` object.
//!
//! The `Object` class represents one of JavaScript's data types.
//!
//! It is used to store various keyed collections and more complex entities.
//! Objects can be created using the `Object()` constructor or the
//! object initializer / literal syntax.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object

use std::ops::Deref;

use super::Array;
use crate::{
    builtins::{map, BuiltIn, JsArgs},
    context::intrinsics::StandardConstructors,
    error::JsNativeError,
    js_string,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        IntegrityLevel, JsObject, ObjectData, ObjectKind,
    },
    property::{Attribute, PropertyDescriptor, PropertyKey, PropertyNameKind},
    string::utf16,
    symbol::WellKnownSymbols,
    value::JsValue,
    Context, JsResult, JsString,
};
use boa_profiler::Profiler;
use tap::{Conv, Pipe};

pub mod for_in_iterator;
#[cfg(test)]
mod tests;

/// The global JavaScript object.
#[derive(Debug, Clone, Copy)]
pub struct Object;

impl BuiltIn for Object {
    const NAME: &'static str = "Object";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let legacy_proto_getter = FunctionBuilder::native(context, Self::legacy_proto_getter)
            .name("get __proto__")
            .build();

        let legacy_setter_proto = FunctionBuilder::native(context, Self::legacy_proto_setter)
            .name("set __proto__")
            .build();

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .inherit(None)
        .accessor(
            "__proto__",
            Some(legacy_proto_getter),
            Some(legacy_setter_proto),
            Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .method(Self::has_own_property, "hasOwnProperty", 1)
        .method(Self::property_is_enumerable, "propertyIsEnumerable", 1)
        .method(Self::to_string, "toString", 0)
        .method(Self::to_locale_string, "toLocaleString", 0)
        .method(Self::value_of, "valueOf", 0)
        .method(Self::is_prototype_of, "isPrototypeOf", 1)
        .method(Self::legacy_define_getter, "__defineGetter__", 2)
        .method(Self::legacy_define_setter, "__defineSetter__", 2)
        .method(Self::legacy_lookup_getter, "__lookupGetter__", 1)
        .method(Self::legacy_lookup_setter, "__lookupSetter__", 1)
        .static_method(Self::create, "create", 2)
        .static_method(Self::set_prototype_of, "setPrototypeOf", 2)
        .static_method(Self::get_prototype_of, "getPrototypeOf", 1)
        .static_method(Self::define_property, "defineProperty", 3)
        .static_method(Self::define_properties, "defineProperties", 2)
        .static_method(Self::assign, "assign", 2)
        .static_method(Self::is, "is", 2)
        .static_method(Self::keys, "keys", 1)
        .static_method(Self::values, "values", 1)
        .static_method(Self::entries, "entries", 1)
        .static_method(Self::seal, "seal", 1)
        .static_method(Self::is_sealed, "isSealed", 1)
        .static_method(Self::freeze, "freeze", 1)
        .static_method(Self::is_frozen, "isFrozen", 1)
        .static_method(Self::prevent_extensions, "preventExtensions", 1)
        .static_method(Self::is_extensible, "isExtensible", 1)
        .static_method(
            Self::get_own_property_descriptor,
            "getOwnPropertyDescriptor",
            2,
        )
        .static_method(
            Self::get_own_property_descriptors,
            "getOwnPropertyDescriptors",
            1,
        )
        .static_method(Self::get_own_property_names, "getOwnPropertyNames", 1)
        .static_method(Self::get_own_property_symbols, "getOwnPropertySymbols", 1)
        .static_method(Self::has_own, "hasOwn", 2)
        .static_method(Self::from_entries, "fromEntries", 1)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl Object {
    const LENGTH: usize = 1;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if !new_target.is_undefined() {
            let prototype =
                get_prototype_from_constructor(new_target, StandardConstructors::object, context)?;
            let object = JsObject::from_proto_and_data(prototype, ObjectData::ordinary());
            return Ok(object.into());
        }
        if let Some(arg) = args.get(0) {
            if !arg.is_null_or_undefined() {
                return Ok(arg.to_object(context)?.into());
            }
        }
        Ok(context.construct_object().into())
    }

    /// `get Object.prototype.__proto__`
    ///
    /// The `__proto__` getter function exposes the value of the
    /// internal `[[Prototype]]` of an object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-object.prototype.__proto__
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/proto
    pub fn legacy_proto_getter(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? ToObject(this value).
        let obj = this.to_object(context)?;

        // 2. Return ? O.[[GetPrototypeOf]]().
        let proto = obj.__get_prototype_of__(context)?;

        Ok(proto.map_or(JsValue::Null, JsValue::new))
    }

    /// `set Object.prototype.__proto__`
    ///
    /// The `__proto__` setter allows the `[[Prototype]]` of
    /// an object to be mutated.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set-object.prototype.__proto__
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/proto
    pub fn legacy_proto_setter(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. If Type(proto) is neither Object nor Null, return undefined.
        let proto = match args.get_or_undefined(0) {
            JsValue::Object(proto) => Some(proto.clone()),
            JsValue::Null => None,
            _ => return Ok(JsValue::undefined()),
        };

        // 3. If Type(O) is not Object, return undefined.
        let object = match this {
            JsValue::Object(object) => object,
            _ => return Ok(JsValue::undefined()),
        };

        // 4. Let status be ? O.[[SetPrototypeOf]](proto).
        let status = object.__set_prototype_of__(proto, context)?;

        // 5. If status is false, throw a TypeError exception.
        if !status {
            return Err(JsNativeError::typ()
                .with_message("__proto__ called on null or undefined")
                .into());
        }

        // 6. Return undefined.
        Ok(JsValue::undefined())
    }

    /// `Object.prototype.__defineGetter__(prop, func)`
    ///
    /// Binds an object's property to a function to be called when that property is looked up.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.__defineGetter__
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/__defineGetter__
    pub fn legacy_define_getter(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let getter = args.get_or_undefined(1);

        // 1. Let O be ? ToObject(this value).
        let obj = this.to_object(context)?;

        // 2. If IsCallable(getter) is false, throw a TypeError exception.
        if !getter.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("Object.prototype.__defineGetter__: Expecting function")
                .into());
        }

        // 3. Let desc be PropertyDescriptor { [[Get]]: getter, [[Enumerable]]: true, [[Configurable]]: true }.
        let desc = PropertyDescriptor::builder()
            .get(getter)
            .enumerable(true)
            .configurable(true);

        // 4. Let key be ? ToPropertyKey(P).
        let key = args.get_or_undefined(0).to_property_key(context)?;

        // 5. Perform ? DefinePropertyOrThrow(O, key, desc).
        obj.define_property_or_throw(key, desc, context)?;

        // 6. Return undefined.
        Ok(JsValue::undefined())
    }

    /// `Object.prototype.__defineSetter__(prop, func)`
    ///
    /// Binds an object's property to a function to be called when an attempt is made to set that property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.__defineSetter__
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/__defineSetter__
    pub fn legacy_define_setter(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let setter = args.get_or_undefined(1);

        // 1. Let O be ? ToObject(this value).
        let obj = this.to_object(context)?;

        // 2. If IsCallable(setter) is false, throw a TypeError exception.
        if !setter.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("Object.prototype.__defineSetter__: Expecting function")
                .into());
        }

        // 3. Let desc be PropertyDescriptor { [[Set]]: setter, [[Enumerable]]: true, [[Configurable]]: true }.
        let desc = PropertyDescriptor::builder()
            .set(setter)
            .enumerable(true)
            .configurable(true);

        // 4. Let key be ? ToPropertyKey(P).
        let key = args.get_or_undefined(0).to_property_key(context)?;

        // 5. Perform ? DefinePropertyOrThrow(O, key, desc).
        obj.define_property_or_throw(key, desc, context)?;

        // 6. Return undefined.
        Ok(JsValue::undefined())
    }

    /// `Object.prototype.__lookupGetter__(prop)`
    ///
    /// Returns the function bound as a getter to the specified property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.__lookupGetter__
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/__lookupGetter__
    pub fn legacy_lookup_getter(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? ToObject(this value).
        let mut obj = this.to_object(context)?;

        // 2. Let key be ? ToPropertyKey(P).
        let key = args.get_or_undefined(0).to_property_key(context)?;

        // 3. Repeat
        loop {
            // a. Let desc be ? O.[[GetOwnProperty]](key).
            let desc = obj.__get_own_property__(&key, context)?;

            // b. If desc is not undefined, then
            if let Some(current_desc) = desc {
                // i. If IsAccessorDescriptor(desc) is true, return desc.[[Get]].
                return if current_desc.is_accessor_descriptor() {
                    Ok(current_desc.expect_get().into())
                } else {
                    // ii. Return undefined.
                    Ok(JsValue::undefined())
                };
            }
            match obj.__get_prototype_of__(context)? {
                // c. Set O to ? O.[[GetPrototypeOf]]().
                Some(o) => obj = o,
                // d. If O is null, return undefined.
                None => return Ok(JsValue::undefined()),
            }
        }
    }
    /// `Object.prototype.__lookupSetter__(prop)`
    ///
    /// Returns the function bound as a getter to the specified property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.__lookupSetter__
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/__lookupSetter__
    pub fn legacy_lookup_setter(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? ToObject(this value).
        let mut obj = this.to_object(context)?;

        // 2. Let key be ? ToPropertyKey(P).
        let key = args.get_or_undefined(0).to_property_key(context)?;

        // 3. Repeat
        loop {
            // a. Let desc be ? O.[[GetOwnProperty]](key).
            let desc = obj.__get_own_property__(&key, context)?;

            // b. If desc is not undefined, then
            if let Some(current_desc) = desc {
                // i. If IsAccessorDescriptor(desc) is true, return desc.[[Set]].
                return if current_desc.is_accessor_descriptor() {
                    Ok(current_desc.expect_set().into())
                } else {
                    // ii. Return undefined.
                    Ok(JsValue::undefined())
                };
            }
            match obj.__get_prototype_of__(context)? {
                // c. Set O to ? O.[[GetPrototypeOf]]().
                Some(o) => obj = o,
                // d. If O is null, return undefined.
                None => return Ok(JsValue::undefined()),
            }
        }
    }

    /// `Object.create( proto, [propertiesObject] )`
    ///
    /// Creates a new object from the provided prototype.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.create
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/create
    pub fn create(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let prototype = args.get_or_undefined(0);
        let properties = args.get_or_undefined(1);

        let obj = match prototype {
            JsValue::Object(_) | JsValue::Null => JsObject::from_proto_and_data(
                prototype.as_object().cloned(),
                ObjectData::ordinary(),
            ),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message(format!(
                        "Object prototype may only be an Object or null: {}",
                        prototype.display()
                    ))
                    .into())
            }
        };

        if !properties.is_undefined() {
            object_define_properties(&obj, properties, context)?;
            return Ok(obj.into());
        }

        Ok(obj.into())
    }

    /// `Object.getOwnPropertyDescriptor( object, property )`
    ///
    /// Returns an object describing the configuration of a specific property on a given object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.getownpropertydescriptor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getOwnPropertyDescriptor
    pub fn get_own_property_descriptor(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let obj be ? ToObject(O).
        let obj = args.get_or_undefined(0).to_object(context)?;

        // 2. Let key be ? ToPropertyKey(P).
        let key = args.get_or_undefined(1).to_property_key(context)?;

        // 3. Let desc be ? obj.[[GetOwnProperty]](key).
        let desc = obj.__get_own_property__(&key, context)?;

        // 4. Return FromPropertyDescriptor(desc).
        Ok(Self::from_property_descriptor(desc, context))
    }

    /// `Object.getOwnPropertyDescriptors( object )`
    ///
    /// Returns all own property descriptors of a given object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.getownpropertydescriptors
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getOwnPropertyDescriptors
    pub fn get_own_property_descriptors(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let obj be ? ToObject(O).
        let obj = args.get_or_undefined(0).to_object(context)?;

        // 2. Let ownKeys be ? obj.[[OwnPropertyKeys]]().
        let own_keys = obj.__own_property_keys__(context)?;

        // 3. Let descriptors be OrdinaryObjectCreate(%Object.prototype%).
        let descriptors = context.construct_object();

        // 4. For each element key of ownKeys, do
        for key in own_keys {
            // a. Let desc be ? obj.[[GetOwnProperty]](key).
            let desc = obj.__get_own_property__(&key, context)?;

            // b. Let descriptor be FromPropertyDescriptor(desc).
            let descriptor = Self::from_property_descriptor(desc, context);

            // c. If descriptor is not undefined,
            //    perform ! CreateDataPropertyOrThrow(descriptors, key, descriptor).
            if !descriptor.is_undefined() {
                descriptors
                    .create_data_property_or_throw(key, descriptor, context)
                    .expect("should not fail according to spec");
            }
        }

        // 5. Return descriptors.
        Ok(descriptors.into())
    }

    /// The abstract operation `FromPropertyDescriptor`.
    ///
    /// [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-frompropertydescriptor
    pub(crate) fn from_property_descriptor(
        desc: Option<PropertyDescriptor>,
        context: &mut Context,
    ) -> JsValue {
        match desc {
            // 1. If Desc is undefined, return undefined.
            None => JsValue::undefined(),
            Some(desc) => {
                // 2. Let obj be ! OrdinaryObjectCreate(%Object.prototype%).
                // 3. Assert: obj is an extensible ordinary object with no own properties.
                let obj = context.construct_object();

                // 4. If Desc has a [[Value]] field, then
                if let Some(value) = desc.value() {
                    // a. Perform ! CreateDataPropertyOrThrow(obj, "value", Desc.[[Value]]).
                    obj.create_data_property_or_throw("value", value, context)
                        .expect("CreateDataPropertyOrThrow cannot fail here");
                }

                // 5. If Desc has a [[Writable]] field, then
                if let Some(writable) = desc.writable() {
                    // a. Perform ! CreateDataPropertyOrThrow(obj, "writable", Desc.[[Writable]]).
                    obj.create_data_property_or_throw("writable", writable, context)
                        .expect("CreateDataPropertyOrThrow cannot fail here");
                }

                // 6. If Desc has a [[Get]] field, then
                if let Some(get) = desc.get() {
                    // a. Perform ! CreateDataPropertyOrThrow(obj, "get", Desc.[[Get]]).
                    obj.create_data_property_or_throw("get", get, context)
                        .expect("CreateDataPropertyOrThrow cannot fail here");
                }

                // 7. If Desc has a [[Set]] field, then
                if let Some(set) = desc.set() {
                    // a. Perform ! CreateDataPropertyOrThrow(obj, "set", Desc.[[Set]]).
                    obj.create_data_property_or_throw("set", set, context)
                        .expect("CreateDataPropertyOrThrow cannot fail here");
                }

                // 8. If Desc has an [[Enumerable]] field, then
                if let Some(enumerable) = desc.enumerable() {
                    // a. Perform ! CreateDataPropertyOrThrow(obj, "enumerable", Desc.[[Enumerable]]).
                    obj.create_data_property_or_throw("enumerable", enumerable, context)
                        .expect("CreateDataPropertyOrThrow cannot fail here");
                }

                // 9. If Desc has a [[Configurable]] field, then
                if let Some(configurable) = desc.configurable() {
                    // a. Perform ! CreateDataPropertyOrThrow(obj, "configurable", Desc.[[Configurable]]).
                    obj.create_data_property_or_throw("configurable", configurable, context)
                        .expect("CreateDataPropertyOrThrow cannot fail here");
                }

                // 10. Return obj.
                obj.into()
            }
        }
    }

    /// Uses the `SameValue` algorithm to check equality of objects
    pub fn is(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let x = args.get_or_undefined(0);
        let y = args.get_or_undefined(1);

        Ok(JsValue::same_value(x, y).into())
    }

    /// Get the `prototype` of an object.
    ///
    /// [More information][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.setprototypeof
    pub fn get_prototype_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if args.is_empty() {
            return Err(JsNativeError::typ()
                .with_message(
                    "Object.getPrototypeOf: At least 1 argument required, but only 0 passed",
                )
                .into());
        }

        // 1. Let obj be ? ToObject(O).
        let obj = args[0].clone().to_object(context)?;

        // 2. Return ? obj.[[GetPrototypeOf]]().
        Ok(obj
            .__get_prototype_of__(context)?
            .map_or(JsValue::Null, JsValue::new))
    }

    /// Set the `prototype` of an object.
    ///
    /// [More information][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.setprototypeof
    pub fn set_prototype_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if args.len() < 2 {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "Object.setPrototypeOf: At least 2 arguments required, but only {} passed",
                    args.len()
                ))
                .into());
        }

        // 1. Set O to ? RequireObjectCoercible(O).
        let o = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .require_object_coercible()?
            .clone();

        let proto = match args.get_or_undefined(1) {
            JsValue::Object(obj) => Some(obj.clone()),
            JsValue::Null => None,
            // 2. If Type(proto) is neither Object nor Null, throw a TypeError exception.
            val => {
                return Err(JsNativeError::typ()
                    .with_message(format!(
                        "expected an object or null, got `{}`",
                        val.type_of()
                    ))
                    .into())
            }
        };

        let Some(obj) = o.as_object() else {
            // 3. If Type(O) is not Object, return O.
            return Ok(o);
        };

        // 4. Let status be ? O.[[SetPrototypeOf]](proto).
        let status = obj.__set_prototype_of__(proto, context)?;

        // 5. If status is false, throw a TypeError exception.
        if !status {
            return Err(JsNativeError::typ()
                .with_message("can't set prototype of this object")
                .into());
        }

        // 6. Return O.
        Ok(o)
    }

    /// `Object.prototype.isPrototypeOf( proto )`
    ///
    /// Check whether or not an object exists within another object's prototype chain.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.isprototypeof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/isPrototypeOf
    pub fn is_prototype_of(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let v = args.get_or_undefined(0);
        if !v.is_object() {
            return Ok(JsValue::new(false));
        }
        let mut v = v.clone();
        let o = JsValue::new(this.to_object(context)?);
        loop {
            v = Self::get_prototype_of(this, &[v], context)?;
            if v.is_null() {
                return Ok(JsValue::new(false));
            }
            if JsValue::same_value(&o, &v) {
                return Ok(JsValue::new(true));
            }
        }
    }

    /// Define a property in an object
    pub fn define_property(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = args.get_or_undefined(0);
        if let JsValue::Object(object) = object {
            let key = args
                .get(1)
                .unwrap_or(&JsValue::Undefined)
                .to_property_key(context)?;
            let desc = args
                .get(2)
                .unwrap_or(&JsValue::Undefined)
                .to_property_descriptor(context)?;

            object.define_property_or_throw(key, desc, context)?;

            Ok(object.clone().into())
        } else {
            Err(JsNativeError::typ()
                .with_message("Object.defineProperty called on non-object")
                .into())
        }
    }

    /// `Object.defineProperties( proto, [propertiesObject] )`
    ///
    /// Creates or update own properties to the object
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.defineproperties
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/defineProperties
    pub fn define_properties(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let arg = args.get_or_undefined(0);
        if let JsValue::Object(obj) = arg {
            let props = args.get_or_undefined(1);
            object_define_properties(obj, props, context)?;
            Ok(arg.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("Expected an object")
                .into())
        }
    }

    /// `Object.prototype.valueOf()`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.valueof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/valueOf
    pub fn value_of(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Return ? ToObject(this value).
        Ok(this.to_object(context)?.into())
    }

    /// `Object.prototype.toString()`
    ///
    /// This method returns a string representing the object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.tostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/toString
    #[allow(clippy::wrong_self_convention)]
    pub fn to_string(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. If the this value is undefined, return "[object Undefined]".
        if this.is_undefined() {
            return Ok("[object Undefined]".into());
        }
        // 2. If the this value is null, return "[object Null]".
        if this.is_null() {
            return Ok("[object Null]".into());
        }
        // 3. Let O be ! ToObject(this value).
        let o = this.to_object(context).expect("toObject cannot fail here");

        //  4. Let isArray be ? IsArray(O).
        //  5. If isArray is true, let builtinTag be "Array".
        let builtin_tag = if o.is_array_abstract()? {
            utf16!("Array")
        } else {
            // 6. Else if O has a [[ParameterMap]] internal slot, let builtinTag be "Arguments".
            // 7. Else if O has a [[Call]] internal method, let builtinTag be "Function".
            // 8. Else if O has an [[ErrorData]] internal slot, let builtinTag be "Error".
            // 9. Else if O has a [[BooleanData]] internal slot, let builtinTag be "Boolean".
            // 10. Else if O has a [[NumberData]] internal slot, let builtinTag be "Number".
            // 11. Else if O has a [[StringData]] internal slot, let builtinTag be "String".
            // 12. Else if O has a [[DateValue]] internal slot, let builtinTag be "Date".
            // 13. Else if O has a [[RegExpMatcher]] internal slot, let builtinTag be "RegExp".
            // 14. Else, let builtinTag be "Object".
            let o = o.borrow();
            match o.kind() {
                ObjectKind::Arguments(_) => utf16!("Arguments"),
                ObjectKind::Function(_) => utf16!("Function"),
                ObjectKind::Error(_) => utf16!("Error"),
                ObjectKind::Boolean(_) => utf16!("Boolean"),
                ObjectKind::Number(_) => utf16!("Number"),
                ObjectKind::String(_) => utf16!("String"),
                ObjectKind::Date(_) => utf16!("Date"),
                ObjectKind::RegExp(_) => utf16!("RegExp"),
                _ => utf16!("Object"),
            }
        };

        // 15. Let tag be ? Get(O, @@toStringTag).
        let tag = o.get(WellKnownSymbols::to_string_tag(), context)?;

        // 16. If Type(tag) is not String, set tag to builtinTag.
        let tag_str = tag.as_string().map_or(builtin_tag, JsString::deref);

        // 17. Return the string-concatenation of "[object ", tag, and "]".
        Ok(js_string!(utf16!("[object "), tag_str, utf16!("]")).into())
    }

    /// `Object.prototype.toLocaleString( [ reserved1 [ , reserved2 ] ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.tolocalestring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/toLocaleString
    #[allow(clippy::wrong_self_convention)]
    pub fn to_locale_string(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Return ? Invoke(O, "toString").
        this.invoke("toString", &[], context)
    }

    /// `Object.prototype.hasOwnProperty( property )`
    ///
    /// The method returns a boolean indicating whether the object has the specified property
    /// as its own property (as opposed to inheriting it).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.hasownproperty
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/hasOwnProperty
    pub fn has_own_property(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let P be ? ToPropertyKey(V).
        let key = args.get_or_undefined(0).to_property_key(context)?;

        // 2. Let O be ? ToObject(this value).
        let object = this.to_object(context)?;

        // 3. Return ? HasOwnProperty(O, P).
        Ok(object.has_own_property(key, context)?.into())
    }

    /// `Object.prototype.propertyIsEnumerable( property )`
    ///
    /// This method returns a Boolean indicating whether the specified property is
    /// enumerable and is the object's own property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.propertyisenumerable
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/propertyIsEnumerable
    pub fn property_is_enumerable(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let key = match args.get(0) {
            None => return Ok(JsValue::new(false)),
            Some(key) => key,
        };

        let key = key.to_property_key(context)?;
        let own_prop = this
            .to_object(context)?
            .__get_own_property__(&key, context)?;

        own_prop
            .as_ref()
            .and_then(PropertyDescriptor::enumerable)
            .unwrap_or_default()
            .conv::<JsValue>()
            .pipe(Ok)
    }

    /// `Object.assign( target, ...sources )`
    ///
    /// This method copies all enumerable own properties from one or more
    /// source objects to a target object. It returns the target object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.assign
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/assign
    pub fn assign(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let to be ? ToObject(target).
        let to = args.get_or_undefined(0).to_object(context)?;

        // 2. If only one argument was passed, return to.
        if args.len() == 1 {
            return Ok(to.into());
        }

        // 3. For each element nextSource of sources, do
        for source in &args[1..] {
            // 3.a. If nextSource is neither undefined nor null, then
            if !source.is_null_or_undefined() {
                // 3.a.i. Let from be ! ToObject(nextSource).
                let from = source
                    .to_object(context)
                    .expect("this ToObject call must not fail");
                // 3.a.ii. Let keys be ? from.[[OwnPropertyKeys]]().
                let keys = from.__own_property_keys__(context)?;
                // 3.a.iii. For each element nextKey of keys, do
                for key in keys {
                    // 3.a.iii.1. Let desc be ? from.[[GetOwnProperty]](nextKey).
                    if let Some(desc) = from.__get_own_property__(&key, context)? {
                        // 3.a.iii.2. If desc is not undefined and desc.[[Enumerable]] is true, then
                        if desc.expect_enumerable() {
                            // 3.a.iii.2.a. Let propValue be ? Get(from, nextKey).
                            let property = from.get(key.clone(), context)?;
                            // 3.a.iii.2.b. Perform ? Set(to, nextKey, propValue, true).
                            to.set(key, property, true, context)?;
                        }
                    }
                }
            }
        }

        // 4. Return to.
        Ok(to.into())
    }

    /// `Object.keys( target )`
    ///
    /// This method returns an array of a given object's own enumerable
    /// property names, iterated in the same order that a normal loop would.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.keys
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/keys
    pub fn keys(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let obj be ? ToObject(target).
        let obj = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_object(context)?;

        // 2. Let nameList be ? EnumerableOwnPropertyNames(obj, key).
        let name_list = obj.enumerable_own_property_names(PropertyNameKind::Key, context)?;

        // 3. Return CreateArrayFromList(nameList).
        let result = Array::create_array_from_list(name_list, context);

        Ok(result.into())
    }

    /// `Object.values( target )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.values
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/values
    pub fn values(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let obj be ? ToObject(target).
        let obj = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_object(context)?;

        // 2. Let nameList be ? EnumerableOwnPropertyNames(obj, value).
        let name_list = obj.enumerable_own_property_names(PropertyNameKind::Value, context)?;

        // 3. Return CreateArrayFromList(nameList).
        let result = Array::create_array_from_list(name_list, context);

        Ok(result.into())
    }

    /// `Object.entries( target )`
    ///
    /// This method returns an array of a given object's own enumerable string-keyed property [key, value] pairs.
    /// This is the same as iterating with a for...in loop,
    /// except that a for...in loop enumerates properties in the prototype chain as well).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.entries
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/entries
    pub fn entries(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let obj be ? ToObject(target).
        let obj = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_object(context)?;

        // 2. Let nameList be ? EnumerableOwnPropertyNames(obj, key+value).
        let name_list =
            obj.enumerable_own_property_names(PropertyNameKind::KeyAndValue, context)?;

        // 3. Return CreateArrayFromList(nameList).
        let result = Array::create_array_from_list(name_list, context);

        Ok(result.into())
    }

    /// `Object.seal( target )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.seal
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/seal
    pub fn seal(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = args.get_or_undefined(0);

        if let Some(o) = o.as_object() {
            // 2. Let status be ? SetIntegrityLevel(O, sealed).
            let status = o.set_integrity_level(IntegrityLevel::Sealed, context)?;
            // 3. If status is false, throw a TypeError exception.
            if !status {
                return Err(JsNativeError::typ()
                    .with_message("cannot seal object")
                    .into());
            }
        }
        // 1. If Type(O) is not Object, return O.
        // 4. Return O.
        Ok(o.clone())
    }

    /// `Object.isSealed( target )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.issealed
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/isSealed
    pub fn is_sealed(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = args.get_or_undefined(0);

        // 1. If Type(O) is not Object, return true.
        // 2. Return ? TestIntegrityLevel(O, sealed).
        if let Some(o) = o.as_object() {
            Ok(o.test_integrity_level(IntegrityLevel::Sealed, context)?
                .into())
        } else {
            Ok(JsValue::new(true))
        }
    }

    /// `Object.freeze( target )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.freeze
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/freeze
    pub fn freeze(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = args.get_or_undefined(0);

        if let Some(o) = o.as_object() {
            // 2. Let status be ? SetIntegrityLevel(O, frozen).
            let status = o.set_integrity_level(IntegrityLevel::Frozen, context)?;
            // 3. If status is false, throw a TypeError exception.
            if !status {
                return Err(JsNativeError::typ()
                    .with_message("cannot freeze object")
                    .into());
            }
        }
        // 1. If Type(O) is not Object, return O.
        // 4. Return O.
        Ok(o.clone())
    }

    /// `Object.isFrozen( target )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.isfrozen
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/isFrozen
    pub fn is_frozen(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = args.get_or_undefined(0);

        // 1. If Type(O) is not Object, return true.
        // 2. Return ? TestIntegrityLevel(O, frozen).
        if let Some(o) = o.as_object() {
            Ok(o.test_integrity_level(IntegrityLevel::Frozen, context)?
                .into())
        } else {
            Ok(JsValue::new(true))
        }
    }

    /// `Object.preventExtensions( target )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.preventextensions
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/preventExtensions
    pub fn prevent_extensions(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let o = args.get_or_undefined(0);

        if let Some(o) = o.as_object() {
            // 2. Let status be ? O.[[PreventExtensions]]().
            let status = o.__prevent_extensions__(context)?;
            // 3. If status is false, throw a TypeError exception.
            if !status {
                return Err(JsNativeError::typ()
                    .with_message("cannot prevent extensions")
                    .into());
            }
        }
        // 1. If Type(O) is not Object, return O.
        // 4. Return O.
        Ok(o.clone())
    }

    /// `Object.isExtensible( target )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.isextensible
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/isExtensible
    pub fn is_extensible(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let o = args.get_or_undefined(0);
        // 1. If Type(O) is not Object, return false.
        if let Some(o) = o.as_object() {
            // 2. Return ? IsExtensible(O).
            Ok(o.is_extensible(context)?.into())
        } else {
            Ok(JsValue::new(false))
        }
    }

    /// `Object.getOwnPropertyNames( object )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.getownpropertynames
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getOwnPropertyNames
    pub fn get_own_property_names(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Return ? GetOwnPropertyKeys(O, string).
        let o = args.get_or_undefined(0);
        get_own_property_keys(o, PropertyKeyType::String, context)
    }

    /// `Object.getOwnPropertySymbols( object )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.getownpropertysymbols
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getOwnPropertySymbols
    pub fn get_own_property_symbols(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Return ? GetOwnPropertyKeys(O, symbol).
        let o = args.get_or_undefined(0);
        get_own_property_keys(o, PropertyKeyType::Symbol, context)
    }

    /// `Object.hasOwn( object, property )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.hasown
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/hasOwn
    pub fn has_own(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let obj be ? ToObject(O).
        let obj = args.get_or_undefined(0).to_object(context)?;

        // 2. Let key be ? ToPropertyKey(P).
        let key = args.get_or_undefined(1).to_property_key(context)?;

        // 3. Return ? HasOwnProperty(obj, key).
        Ok(obj.has_own_property(key, context)?.into())
    }

    /// `Object.fromEntries( iterable )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.fromentries
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/fromEntries
    pub fn from_entries(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Perform ? RequireObjectCoercible(iterable).
        let iterable = args.get_or_undefined(0).require_object_coercible()?;

        // 2. Let obj be ! OrdinaryObjectCreate(%Object.prototype%).
        // 3. Assert: obj is an extensible ordinary object with no own properties.
        let obj = context.construct_object();

        // 4. Let closure be a new Abstract Closure with parameters (key, value) that captures
        // obj and performs the following steps when called:
        let closure = FunctionBuilder::closure_with_captures(
            context,
            |_, args, obj, context| {
                let key = args.get_or_undefined(0);
                let value = args.get_or_undefined(1);

                // a. Let propertyKey be ? ToPropertyKey(key).
                let property_key = key.to_property_key(context)?;

                // b. Perform ! CreateDataPropertyOrThrow(obj, propertyKey, value).
                obj.create_data_property_or_throw(property_key, value, context)?;

                // c. Return undefined.
                Ok(JsValue::undefined())
            },
            obj.clone(),
        );

        // 5. Let adder be ! CreateBuiltinFunction(closure, 2, "", « »).
        let adder = closure.length(2).name("").build();

        // 6. Return ? AddEntriesFromIterable(obj, iterable, adder).
        map::add_entries_from_iterable(&obj, iterable, &adder.into(), context)
    }
}

/// The abstract operation `ObjectDefineProperties`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-object.defineproperties
#[inline]
fn object_define_properties(
    object: &JsObject,
    props: &JsValue,
    context: &mut Context,
) -> JsResult<()> {
    // 1. Assert: Type(O) is Object.
    // 2. Let props be ? ToObject(Properties).
    let props = &props.to_object(context)?;

    // 3. Let keys be ? props.[[OwnPropertyKeys]]().
    let keys = props.__own_property_keys__(context)?;

    // 4. Let descriptors be a new empty List.
    let mut descriptors: Vec<(PropertyKey, PropertyDescriptor)> = Vec::new();

    // 5. For each element nextKey of keys, do
    for next_key in keys {
        // a. Let propDesc be ? props.[[GetOwnProperty]](nextKey).
        // b. If propDesc is not undefined and propDesc.[[Enumerable]] is true, then
        if let Some(prop_desc) = props.__get_own_property__(&next_key, context)? {
            if prop_desc.expect_enumerable() {
                // i. Let descObj be ? Get(props, nextKey).
                let desc_obj = props.get(next_key.clone(), context)?;

                // ii. Let desc be ? ToPropertyDescriptor(descObj).
                let desc = desc_obj.to_property_descriptor(context)?;

                // iii. Append the pair (a two element List) consisting of nextKey and desc to the end of descriptors.
                descriptors.push((next_key, desc));
            }
        }
    }

    // 6. For each element pair of descriptors, do
    // a. Let P be the first element of pair.
    // b. Let desc be the second element of pair.
    for (p, d) in descriptors {
        // c. Perform ? DefinePropertyOrThrow(O, P, desc).
        object.define_property_or_throw(p, d, context)?;
    }

    // 7. Return O.
    Ok(())
}

/// Type enum used in the abstract operation `GetOwnPropertyKeys`.
#[derive(Debug, Copy, Clone)]
enum PropertyKeyType {
    String,
    Symbol,
}

/// The abstract operation `GetOwnPropertyKeys`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-getownpropertykeys
fn get_own_property_keys(
    o: &JsValue,
    r#type: PropertyKeyType,
    context: &mut Context,
) -> JsResult<JsValue> {
    // 1. Let obj be ? ToObject(o).
    let obj = o.to_object(context)?;

    // 2. Let keys be ? obj.[[OwnPropertyKeys]]().
    let keys = obj.__own_property_keys__(context)?;

    // 3. Let nameList be a new empty List.
    // 4. For each element nextKey of keys, do
    let name_list = keys.iter().filter_map(|next_key| {
        // a. If Type(nextKey) is Symbol and type is symbol or Type(nextKey) is String and type is string, then
        // i. Append nextKey as the last element of nameList.
        match (r#type, &next_key) {
            (PropertyKeyType::String, PropertyKey::String(_))
            | (PropertyKeyType::Symbol, PropertyKey::Symbol(_)) => Some(next_key.into()),
            (PropertyKeyType::String, PropertyKey::Index(index)) => Some(index.to_string().into()),
            _ => None,
        }
    });

    // 5. Return CreateArrayFromList(nameList).
    Ok(Array::create_array_from_list(name_list, context).into())
}
