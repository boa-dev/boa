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

use crate::{
    builtins::{BuiltIn, JsArgs},
    context::StandardObjects,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, IntegrityLevel,
        JsObject, Object as BuiltinObject, ObjectData, ObjectInitializer, ObjectKind,
    },
    property::{Attribute, DescriptorKind, PropertyDescriptor, PropertyKey, PropertyNameKind},
    symbol::WellKnownSymbols,
    value::{JsValue, Type},
    BoaProfiler, Context, JsResult,
};

use super::Array;

pub mod for_in_iterator;
#[cfg(test)]
mod tests;

/// The global JavaScript object.
#[derive(Debug, Clone, Copy)]
pub struct Object;

impl BuiltIn for Object {
    const NAME: &'static str = "Object";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().object_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .inherit(JsValue::null())
        .method(Self::has_own_property, "hasOwnProperty", 0)
        .method(Self::property_is_enumerable, "propertyIsEnumerable", 0)
        .method(Self::to_string, "toString", 0)
        .method(Self::value_of, "valueOf", 0)
        .method(Self::is_prototype_of, "isPrototypeOf", 0)
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
        .build();

        (Self::NAME, object.into(), Self::attribute())
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
            let prototype = get_prototype_from_constructor(
                new_target,
                StandardObjects::object_object,
                context,
            )?;
            let object = JsValue::new_object(context);

