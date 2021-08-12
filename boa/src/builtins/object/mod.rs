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
    builtins::BuiltIn,
    object::{
        ConstructorBuilder, Object as BuiltinObject, ObjectData, ObjectInitializer, PROTOTYPE,
    },
    property::{Attribute, DescriptorKind, PropertyDescriptor},
    symbol::WellKnownSymbols,
    value::{JsValue, Type},
    BoaProfiler, Context, Result,
};

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
        .method(Self::is_prototype_of, "isPrototypeOf", 0)
        .static_method(Self::create, "create", 2)
        .static_method(Self::set_prototype_of, "setPrototypeOf", 2)
        .static_method(Self::get_prototype_of, "getPrototypeOf", 1)
        .static_method(Self::define_property, "defineProperty", 3)
        .static_method(Self::define_properties, "defineProperties", 2)
        .static_method(Self::assign, "assign", 2)
        .static_method(Self::is, "is", 2)
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
    ) -> Result<JsValue> {
        if !new_target.is_undefined() {
            let prototype = new_target
                .as_object()
                .and_then(|obj| {
                    obj.__get__(&PROTOTYPE.into(), obj.clone().into(), context)
                        .map(|o| o.as_object())
                        .transpose()
                })
                .transpose()?
                .unwrap_or_else(|| context.standard_objects().object_object().prototype());
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
    pub fn create(_: &JsValue, args: &[JsValue], context: &mut Context) -> Result<JsValue> {
        let prototype = args.get(0).cloned().unwrap_or_else(JsValue::undefined);
        let properties = args.get(1).cloned().unwrap_or_else(JsValue::undefined);

        let obj = match prototype {
            JsValue::Object(_) | JsValue::Null => JsValue::new(BuiltinObject::with_prototype(
                prototype,
                ObjectData::Ordinary,
            )),
            _ => {
                return context.throw_type_error(format!(
                    "Object prototype may only be an Object or null: {}",
                    prototype.display()
                ))
            }
        };

        if !properties.is_undefined() {
            return Object::define_properties(&JsValue::undefined(), &[obj, properties], context);
        }

        Ok(obj)
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
    ) -> Result<JsValue> {
        let object = args
            .get(0)
            .unwrap_or(&JsValue::undefined())
            .to_object(context)?;
        if let Some(key) = args.get(1) {
            let key = key.to_property_key(context)?;

            if let Some(desc) = object.__get_own_property__(&key) {
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
    ) -> Result<JsValue> {
        let object = args
            .get(0)
            .unwrap_or(&JsValue::undefined())
            .to_object(context)?;
        let descriptors = context.construct_object();

        for key in object.borrow().properties().keys() {
            let descriptor = {
                let desc = object
                    .__get_own_property__(&key)
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
    pub fn is(_: &JsValue, args: &[JsValue], _: &mut Context) -> Result<JsValue> {
        let x = args.get(0).cloned().unwrap_or_else(JsValue::undefined);
        let y = args.get(1).cloned().unwrap_or_else(JsValue::undefined);

        Ok(JsValue::same_value(&x, &y).into())
    }

    /// Get the `prototype` of an object.
    pub fn get_prototype_of(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> Result<JsValue> {
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
    pub fn set_prototype_of(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> Result<JsValue> {
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
        let proto = args.get(1).cloned().unwrap_or_default();
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
            .__set_prototype_of__(proto);

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
    ) -> Result<JsValue> {
        let undefined = JsValue::undefined();
        let mut v = args.get(0).unwrap_or(&undefined).clone();
        if !v.is_object() {
            return Ok(JsValue::new(false));
        }
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
    ) -> Result<JsValue> {
        let object = args.get(0).cloned().unwrap_or_else(JsValue::undefined);
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
    ) -> Result<JsValue> {
        let arg = args.get(0).cloned().unwrap_or_default();
        let arg_obj = arg.as_object();
        if let Some(mut obj) = arg_obj {
            let props = args.get(1).cloned().unwrap_or_else(JsValue::undefined);
            obj.define_properties(props, context)?;
            Ok(arg)
        } else {
            context.throw_type_error("Expected an object")
        }
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
    pub fn to_string(this: &JsValue, _: &[JsValue], context: &mut Context) -> Result<JsValue> {
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
            match &o.data {
                ObjectData::Array => "Array",
                // TODO: Arguments Exotic Objects are currently not supported
                ObjectData::Function(_) => "Function",
                ObjectData::Error => "Error",
                ObjectData::Boolean(_) => "Boolean",
                ObjectData::Number(_) => "Number",
                ObjectData::String(_) => "String",
                ObjectData::Date(_) => "Date",
                ObjectData::RegExp(_) => "RegExp",
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
    ) -> Result<JsValue> {
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
    ) -> Result<JsValue> {
        let key = match args.get(0) {
            None => return Ok(JsValue::new(false)),
            Some(key) => key,
        };

        let key = key.to_property_key(context)?;
        let own_property = this.to_object(context)?.__get_own_property__(&key);

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
    pub fn assign(_: &JsValue, args: &[JsValue], context: &mut Context) -> Result<JsValue> {
        //
        //
        // 1. Let to be ? ToObject(target).
        let to = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_object(context)?;

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
                let keys = from.own_property_keys();
                // 3.a.iii. For each element nextKey of keys, do
                for key in keys {
                    // 3.a.iii.1. Let desc be ? from.[[GetOwnProperty]](nextKey).
                    if let Some(desc) = from.__get_own_property__(&key) {
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
}
