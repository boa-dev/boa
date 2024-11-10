use crate::{
    builtins::function::ConstructorKind,
    gc::custom_trace,
    native_function::{
        native_function_call_inner, native_function_construct_inner, NativeFunctionObject,
    },
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
    Context, JsData, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
};
use boa_gc::{Finalize, Trace, WeakGc};

#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) enum BuiltinKind {
    Function(NativeFunctionObject),
    #[allow(dead_code)]
    Ordinary,
}

/// A Lazy Built-in data structure. Used for lazy initialization of builtins.
#[derive(Clone, Finalize, Debug)]
#[allow(clippy::type_complexity)]
pub struct LazyBuiltIn {
    pub(crate) init_and_realm: Option<(fn(&Realm), WeakGc<RealmInner>)>,
    pub(crate) kind: BuiltinKind,
}

impl LazyBuiltIn {
    pub(crate) fn set_constructor(&mut self, function: NativeFunction, realm: Realm) {
        if let BuiltinKind::Function(ref mut native_function) = self.kind {
            native_function.f = function;
            native_function.constructor = Some(ConstructorKind::Base);
            native_function.realm = Some(realm);
        } else {
            panic!("Expected BuiltinKind::Function");
        }
    }

    pub(crate) fn ensure_init(built_in: &JsObject<LazyBuiltIn>) {
        let borrowed_built_in = built_in.borrow_mut().data.init_and_realm.take();
        if let Some((init, realm_inner)) = borrowed_built_in {
            let realm = &Realm {
                inner: realm_inner.upgrade().expect("realm_inner not set"),
            };
            init(realm);
        }
    }
}

// SAFETY: Temporary, TODO move back to derived Trace when possible
unsafe impl Trace for LazyBuiltIn {
    custom_trace!(this, mark, {
        mark(&this.kind);
    });
}

// Implement the trait for JsData by overriding all internal_methods by calling init before calling into the underlying internel_method
impl JsData for LazyBuiltIn {
    fn internal_methods(&self) -> &'static InternalObjectMethods {
        static FUNCTION: InternalObjectMethods = InternalObjectMethods {
            __construct__: lazy_construct,
            __call__: lazy_call,
            ..LAZY_INTERNAL_METHODS
        };

        if let BuiltinKind::Function(_) = self.kind {
            return &FUNCTION;
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
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);
    ordinary_get_prototype_of(obj, context)
}

pub(crate) fn lazy_set_prototype_of(
    obj: &JsObject,
    prototype: JsPrototype,
    context: &mut Context,
) -> JsResult<bool> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);
    ordinary_set_prototype_of(obj, prototype, context)
}
pub(crate) fn lazy_is_extensible(obj: &JsObject, context: &mut Context) -> JsResult<bool> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);
    ordinary_is_extensible(obj, context)
}

pub(crate) fn lazy_prevent_extensions(obj: &JsObject, context: &mut Context) -> JsResult<bool> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);
    ordinary_prevent_extensions(obj, context)
}

pub(crate) fn lazy_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<Option<PropertyDescriptor>> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);
    ordinary_get_own_property(obj, key, context)
}

pub(crate) fn lazy_define_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    desc: PropertyDescriptor,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);

    ordinary_define_own_property(obj, key, desc, context)
}

pub(crate) fn lazy_has_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);
    ordinary_has_property(obj, key, context)
}

pub(crate) fn lazy_try_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<Option<JsValue>> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);

    ordinary_try_get(obj, key, receiver, context)
}

pub(crate) fn lazy_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<JsValue> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);
    ordinary_get(obj, key, receiver, context)
}

pub(crate) fn lazy_set(
    obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);
    ordinary_set(obj, key, value, receiver, context)
}

pub(crate) fn lazy_delete(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);
    ordinary_delete(obj, key, context)
}

pub(crate) fn lazy_own_property_keys(
    obj: &JsObject,
    context: &mut Context,
) -> JsResult<Vec<PropertyKey>> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);

    ordinary_own_property_keys(obj, context)
}

pub(crate) fn lazy_construct(
    obj: &JsObject,
    argument_count: usize,
    context: &mut Context,
) -> JsResult<CallValue> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);
    let kind = &builtin.borrow().data.kind.clone();

    match kind {
        BuiltinKind::Ordinary => Err(JsNativeError::typ()
            .with_message("not a constructor")
            .with_realm(context.realm().clone())
            .into()),
        BuiltinKind::Function(cons) => {
            // builtin needs to be dropped before calling the constructor to avoid a double borrow
            drop(builtin);
            native_function_construct_inner(cons, obj.clone(), argument_count, context)
        }
    }
}

pub(crate) fn lazy_call(
    obj: &JsObject,
    argument_count: usize,
    context: &mut Context,
) -> JsResult<CallValue> {
    let builtin: JsObject<LazyBuiltIn> = obj.clone().downcast().expect("obj is not a Builtin");
    LazyBuiltIn::ensure_init(&builtin);
    let kind = &builtin.borrow().data.kind.clone();
    match kind {
        BuiltinKind::Ordinary => Err(JsNativeError::typ()
            .with_message("not a constructor")
            .with_realm(context.realm().clone())
            .into()),
        BuiltinKind::Function(function) => {
            // builtin needs to be dropped before calling the constructor to avoid a double borrow
            drop(builtin);
            native_function_call_inner(obj, function, argument_count, context)
        }
    }
}
