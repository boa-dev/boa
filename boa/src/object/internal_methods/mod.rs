//! This module defines the object internal methods.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots

use crate::{
    object::JsObject,
    property::{DescriptorKind, PropertyDescriptor, PropertyKey},
    value::JsValue,
    BoaProfiler, Context, JsResult,
};

pub(super) mod array;
pub(super) mod string;

impl JsObject {
    /// `[[hasProperty]]`
    #[inline]
    pub(crate) fn __has_property__(
        &self,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__has_property__;
        func(self, key, context)
    }

    /// Check if it is extensible.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-isextensible
    #[inline]
    pub(crate) fn __is_extensible__(&self, context: &mut Context) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__is_extensible__;
        func(self, context)
    }

    /// Disable extensibility.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
    #[inline]
    pub(crate) fn __prevent_extensions__(&mut self, context: &mut Context) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__prevent_extensions__;
        func(self, context)
    }

    /// Delete property.
    #[inline]
    pub(crate) fn __delete__(&self, key: &PropertyKey, context: &mut Context) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__delete__;
        func(self, key, context)
    }

    /// `[[Get]]`
    pub(crate) fn __get__(
        &self,
        key: &PropertyKey,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let func = self.borrow().data.internal_methods.__get__;
        func(self, key, receiver, context)
    }

    /// `[[Set]]`
    pub(crate) fn __set__(
        &self,
        key: PropertyKey,
        value: JsValue,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<bool> {
        let _timer = BoaProfiler::global().start_event("Object::set", "object");
        let func = self.borrow().data.internal_methods.__set__;
        func(self, key, value, receiver, context)
    }

    /// `[[defineOwnProperty]]`
    pub(crate) fn __define_own_property__(
        &self,
        key: PropertyKey,
        desc: PropertyDescriptor,
        context: &mut Context,
    ) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__define_own_property__;
        func(self, key, desc, context)
    }

    /// Gets own property of 'Object'
    ///
    #[inline]
    pub(crate) fn __get_own_property__(
        &self,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<Option<PropertyDescriptor>> {
        let _timer = BoaProfiler::global().start_event("Object::get_own_property", "object");
        let func = self.borrow().data.internal_methods.__get_own_property__;
        func(self, key, context)
    }

    /// Essential internal method OwnPropertyKeys
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#table-essential-internal-methods
    #[inline]
    #[track_caller]
    pub(crate) fn __own_property_keys__(
        &self,
        context: &mut Context,
    ) -> JsResult<Vec<PropertyKey>> {
        let func = self.borrow().data.internal_methods.__own_property_keys__;
        func(self, context)
    }

    /// `Object.setPropertyOf(obj, prototype)`
    ///
    /// This method sets the prototype (i.e., the internal `[[Prototype]]` property)
    /// of a specified object to another object or `null`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/setPrototypeOf
    #[inline]
    pub(crate) fn __set_prototype_of__(
        &mut self,
        val: JsValue,
        context: &mut Context,
    ) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__set_prototype_of__;
        func(self, val, context)
    }

    /// Returns either the prototype or null
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getPrototypeOf
    #[inline]
    #[track_caller]
    pub(crate) fn __get_prototype_of__(&self, context: &mut Context) -> JsResult<JsValue> {
        let func = self.borrow().data.internal_methods.__get_prototype_of__;
        func(self, context)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct InternalObjectMethods {
    pub(crate) __get_prototype_of__: fn(&JsObject, &mut Context) -> JsResult<JsValue>,
    pub(crate) __set_prototype_of__: fn(&JsObject, JsValue, &mut Context) -> JsResult<bool>,
    pub(crate) __is_extensible__: fn(&JsObject, &mut Context) -> JsResult<bool>,
    pub(crate) __prevent_extensions__: fn(&JsObject, &mut Context) -> JsResult<bool>,
    pub(crate) __get_own_property__:
        fn(&JsObject, &PropertyKey, &mut Context) -> JsResult<Option<PropertyDescriptor>>,
    pub(crate) __define_own_property__:
        fn(&JsObject, PropertyKey, PropertyDescriptor, &mut Context) -> JsResult<bool>,
    pub(crate) __has_property__: fn(&JsObject, &PropertyKey, &mut Context) -> JsResult<bool>,
    pub(crate) __get__: fn(&JsObject, &PropertyKey, JsValue, &mut Context) -> JsResult<JsValue>,
    pub(crate) __set__:
        fn(&JsObject, PropertyKey, JsValue, JsValue, &mut Context) -> JsResult<bool>,
    pub(crate) __delete__: fn(&JsObject, &PropertyKey, &mut Context) -> JsResult<bool>,
    pub(crate) __own_property_keys__: fn(&JsObject, &mut Context) -> JsResult<Vec<PropertyKey>>,
}

impl Default for InternalObjectMethods {
    fn default() -> Self {
        Self {
            __get_prototype_of__: ordinary_get_prototype_of,
            __set_prototype_of__: ordinary_set_prototype_of,
            __is_extensible__: ordinary_is_extensible,
            __prevent_extensions__: ordinary_prevent_extensions,
            __get_own_property__: ordinary_get_own_property,
            __define_own_property__: ordinary_define_own_property,
            __has_property__: ordinary_has_property,
            __get__: ordinary_get,
            __set__: ordinary_set,
            __delete__: ordinary_delete,
            __own_property_keys__: ordinary_own_property_keys,
        }
    }
}

/// Returns either the prototype or null
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getPrototypeOf
#[inline]
pub(crate) fn ordinary_get_prototype_of(
    obj: &JsObject,
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(obj.borrow().prototype.clone())
}

/// `Object.setPropertyOf(obj, prototype)`
///
/// This method sets the prototype (i.e., the internal `[[Prototype]]` property)
/// of a specified object to another object or `null`.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/setPrototypeOf
#[inline]
pub(crate) fn ordinary_set_prototype_of(
    obj: &JsObject,
    val: JsValue,
    context: &mut Context,
) -> JsResult<bool> {
    debug_assert!(val.is_object() || val.is_null());
    let current = obj.__get_prototype_of__(context)?;
    if JsValue::same_value(&current, &val) {
        return Ok(true);
    }
    if !obj.__is_extensible__(context)? {
        return Ok(false);
    }
    let mut p = val.clone();
    let mut done = false;
    while !done {
        match p {
            JsValue::Null => done = true,
            JsValue::Object(ref proto) => {
                if JsObject::equals(proto, obj) {
                    return Ok(false);
                } else if proto.borrow().data.internal_methods.__get_prototype_of__ as usize
                    != ordinary_get_prototype_of as usize
                {
                    done = true;
                } else {
                    p = proto.__get_prototype_of__(context)?;
                }
            }
            _ => unreachable!(),
        }
    }
    obj.borrow_mut().prototype = val;
    Ok(true)
}

/// Check if it is extensible.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-isextensible
// NOTE: for now context is not used but it will in the future.
#[inline]
pub(crate) fn ordinary_is_extensible(obj: &JsObject, _context: &mut Context) -> JsResult<bool> {
    Ok(obj.borrow().extensible)
}

/// Disable extensibility.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
#[inline]
pub(crate) fn ordinary_prevent_extensions(
    obj: &JsObject,
    _context: &mut Context,
) -> JsResult<bool> {
    obj.borrow_mut().extensible = false;
    Ok(true)
}

/// Get property of object without checking its prototype.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
#[inline]
pub(crate) fn ordinary_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    _context: &mut Context,
) -> JsResult<Option<PropertyDescriptor>> {
    Ok(obj.borrow().properties.get(key).cloned())
}