            object
                .as_object()
                .expect("this should be an object")
                .set_prototype_instance(prototype.into());
            return Ok(object);
        }
        if let Some(arg) = args.get(0) {
            if !arg.is_null_or_undefined() {
                return Ok(arg.to_object(context)?.into());
            }
        }
        Ok(JsValue::new_object(context))
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
            JsValue::Object(_) | JsValue::Null => JsObject::new(BuiltinObject::with_prototype(
                prototype.clone(),
                ObjectData::ordinary(),
            )),
            _ => {
                return context.throw_type_error(format!(
                    "Object prototype may only be an Object or null: {}",
                    prototype.display()
                ))
            }
        };

        if !properties.is_undefined() {
            object_define_properties(&obj, properties.clone(), context)?;
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
        let object = args.get_or_undefined(0).to_object(context)?;
        if let Some(key) = args.get(1) {
            let key = key.to_property_key(context)?;

            if let Some(desc) = object.__get_own_property__(&key, context)? {
                return Ok(Self::from_property_descriptor(desc, context));
            }
        }

        Ok(JsValue::undefined())
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
        let object = args
            .get(0)
            .unwrap_or(&JsValue::undefined())
            .to_object(context)?;
        let descriptors = context.construct_object();

        for key in object.borrow().properties().keys() {
            let descriptor = {
                let desc = object
                    .__get_own_property__(&key, context)?
                    .expect("Expected property to be on object.");
                Self::from_property_descriptor(desc, context)
            };

            if !descriptor.is_undefined() {
                descriptors.borrow_mut().insert(
                    key,
                    PropertyDescriptor::builder()
                        .value(descriptor)
                        .writable(true)
                        .enumerable(true)
                        .configurable(true),
                );
            }
        }

        Ok(JsValue::Object(descriptors))
    }

    /// The abstract operation `FromPropertyDescriptor`.
    ///
    /// [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-frompropertydescriptor
    fn from_property_descriptor(desc: PropertyDescriptor, context: &mut Context) -> JsValue {
        let mut descriptor = ObjectInitializer::new(context);

        // TODO: use CreateDataPropertyOrThrow

        match desc.kind() {
            DescriptorKind::Data { value, writable } => {
                if let Some(value) = value {
                    descriptor.property("value", value.clone(), Attribute::all());
                }
                if let Some(writable) = writable {
                    descriptor.property("writable", *writable, Attribute::all());
                }
            }
            DescriptorKind::Accessor { get, set } => {
                if let Some(get) = get {
                    descriptor.property("get", get.clone(), Attribute::all());
                }
                if let Some(set) = set {
                    descriptor.property("set", set.clone(), Attribute::all());
                }
            }
            _ => {}
        }

        if let Some(enumerable) = desc.enumerable() {
            descriptor.property("enumerable", enumerable, Attribute::all());
        }

        if let Some(configurable) = desc.configurable() {
            descriptor.property("configurable", configurable, Attribute::all());
        }

        descriptor.build().into()
    }

    /// Uses the SameValue algorithm to check equality of objects
    pub fn is(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let x = args.get_or_undefined(0);
        let y = args.get_or_undefined(1);

        Ok(JsValue::same_value(x, y).into())
    }

    /// Get the `prototype` of an object.
    pub fn get_prototype_of(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        if args.is_empty() {
            return ctx.throw_type_error(
                "Object.getPrototypeOf: At least 1 argument required, but only 0 passed",
            );
        }

        // 1. Let obj be ? ToObject(O).
        let obj = args[0].clone().to_object(ctx)?;

        // 2. Return ? obj.[[GetPrototypeOf]]().
        Ok(obj.prototype_instance())
    }

    /// Set the `prototype` of an object.
    ///
    /// [More information][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.setprototypeof
    pub fn set_prototype_of(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        if args.len() < 2 {
            return ctx.throw_type_error(format!(
                "Object.setPrototypeOf: At least 2 arguments required, but only {} passed",
                args.len()
            ));
        }

        // 1. Set O to ? RequireObjectCoercible(O).
        let obj = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .require_object_coercible(ctx)?
            .clone();

        // 2. If Type(proto) is neither Object nor Null, throw a TypeError exception.
        let proto = args.get_or_undefined(1);
        if !matches!(proto.get_type(), Type::Object | Type::Null) {
            return ctx.throw_type_error(format!(
                "expected an object or null, got {}",
                proto.type_of()
            ));
        }

        // 3. If Type(O) is not Object, return O.
        if !obj.is_object() {
            return Ok(obj);
        }

        // 4. Let status be ? O.[[SetPrototypeOf]](proto).
        let status = obj
            .as_object()
            .expect("obj was not an object")
            .__set_prototype_of__(proto.clone(), ctx)?;

        // 5. If status is false, throw a TypeError exception.
        if !status {
            return ctx.throw_type_error("can't set prototype of this object");
        }

        // 6. Return O.
        Ok(obj)
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
        if let Some(object) = object.as_object() {
            let key = args
                .get(1)
                .unwrap_or(&JsValue::Undefined)
                .to_property_key(context)?;
            let desc = args
                .get(2)
                .unwrap_or(&JsValue::Undefined)
                .to_property_descriptor(context)?;

            object.define_property_or_throw(key, desc, context)?;

            Ok(object.into())
        } else {
            context.throw_type_error("Object.defineProperty called on non-object")
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
        let arg_obj = arg.as_object();
        if let Some(obj) = arg_obj {
            let props = args.get_or_undefined(1);
            object_define_properties(&obj, props.clone(), context)?;
            Ok(arg.clone())
        } else {
            context.throw_type_error("Expected an object")
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
        let o = this.to_object(context)?;
        // TODO: 4. Let isArray be ? IsArray(O).
        // TODO: 5. If isArray is true, let builtinTag be "Array".

        // 6. Else if O has a [[ParameterMap]] internal slot, let builtinTag be "Arguments".
        // 7. Else if O has a [[Call]] internal method, let builtinTag be "Function".
        // 8. Else if O has an [[ErrorData]] internal slot, let builtinTag be "Error".
        // 9. Else if O has a [[BooleanData]] internal slot, let builtinTag be "Boolean".
        // 10. Else if O has a [[NumberData]] internal slot, let builtinTag be "Number".
        // 11. Else if O has a [[StringData]] internal slot, let builtinTag be "String".
        // 12. Else if O has a [[DateValue]] internal slot, let builtinTag be "Date".
        // 13. Else if O has a [[RegExpMatcher]] internal slot, let builtinTag be "RegExp".
        // 14. Else, let builtinTag be "Object".
        let builtin_tag = {
            let o = o.borrow();
            match o.kind() {
                ObjectKind::Array => "Array",
                // TODO: Arguments Exotic Objects are currently not supported
                ObjectKind::Function(_) => "Function",
                ObjectKind::Error => "Error",
                ObjectKind::Boolean(_) => "Boolean",
                ObjectKind::Number(_) => "Number",
                ObjectKind::String(_) => "String",
                ObjectKind::Date(_) => "Date",
                ObjectKind::RegExp(_) => "RegExp",
                _ => "Object",
            }
        };

        // 15. Let tag be ? Get(O, @@toStringTag).
        let tag = o.get(WellKnownSymbols::to_string_tag(), context)?;

        // 16. If Type(tag) is not String, set tag to builtinTag.
        let tag_str = tag.as_string().map(|s| s.as_str()).unwrap_or(builtin_tag);

        // 17. Return the string-concatenation of "[object ", tag, and "]".
        Ok(format!("[object {}]", tag_str).into())
    }

    /// `Object.prototype.hasOwnPrototype( property )`
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
        let key = args
            .get(0)
            .unwrap_or(&JsValue::undefined())
            .to_property_key(context)?;
        let object = this.to_object(context)?;

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
        let own_property = this
            .to_object(context)?
            .__get_own_property__(&key, context)?;

        Ok(own_property.map_or(JsValue::new(false), |own_prop| {
            JsValue::new(own_prop.enumerable())
        }))
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
                let from = source.to_object(context).unwrap();
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
                return context.throw_type_error("cannot seal object");
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
                return context.throw_type_error("cannot freeze object");
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
                return context.throw_type_error("cannot prevent extensions");
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
}

/// The abstract operation ObjectDefineProperties
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-object.defineproperties
#[inline]
fn object_define_properties(
    object: &JsObject,
    props: JsValue,
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
