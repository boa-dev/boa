//! This module implements the global `Reflect` object.
//!
//! The `Reflect` global object is a built-in object that provides methods for interceptable
//! JavaScript operations.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-reflect-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect

use crate::{
    builtins::{self, BuiltIn},
    object::ObjectInitializer,
    property::Attribute,
    symbol::WellKnownSymbols,
    BoaProfiler, Context, JsResult, JsValue,
};

use super::Array;

#[cfg(test)]
mod tests;

/// Javascript `Reflect` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Reflect;

impl BuiltIn for Reflect {
    const NAME: &'static str = "Reflect";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let to_string_tag = WellKnownSymbols::to_string_tag();

        let object = ObjectInitializer::new(context)
            .function(Self::apply, "apply", 3)
            .function(Self::construct, "construct", 2)
            .function(Self::define_property, "defineProperty", 3)
            .function(Self::delete_property, "deleteProperty", 2)
            .function(Self::get, "get", 2)
            .function(
                Self::get_own_property_descriptor,
                "getOwnPropertyDescriptor",
                2,
            )
            .function(Self::get_prototype_of, "getPrototypeOf", 1)
            .function(Self::has, "has", 2)
            .function(Self::is_extensible, "isExtensible", 1)
            .function(Self::own_keys, "ownKeys", 1)
            .function(Self::prevent_extensions, "preventExtensions", 1)
            .function(Self::set, "set", 3)
            .function(Self::set_prototype_of, "setPrototypeOf", 3)
            .property(
                to_string_tag,
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
        (Self::NAME, object.into(), Self::attribute())
    }
}

impl Reflect {
    /// Calls a target function with arguments.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.apply
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/apply
    pub(crate) fn apply(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be a function"))?;
        let this_arg = args.get(1).cloned().unwrap_or_default();
        let args_list = args.get(2).cloned().unwrap_or_default();

        if !target.is_callable() {
            return context.throw_type_error("target must be a function");
        }
        let args = args_list.create_list_from_array_like(&[], context)?;
        target.call(&this_arg, &args, context)
    }

    /// Calls a target function as a constructor with arguments.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.construct
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/construct
    pub(crate) fn construct(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be a function"))?;
        let args_list = args.get(1).cloned().unwrap_or_default();

        if !target.is_constructable() {
            return context.throw_type_error("target must be a constructor");
        }

        let new_target = if let Some(new_target) = args.get(2) {
            if new_target.as_object().map(|o| o.is_constructable()) != Some(true) {
                return context.throw_type_error("newTarget must be constructor");
            }
            new_target.clone()
        } else {
            target.clone().into()
        };

        let args = args_list.create_list_from_array_like(&[], context)?;
        target.construct(&args, &new_target, context)
    }

    /// Defines a property on an object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.defineProperty
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/defineProperty
    pub(crate) fn define_property(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let undefined = JsValue::undefined();
        let target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be an object"))?;
        let key = args.get(1).unwrap_or(&undefined).to_property_key(context)?;
        let prop_desc: JsValue = args
            .get(2)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("property descriptor must be an object"))?
            .into();

        target
            .__define_own_property__(key, prop_desc.to_property_descriptor(context)?, context)
            .map(|b| b.into())
    }

    /// Defines a property on an object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.deleteproperty
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/deleteProperty
    pub(crate) fn delete_property(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let undefined = JsValue::undefined();
        let target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be an object"))?;
        let key = args.get(1).unwrap_or(&undefined).to_property_key(context)?;

        Ok(target.__delete__(&key, context)?.into())
    }

    /// Gets a property of an object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.get
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/get
    pub(crate) fn get(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let undefined = JsValue::undefined();
        // 1. If Type(target) is not Object, throw a TypeError exception.
        let target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be an object"))?;
        // 2. Let key be ? ToPropertyKey(propertyKey).
        let key = args.get(1).unwrap_or(&undefined).to_property_key(context)?;
        // 3. If receiver is not present, then
        let receiver = if let Some(receiver) = args.get(2).cloned() {
            receiver
        } else {
            // 3.a. Set receiver to target.
            target.clone().into()
        };
        // 4. Return ? target.[[Get]](key, receiver).
        target.__get__(&key, receiver, context)
    }

    /// Gets a property of an object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.getownpropertydescriptor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/getOwnPropertyDescriptor
    pub(crate) fn get_own_property_descriptor(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        match args.get(0) {
            Some(v) if v.is_object() => (),
            _ => return context.throw_type_error("target must be an object"),
        }
        // This function is the same as Object.prototype.getOwnPropertyDescriptor, that why
        // it is invoked here.
        builtins::object::Object::get_own_property_descriptor(&JsValue::undefined(), args, context)
    }

    /// Gets the prototype of an object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.getprototypeof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/getPrototypeOf
    pub(crate) fn get_prototype_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be an object"))?;
        target.__get_prototype_of__(context)
    }

    /// Returns `true` if the object has the property, `false` otherwise.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.has
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/has
    pub(crate) fn has(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be an object"))?;
        let key = args
            .get(1)
            .unwrap_or(&JsValue::undefined())
            .to_property_key(context)?;
        Ok(target.__has_property__(&key, context)?.into())
    }

    /// Returns `true` if the object is extensible, `false` otherwise.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.isextensible
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/isExtensible
    pub(crate) fn is_extensible(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be an object"))?;
        Ok(target.__is_extensible__(context)?.into())
    }

    /// Returns an array of object own property keys.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.ownkeys
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/ownKeys
    pub(crate) fn own_keys(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be an object"))?;

        let keys: Vec<JsValue> = target
            .__own_property_keys__(context)?
            .into_iter()
            .map(|key| key.into())
            .collect();

        Ok(Array::create_array_from_list(keys, context).into())
    }

    /// Prevents new properties from ever being added to an object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.preventextensions
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/preventExtensions
    pub(crate) fn prevent_extensions(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be an object"))?;

        Ok(target.__prevent_extensions__(context)?.into())
    }

    /// Sets a property of an object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.set
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/set
    pub(crate) fn set(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let undefined = JsValue::undefined();
        let target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be an object"))?;
        let key = args.get(1).unwrap_or(&undefined).to_property_key(context)?;
        let value = args.get(2).unwrap_or(&undefined);
        let receiver = if let Some(receiver) = args.get(3).cloned() {
            receiver
        } else {
            target.clone().into()
        };
        Ok(target
            .__set__(key, value.clone(), receiver, context)?
            .into())
    }

    /// Sets the prototype of an object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect.setprototypeof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/setPrototypeOf
    pub(crate) fn set_prototype_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let undefined = JsValue::undefined();
        let mut target = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| context.construct_type_error("target must be an object"))?;
        let proto = args.get(1).unwrap_or(&undefined);
        if !proto.is_null() && !proto.is_object() {
            return context.throw_type_error("proto must be an object or null");
        }
        Ok(target.__set_prototype_of__(proto.clone(), context)?.into())
    }
}