/// Define property of object.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc
#[inline]
pub(crate) fn ordinary_define_own_property(
    obj: &JsObject,
    key: PropertyKey,
    desc: PropertyDescriptor,
    context: &mut Context,
) -> JsResult<bool> {
    let current = obj.__get_own_property__(&key, context)?;
    let extensible = obj.__is_extensible__(context)?;
    Ok(validate_and_apply_property_descriptor(
        Some((obj, key)),
        extensible,
        desc,
        current,
    ))
}

// Check if object has property.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
#[inline]
pub(crate) fn ordinary_has_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context,
) -> JsResult<bool> {
    let prop = obj.__get_own_property__(key, context)?;
    if prop.is_none() {
        let parent = obj.__get_prototype_of__(context)?;
        return if let JsValue::Object(ref object) = parent {
            object.__has_property__(key, context)
        } else {
            Ok(false)
        };
    }
    Ok(true)
}

/// `OrdinaryGet`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-get-p-receiver
#[inline]
pub(crate) fn ordinary_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    match obj.__get_own_property__(key, context)? {
        None => {
            // parent will either be null or an Object
            if let Some(parent) = obj.__get_prototype_of__(context)?.as_object() {
                parent.__get__(key, receiver, context)
            } else {
                Ok(JsValue::undefined())
            }
        }
        Some(ref desc) => match desc.kind() {
            DescriptorKind::Data {
                value: Some(value), ..
            } => Ok(value.clone()),
            DescriptorKind::Accessor { get: Some(get), .. } if !get.is_undefined() => {
                context.call(get, &receiver, &[])
            }
            _ => Ok(JsValue::undefined()),
        },
    }
}

