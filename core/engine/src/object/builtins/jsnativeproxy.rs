//! Exotic objects with property internal methods implemented in Rust.
use std::{marker::PhantomData, ops::Deref};

use boa_gc::{Finalize, Trace};

use crate::{
    Context, JsExpect, JsResult, JsValue,
    object::{
        JsObject, NativeObject,
        internal_methods::{
            InternalMethodPropertyContext, InternalObjectMethods, ORDINARY_INTERNAL_METHODS,
            ordinary_define_own_property, ordinary_delete, ordinary_get, ordinary_get_own_property,
            ordinary_has_property, ordinary_own_property_keys, ordinary_set,
        },
        shape::slot::SlotAttributes,
    },
    property::{CompletePropertyDescriptor, PropertyDescriptor, PropertyKey},
};

/// Property internal methods of a [`JsNativeProxy`]. Each hook defaults to the ordinary one.
///
/// The [invariants] are not checked; upholding them is up to the implementor.
///
/// # Example
///
/// ```
/// use boa_engine::{
///     Context, JsData, JsResult, JsValue, Source, js_string,
///     object::builtins::{JsNativeProxy, NativeProxyHooks},
///     property::{CompletePropertyDescriptor, PropertyKey},
/// };
/// use boa_gc::{Finalize, Trace};
///
/// #[derive(Trace, Finalize, JsData)]
/// struct List {
///     #[unsafe_ignore_trace]
///     items: Vec<&'static str>,
/// }
///
/// impl NativeProxyHooks for List {
///     fn get_own_property(
///         this: &JsNativeProxy<Self>,
///         key: &PropertyKey,
///         context: &mut Context,
///     ) -> JsResult<Option<CompletePropertyDescriptor>> {
///         if let PropertyKey::Index(index) = key {
///             let item = this.borrow().data().items.get(index.get() as usize).copied();
///             return Ok(item.map(|item| CompletePropertyDescriptor::Data {
///                 value: js_string!(item).into(),
///                 writable: false,
///                 enumerable: true,
///                 configurable: true,
///             }));
///         }
///         this.ordinary_get_own_property(key, context)
///     }
/// }
///
/// let context = &mut Context::default();
/// let list = JsNativeProxy::new(
///     context.intrinsics().constructors().object().prototype(),
///     List { items: vec!["a", "b"] },
/// );
/// context
///     .global_object()
///     .set(js_string!("list"), list, false, context)?;
///
/// let result = context.eval(Source::from_bytes("list[0] + list[1] + list[2]"))?;
/// assert_eq!(result, JsValue::from(js_string!("abundefined")));
/// # Ok::<(), boa_engine::JsError>(())
/// ```
///
/// [invariants]: https://tc39.es/ecma262/#sec-invariants-of-the-essential-internal-methods
pub trait NativeProxyHooks: NativeObject + Sized {
    /// The `[[GetOwnProperty]]` internal method.
    fn get_own_property(
        this: &JsNativeProxy<Self>,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<Option<CompletePropertyDescriptor>> {
        this.ordinary_get_own_property(key, context)
    }

    /// The `[[DefineOwnProperty]]` internal method.
    fn define_own_property(
        this: &JsNativeProxy<Self>,
        key: &PropertyKey,
        desc: PropertyDescriptor,
        context: &mut Context,
    ) -> JsResult<bool> {
        this.ordinary_define_own_property(key, desc, context)
    }

    /// The `[[HasProperty]]` internal method.
    fn has_property(
        this: &JsNativeProxy<Self>,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<bool> {
        this.ordinary_has_property(key, context)
    }

    /// The `[[Get]]` internal method.
    fn get(
        this: &JsNativeProxy<Self>,
        key: &PropertyKey,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        this.ordinary_get(key, receiver, context)
    }

    /// The `[[Set]]` internal method.
    fn set(
        this: &JsNativeProxy<Self>,
        key: PropertyKey,
        value: JsValue,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<bool> {
        this.ordinary_set(key, value, receiver, context)
    }

    /// The `[[Delete]]` internal method.
    fn delete(
        this: &JsNativeProxy<Self>,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<bool> {
        this.ordinary_delete(key, context)
    }

    /// The `[[OwnPropertyKeys]]` internal method.
    fn own_property_keys(
        this: &JsNativeProxy<Self>,
        context: &mut Context,
    ) -> JsResult<Vec<PropertyKey>> {
        this.ordinary_own_property_keys(context)
    }
}

/// An object whose property internal methods are the [`NativeProxyHooks`] of `T`.
#[derive(Trace, Finalize)]
pub struct JsNativeProxy<T: NativeProxyHooks> {
    inner: JsObject<T>,
}

impl<T: NativeProxyHooks> JsNativeProxy<T> {
    /// Creates a new object.
    pub fn new<O: Into<Option<JsObject>>>(prototype: O, data: T) -> Self {
        let object =
            JsObject::new_unique_with_vtable(prototype, data, &NativeProxyMethods::<T>::VTABLE);
        Self { inner: object }
    }

    /// Returns the typed object.
    #[inline]
    #[must_use]
    pub fn as_object(&self) -> &JsObject<T> {
        &self.inner
    }

    /// Returns the object with its type erased.
    #[inline]
    #[must_use]
    pub fn upcast(&self) -> JsObject {
        self.inner.clone().upcast()
    }

    /// The ordinary `[[GetOwnProperty]]` of this object.
    pub fn ordinary_get_own_property(
        &self,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<Option<CompletePropertyDescriptor>> {
        ordinary_get_own_property(&self.upcast(), key, &mut context.into())
    }

    /// The ordinary `[[DefineOwnProperty]]` of this object.
    pub fn ordinary_define_own_property(
        &self,
        key: &PropertyKey,
        desc: PropertyDescriptor,
        context: &mut Context,
    ) -> JsResult<bool> {
        ordinary_define_own_property(&self.upcast(), key, desc, &mut context.into())
    }

    /// The ordinary `[[HasProperty]]` of this object.
    pub fn ordinary_has_property(
        &self,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<bool> {
        ordinary_has_property(&self.upcast(), key, &mut context.into())
    }

    /// The ordinary `[[Get]]` of this object.
    pub fn ordinary_get(
        &self,
        key: &PropertyKey,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        ordinary_get(&self.upcast(), key, receiver, &mut context.into())
    }

    /// The ordinary `[[Set]]` of this object.
    pub fn ordinary_set(
        &self,
        key: PropertyKey,
        value: JsValue,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<bool> {
        ordinary_set(&self.upcast(), key, value, receiver, &mut context.into())
    }

    /// The ordinary `[[Delete]]` of this object.
    pub fn ordinary_delete(&self, key: &PropertyKey, context: &mut Context) -> JsResult<bool> {
        ordinary_delete(&self.upcast(), key, &mut context.into())
    }

    /// The ordinary `[[OwnPropertyKeys]]` of this object.
    pub fn ordinary_own_property_keys(&self, context: &mut Context) -> JsResult<Vec<PropertyKey>> {
        ordinary_own_property_keys(&self.upcast(), context)
    }
}

impl<T: NativeProxyHooks> Clone for JsNativeProxy<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: NativeProxyHooks> std::fmt::Debug for JsNativeProxy<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: NativeProxyHooks> Deref for JsNativeProxy<T> {
    type Target = JsObject<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: NativeProxyHooks> From<JsNativeProxy<T>> for JsObject<T> {
    #[inline]
    fn from(o: JsNativeProxy<T>) -> Self {
        o.inner.clone()
    }
}

impl<T: NativeProxyHooks> From<JsNativeProxy<T>> for JsObject {
    #[inline]
    fn from(o: JsNativeProxy<T>) -> Self {
        o.inner.clone().upcast()
    }
}

impl<T: NativeProxyHooks> From<JsNativeProxy<T>> for JsValue {
    #[inline]
    fn from(o: JsNativeProxy<T>) -> Self {
        o.inner.clone().upcast().into()
    }
}

struct NativeProxyMethods<T>(PhantomData<T>);

impl<T: NativeProxyHooks> NativeProxyMethods<T> {
    const VTABLE: InternalObjectMethods = InternalObjectMethods {
        __get_own_property__: native_proxy_get_own_property::<T>,
        __define_own_property__: native_proxy_define_own_property::<T>,
        __has_property__: native_proxy_has_property::<T>,
        __try_get__: native_proxy_try_get::<T>,
        __get__: native_proxy_get::<T>,
        __set__: native_proxy_set::<T>,
        __delete__: native_proxy_delete::<T>,
        __own_property_keys__: native_proxy_own_property_keys::<T>,
        ..ORDINARY_INTERNAL_METHODS
    };
}

fn typed<T: NativeProxyHooks>(obj: &JsObject) -> JsResult<JsNativeProxy<T>> {
    let inner = obj
        .clone()
        .downcast::<T>()
        .ok()
        .js_expect("native proxy internal method called on an object of another type")?;
    Ok(JsNativeProxy { inner })
}

/// Keeps inline caches from skipping the hooks.
fn not_cacheable(context: &mut InternalMethodPropertyContext<'_>) {
    context.slot().attributes |= SlotAttributes::NOT_CACHEABLE;
}

fn native_proxy_get_own_property<T: NativeProxyHooks>(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<Option<CompletePropertyDescriptor>> {
    not_cacheable(context);
    T::get_own_property(&typed(obj)?, key, context)
}

fn native_proxy_define_own_property<T: NativeProxyHooks>(
    obj: &JsObject,
    key: &PropertyKey,
    desc: PropertyDescriptor,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<bool> {
    not_cacheable(context);
    T::define_own_property(&typed(obj)?, key, desc, context)
}

fn native_proxy_has_property<T: NativeProxyHooks>(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<bool> {
    not_cacheable(context);
    T::has_property(&typed(obj)?, key, context)
}

/// `has_property` + `get`, like `Proxy`.
fn native_proxy_try_get<T: NativeProxyHooks>(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<Option<JsValue>> {
    not_cacheable(context);
    let this = typed(obj)?;
    if T::has_property(&this, key, context)? {
        Ok(Some(T::get(&this, key, receiver, context)?))
    } else {
        Ok(None)
    }
}

fn native_proxy_get<T: NativeProxyHooks>(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<JsValue> {
    not_cacheable(context);
    T::get(&typed(obj)?, key, receiver, context)
}

fn native_proxy_set<T: NativeProxyHooks>(
    obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<bool> {
    not_cacheable(context);
    T::set(&typed(obj)?, key, value, receiver, context)
}

fn native_proxy_delete<T: NativeProxyHooks>(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<bool> {
    not_cacheable(context);
    T::delete(&typed(obj)?, key, context)
}

fn native_proxy_own_property_keys<T: NativeProxyHooks>(
    obj: &JsObject,
    context: &mut Context,
) -> JsResult<Vec<PropertyKey>> {
    T::own_property_keys(&typed(obj)?, context)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use boa_gc::{Finalize, Trace};
    use indoc::indoc;
    use rustc_hash::FxHashMap;

    use super::{JsNativeProxy, NativeProxyHooks};
    use crate::{
        Context, JsData, JsNativeError, JsNativeErrorKind, JsResult, JsString, JsValue, TestAction,
        js_string,
        native_function::NativeFunction,
        object::{JsObject, ObjectInitializer},
        property::{CompletePropertyDescriptor, PropertyDescriptor, PropertyKey},
        run_test_actions,
    };

    /// Storage-like named properties.
    #[derive(Default, Trace, Finalize, JsData)]
    struct NamedMap {
        #[unsafe_ignore_trace]
        entries: FxHashMap<JsString, JsString>,
        #[unsafe_ignore_trace]
        order: Vec<JsString>,
    }

    impl NamedMap {
        fn insert(&mut self, key: JsString, value: JsString) {
            if self.entries.insert(key.clone(), value).is_none() {
                self.order.push(key);
            }
        }

        fn remove(&mut self, key: &JsString) -> bool {
            if self.entries.remove(key).is_some() {
                self.order.retain(|k| k != key);
                true
            } else {
                false
            }
        }
    }

    impl NativeProxyHooks for NamedMap {
        fn get_own_property(
            this: &JsNativeProxy<Self>,
            key: &PropertyKey,
            context: &mut Context,
        ) -> JsResult<Option<CompletePropertyDescriptor>> {
            if let PropertyKey::String(name) = key
                && let Some(value) = this.borrow().data().entries.get(name)
            {
                return Ok(Some(CompletePropertyDescriptor::Data {
                    value: value.clone().into(),
                    writable: true,
                    enumerable: true,
                    configurable: true,
                }));
            }
            this.ordinary_get_own_property(key, context)
        }

        fn define_own_property(
            this: &JsNativeProxy<Self>,
            key: &PropertyKey,
            desc: PropertyDescriptor,
            context: &mut Context,
        ) -> JsResult<bool> {
            if let PropertyKey::String(name) = key
                && let Some(value) = desc.value()
            {
                let value = value.to_string(context)?;
                this.borrow_mut().data_mut().insert(name.clone(), value);
                return Ok(true);
            }
            this.ordinary_define_own_property(key, desc, context)
        }

        fn delete(
            this: &JsNativeProxy<Self>,
            key: &PropertyKey,
            context: &mut Context,
        ) -> JsResult<bool> {
            if let PropertyKey::String(name) = key
                && this.borrow_mut().data_mut().remove(name)
            {
                return Ok(true);
            }
            this.ordinary_delete(key, context)
        }

        fn own_property_keys(
            this: &JsNativeProxy<Self>,
            context: &mut Context,
        ) -> JsResult<Vec<PropertyKey>> {
            let mut keys = this.ordinary_own_property_keys(context)?;
            keys.extend(
                this.borrow()
                    .data()
                    .order
                    .iter()
                    .cloned()
                    .map(PropertyKey::from),
            );
            Ok(keys)
        }
    }

    /// `next` returns a new number on every read.
    #[derive(Default, Trace, Finalize, JsData)]
    struct Counter {
        #[unsafe_ignore_trace]
        count: Cell<u32>,
    }

    impl NativeProxyHooks for Counter {
        fn has_property(
            this: &JsNativeProxy<Self>,
            key: &PropertyKey,
            context: &mut Context,
        ) -> JsResult<bool> {
            if key == &PropertyKey::from(js_string!("next")) {
                return Ok(true);
            }
            this.ordinary_has_property(key, context)
        }

        fn get(
            this: &JsNativeProxy<Self>,
            key: &PropertyKey,
            receiver: JsValue,
            context: &mut Context,
        ) -> JsResult<JsValue> {
            if key == &PropertyKey::from(js_string!("next")) {
                let data = this.borrow();
                let count = &data.data().count;
                count.set(count.get() + 1);
                return Ok(count.get().into());
            }
            this.ordinary_get(key, receiver, context)
        }
    }

    #[derive(Trace, Finalize, JsData)]
    struct Plain;

    impl NativeProxyHooks for Plain {}

    fn size(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let map = this
            .as_object()
            .and_then(|o| o.downcast_ref::<NamedMap>().map(|m| m.entries.len()))
            .ok_or_else(|| JsNativeError::typ().with_message("Illegal invocation"))?;
        Ok(map.into())
    }

    fn register(context: &mut Context) {
        let object_prototype = context.intrinsics().constructors().object().prototype();
        let prototype = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(size), js_string!("size"), 0)
            .build();
        prototype.set_prototype(Some(object_prototype.clone()));

        let map = JsNativeProxy::new(prototype.clone(), NamedMap::default());
        let counter = JsNativeProxy::new(object_prototype.clone(), Counter::default());
        let plain = JsNativeProxy::new(object_prototype, Plain);

        let global = context.global_object();
        for (name, value) in [
            (js_string!("map"), JsValue::from(map)),
            (js_string!("mapPrototype"), prototype.into()),
            (js_string!("counter"), counter.into()),
            (js_string!("plain"), plain.into()),
        ] {
            global
                .define_property_or_throw(
                    name,
                    PropertyDescriptor::builder()
                        .value(value)
                        .writable(true)
                        .enumerable(false)
                        .configurable(true),
                    context,
                )
                .expect("should define the test globals");
        }
    }

    fn map_object(context: &mut Context) -> JsObject {
        context
            .global_object()
            .get(js_string!("map"), context)
            .expect("map should be defined")
            .as_object()
            .expect("map should be an object")
            .clone()
    }

    #[test]
    fn named_properties() {
        run_test_actions([
            TestAction::run_harness(),
            TestAction::inspect_context(register),
            TestAction::run(indoc! {r#"
                map.a = 1;
                map["b"] = { toString() { return "two"; } };
                Object.defineProperty(map, "c", { value: 3 });
            "#}),
            TestAction::assert_eq("map.a", js_string!("1")),
            TestAction::assert_eq("map.b", js_string!("two")),
            TestAction::assert("'c' in map"),
            TestAction::assert(r#"arrayEquals(Object.keys(map), ["a", "b", "c"])"#),
            TestAction::assert(r#"JSON.stringify(map) === '{"a":"1","b":"two","c":"3"}'"#),
            TestAction::assert("delete map.b"),
            TestAction::assert("!('b' in map) && map.b === undefined"),
            TestAction::assert(r#"arrayEquals(Object.getOwnPropertyNames(map), ["a", "c"])"#),
            TestAction::inspect_context(|context| {
                let map = map_object(context);
                let data = map
                    .downcast_ref::<NamedMap>()
                    .expect("map should be a NamedMap");
                assert_eq!(data.order, [js_string!("a"), js_string!("c")]);
            }),
        ]);
    }

    #[test]
    fn prototype_methods_see_the_data() {
        run_test_actions([
            TestAction::inspect_context(register),
            TestAction::run("map.x = 'x'; map.y = 'y'"),
            TestAction::assert_eq("map.size()", 2),
            TestAction::assert_native_error(
                "mapPrototype.size.call({})",
                JsNativeErrorKind::Type,
                "Illegal invocation",
            ),
        ]);
    }

    #[test]
    fn hooks_are_not_cached() {
        run_test_actions([
            TestAction::run_harness(),
            TestAction::inspect_context(register),
            TestAction::run(indoc! {r#"
                function read(o) { return o.next; }
                var direct = [];
                var inherited = [];
                var child = Object.create(counter);
                for (let i = 0; i < 3; i++) {
                    direct.push(read(counter));
                }
                for (let i = 0; i < 3; i++) {
                    inherited.push(read(child));
                }
            "#}),
            TestAction::assert("arrayEquals(direct, [1, 2, 3])"),
            TestAction::assert("arrayEquals(inherited, [4, 5, 6])"),
            TestAction::run(indoc! {r#"
                function readA(o) { return o.a; }
                var values = [];
                for (let i = 0; i < 3; i++) {
                    map.a = "v" + i;
                    values.push(readA(map));
                }
            "#}),
            TestAction::assert(r#"arrayEquals(values, ["v0", "v1", "v2"])"#),
        ]);
    }

    #[test]
    fn with_statement_uses_has_and_get() {
        run_test_actions([
            TestAction::inspect_context(register),
            TestAction::assert_eq("with (counter) { next + next }", 3),
        ]);
    }

    #[test]
    fn default_hooks_are_ordinary() {
        run_test_actions([
            TestAction::run_harness(),
            TestAction::inspect_context(register),
            TestAction::run(indoc! {r#"
                plain.a = 1;
                Object.defineProperty(plain, "fixed", { value: 2 });
                plain[0] = "zero";
            "#}),
            TestAction::assert_eq("plain.a + plain.fixed", 3),
            TestAction::assert(r#"arrayEquals(Object.keys(plain), ["0", "a"])"#),
            TestAction::assert("!delete plain.fixed && delete plain.a && !('a' in plain)"),
            TestAction::assert_native_error(
                "'use strict'; plain.fixed = 3",
                JsNativeErrorKind::Type,
                "cannot set non-writable property: fixed",
            ),
        ]);
    }
}
