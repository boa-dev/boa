use super::JsFunction;
use crate::{
    gc::custom_trace,
    object::{
        internal_methods::{
            non_existant_call, non_existant_construct, ordinary_define_own_property,
            ordinary_delete, ordinary_get, ordinary_get_own_property, ordinary_get_prototype_of,
            ordinary_has_property, ordinary_is_extensible, ordinary_own_property_keys,
            ordinary_prevent_extensions, ordinary_set, ordinary_set_prototype_of, ordinary_try_get,
            CallValue, InternalMethodContext, InternalObjectMethods,
        },
        JsPrototype,
    },
    property::{PropertyDescriptor, PropertyKey},
    realm::{Realm, RealmInner},
    Context, JsData, JsNativeError, JsObject, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace, WeakGc};
use std::cell::Cell;

#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) enum BuiltinKind {
    Constructor(JsFunction),
    Ordinary,
}

/// A builtin function. Used for lazy initialization of builtins.

#[derive(Clone, Finalize)]
pub struct BuiltIn {
    pub(crate) init: fn(&Realm),
    pub(crate) is_initialized: Cell<bool>,
    pub(crate) kind: BuiltinKind,
    pub(crate) realm_inner: Option<WeakGc<RealmInner>>,
}

// SAFETY: Temporary, TODO move back to derived Trace when possible
unsafe impl Trace for BuiltIn {
    custom_trace!(this, mark, {
        mark(&this.kind);
    });
}

// Implement the trait for JsData by overriding all internal_methods by calling init before calling into the underlying internel_method
impl JsData for BuiltIn {
    fn internal_methods(&self) -> &'static InternalObjectMethods {
        static CONSTRUCTOR: InternalObjectMethods = InternalObjectMethods {
            __construct__: lazy_construct,
            ..LAZY_INTERNAL_METHODS
        };

        if let BuiltinKind::Constructor(_) = self.kind {
            return &CONSTRUCTOR;
        }

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
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_get_prototype_of(obj, context)
}

pub(crate) fn lazy_set_prototype_of(
    obj: &JsObject,
    prototype: JsPrototype,
    context: &mut Context,
) -> JsResult<bool> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_set_prototype_of(obj, prototype, context)
}
pub(crate) fn lazy_is_extensible(obj: &JsObject, context: &mut Context) -> JsResult<bool> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_is_extensible(obj, context)
}

pub(crate) fn lazy_prevent_extensions(obj: &JsObject, context: &mut Context) -> JsResult<bool> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_prevent_extensions(obj, context)
}

pub(crate) fn lazy_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<Option<PropertyDescriptor>> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_get_own_property(obj, key, context)
}

pub(crate) fn lazy_define_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    desc: PropertyDescriptor,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_define_own_property(obj, key, desc, context)
}

pub(crate) fn lazy_has_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_has_property(obj, key, context)
}

pub(crate) fn lazy_try_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<Option<JsValue>> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_try_get(obj, key, receiver, context)
}

pub(crate) fn lazy_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<JsValue> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_get(obj, key, receiver, context)
}

pub(crate) fn lazy_set(
    obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        builtin_borrow.data.is_initialized.set(true);
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_set(obj, key, value, receiver, context)
}

pub(crate) fn lazy_delete(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_delete(obj, key, context)
}

pub(crate) fn lazy_own_property_keys(
    obj: &JsObject,
    context: &mut Context,
) -> JsResult<Vec<PropertyKey>> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    ordinary_own_property_keys(obj, context)
}

pub(crate) fn lazy_construct(
    obj: &JsObject,
    argument_count: usize,
    context: &mut Context,
) -> JsResult<CallValue> {
    let builtin: JsObject<BuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    let kind = &builtin.borrow().data.kind;
    if !builtin.borrow().data.is_initialized.get() {
        let builtin_borrow = builtin.borrow_mut();
        let realm = &Realm {
            inner: builtin_borrow
                .data
                .realm_inner
                .as_ref()
                .expect("realm_inner not set")
                .upgrade()
                .expect("realm_inner not set"),
        };
        builtin_borrow.data.is_initialized.set(true);
        let init_fn = builtin_borrow.data.init;
        drop(builtin_borrow);
        init_fn(realm);
    }

    match kind {
        BuiltinKind::Ordinary => Err(JsNativeError::typ()
            .with_message("not a constructor")
            .with_realm(context.realm().clone())
            .into()),
        BuiltinKind::Constructor(constructor) => Ok(constructor.__construct__(argument_count)),
    }
}