/// `[[Set]]`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-set-p-v-receiver
#[inline]
pub(crate) fn ordinary_set(
    obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut Context,
) -> JsResult<bool> {
    // Fetch property key
    let own_desc = if let Some(desc) = obj.__get_own_property__(&key, context)? {
        desc
    } else if let Some(ref mut parent) = obj.__get_prototype_of__(context)?.as_object() {
        return parent.__set__(key, value, receiver, context);
    } else {
        PropertyDescriptor::builder()
            .value(JsValue::undefined())
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build()
    };

    if own_desc.is_data_descriptor() {
        if !own_desc.expect_writable() {
            return Ok(false);
        }

        let receiver = match receiver.as_object() {
            Some(obj) => obj,
            _ => return Ok(false),
        };

        if let Some(ref existing_desc) = receiver.__get_own_property__(&key, context)? {
            if existing_desc.is_accessor_descriptor() {
                return Ok(false);
            }
            if !existing_desc.expect_writable() {
                return Ok(false);
            }
            return receiver.__define_own_property__(
                key,
                PropertyDescriptor::builder().value(value).build(),
                context,
            );
        } else {
            return receiver.create_data_property(key, value, context);
        }
    }

    match own_desc.set() {
        Some(set) if !set.is_undefined() => {
            context.call(set, &receiver, &[value])?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Delete property.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-delete-p
#[inline]
pub(crate) fn ordinary_delete(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context,
) -> JsResult<bool> {
    Ok(match obj.__get_own_property__(key, context)? {
        Some(desc) if desc.expect_configurable() => {
            obj.borrow_mut().remove(key);
            true
        }
        Some(_) => false,
        None => true,
    })
}

/// Essential internal method `[[OwnPropertyKeys]]`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-ownpropertykeys
#[inline]
pub(crate) fn ordinary_own_property_keys(
    obj: &JsObject,
    _context: &mut Context,
) -> JsResult<Vec<PropertyKey>> {
    // todo: sort keys or ensure in some way that indexed properties are sorted... or maybe it's not necessary?
    Ok(obj.borrow().properties.keys().collect())
}

/// Abstract operation `IsCompatiblePropertyDescriptor `
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-iscompatiblepropertydescriptor
#[inline]
pub(crate) fn is_compatible_property_descriptor(
    extensible: bool,
    desc: PropertyDescriptor,
    current: PropertyDescriptor,
) -> bool {
    validate_and_apply_property_descriptor(None, extensible, desc, Some(current))
}

/// Abstract operation `ValidateAndApplyPropertyDescriptor`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-validateandapplypropertydescriptor
#[inline]
pub(crate) fn validate_and_apply_property_descriptor(
    obj_and_key: Option<(&JsObject, PropertyKey)>,
    extensible: bool,
    desc: PropertyDescriptor,
    current: Option<PropertyDescriptor>,
) -> bool {
    let mut current = if let Some(own) = current {
        own
    } else {
        if !extensible {
            return false;
        }

        if let Some((obj, key)) = obj_and_key {
            obj.borrow_mut().properties.insert(
                key,
                if desc.is_generic_descriptor() || desc.is_data_descriptor() {
                    desc.into_data_defaulted()
                } else {
                    desc.into_accessor_defaulted()
                },
            );
        }

        return true;
    };

    // 3
    if desc.is_empty() {
        return true;
    }

    // 4
    if !current.expect_configurable() {
        if matches!(desc.configurable(), Some(true)) {
            return false;
        }

        if matches!(desc.enumerable(), Some(desc_enum) if desc_enum != current.expect_enumerable())
        {
            return false;
        }
    }

    // 5
    if desc.is_generic_descriptor() {
        // no further validation required
    } else if current.is_data_descriptor() != desc.is_data_descriptor() {
        if !current.expect_configurable() {
            return false;
        }
        if current.is_data_descriptor() {
            current = current.into_accessor_defaulted();
        } else {
            current = current.into_data_defaulted();
        }
    } else if current.is_data_descriptor() && desc.is_data_descriptor() {
        if !current.expect_configurable() && !current.expect_writable() {
            if matches!(desc.writable(), Some(true)) {
                return false;
            }
            if matches!(desc.value(), Some(value) if !JsValue::same_value(value, current.expect_value()))
            {
                return false;
            }
            return true;
        }
    } else if !current.expect_configurable() {
        if matches!(desc.set(), Some(set) if !JsValue::same_value(set, current.expect_set())) {
            return false;
        }
        if matches!(desc.get(), Some(get) if !JsValue::same_value(get, current.expect_get())) {
            return false;
        }
        return true;
    }

    if let Some((obj, key)) = obj_and_key {
        current.fill_with(desc);
        obj.borrow_mut().properties.insert(key, current);
    }

    true
}
