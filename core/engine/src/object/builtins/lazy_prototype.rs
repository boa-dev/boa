use crate::{
    object::{
        internal_methods::{
            non_existant_call, non_existant_construct, ordinary_define_own_property,
            ordinary_delete, ordinary_get, ordinary_get_own_property, ordinary_get_prototype_of,
            ordinary_has_property, ordinary_is_extensible, ordinary_own_property_keys,
            ordinary_prevent_extensions, ordinary_set, ordinary_set_prototype_of, ordinary_try_get,
            InternalMethodContext, InternalObjectMethods,
        },
        JsPrototype,
    },
    property::{PropertyDescriptor, PropertyKey},
    Context, JsData, JsObject, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};

use super::LazyBuiltIn;

/// The `LazyPrototype` struct is responsible for ensuring that the prototype
/// of a constructor is lazily initialized. In JavaScript, constructors have a
/// `prototype` property that points to an object which is used as the prototype
/// for instances created by that constructor.
///
/// The `LazyPrototype` struct is used within `JsObject` (e.g., `JsObject<LazyPrototype>`)
/// to defer the creation of the prototype object until it is actually needed.
/// This lazy initialization helps improve performance by avoiding the creation
/// of the prototype object until it is necessary.
///
/// Each `LazyPrototype` instance points to the constructor's `lazyBuiltin` (via its
/// object) and triggers the initialization of the prototype if any methods on
/// the prototype are called.

#[derive(Clone, Trace, Finalize, Debug)]
#[allow(clippy::type_complexity)]
pub struct LazyPrototype {
    pub(crate) constructor: JsObject<LazyBuiltIn>,
}

// Implement the trait for JsData by overriding all internal_methods by calling init on the LazyBuiltIn associated with this prototype
impl JsData for LazyPrototype {
    fn internal_methods(&self) -> &'static InternalObjectMethods {
        &LAZY_INTERNAL_METHODS
    }
}

pub(crate) static LAZY_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __get_prototype_of__: lazy_get_prototype_of,
    __set_prototype_of__: lazy_set_prototype_of,
    __is_extensible__: lazy_is_extensible,
    __prevent_extensions__: lazy_prevent_extensions,
    __get_own_property__: lazy_get_own_property,
    __define_own_property__: lazy_define_own_property,
    __has_property__: lazy_has_property,
    __try_get__: lazy_try_get,
    __get__: lazy_get,
    __set__: lazy_set,
    __delete__: lazy_delete,
    __own_property_keys__: lazy_own_property_keys,
    __call__: non_existant_call,
    __construct__: non_existant_construct,
};

pub(crate) fn lazy_get_prototype_of(
    obj: &JsObject,
    context: &mut Context,
) -> JsResult<JsPrototype> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);
    ordinary_get_prototype_of(obj, context)
}

pub(crate) fn lazy_set_prototype_of(
    obj: &JsObject,
    prototype: JsPrototype,
    context: &mut Context,
) -> JsResult<bool> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);
    ordinary_set_prototype_of(obj, prototype, context)
}
pub(crate) fn lazy_is_extensible(obj: &JsObject, context: &mut Context) -> JsResult<bool> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);
    ordinary_is_extensible(obj, context)
}

pub(crate) fn lazy_prevent_extensions(obj: &JsObject, context: &mut Context) -> JsResult<bool> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);
    ordinary_prevent_extensions(obj, context)
}

pub(crate) fn lazy_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<Option<PropertyDescriptor>> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);
    ordinary_get_own_property(obj, key, context)
}

pub(crate) fn lazy_define_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    desc: PropertyDescriptor,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);

    ordinary_define_own_property(obj, key, desc, context)
}

pub(crate) fn lazy_has_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);
    ordinary_has_property(obj, key, context)
}

pub(crate) fn lazy_try_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<Option<JsValue>> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);

    ordinary_try_get(obj, key, receiver, context)
}

pub(crate) fn lazy_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<JsValue> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);
    ordinary_get(obj, key, receiver, context)
}

pub(crate) fn lazy_set(
    obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);
    ordinary_set(obj, key, value, receiver, context)
}

pub(crate) fn lazy_delete(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);
    ordinary_delete(obj, key, context)
}

pub(crate) fn lazy_own_property_keys(
    obj: &JsObject,
    context: &mut Context,
) -> JsResult<Vec<PropertyKey>> {
    let lazy_prototype: JsObject<LazyPrototype> = obj
        .clone()
        .downcast::<LazyPrototype>()
        .expect("obj is not a Builtin");
    let lazy_built_in = &lazy_prototype.borrow_mut().data.constructor.clone();
    LazyBuiltIn::ensure_init(lazy_built_in);

    ordinary_own_property_keys(obj, context)
}
