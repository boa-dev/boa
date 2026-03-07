//! Boa's implementation of the `Iterator` global object and iterator helper objects.
//!
//! The `Iterator` constructor and its prototype methods are defined by the
//! iterator helpers proposal (Stage 4, part of ECMAScript 2025).
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/proposal-iterator-helpers/
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Iterator

use crate::{
    Context, JsArgs, JsData, JsResult, JsString,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
        array::Array,
        iterable::{IteratorRecord, create_iter_result_object, if_abrupt_close_iterator},
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};

/// The state for a `map` iterator helper.
#[derive(Debug, Trace, Finalize)]
pub(crate) struct MapState {
    iterated: IteratorRecord,
    mapper: JsObject,
    counter: f64,
}

/// The state for a `filter` iterator helper.
#[derive(Debug, Trace, Finalize)]
pub(crate) struct FilterState {
    iterated: IteratorRecord,
    predicate: JsObject,
    counter: f64,
}

/// The state for a `take` iterator helper.
#[derive(Debug, Trace, Finalize)]
pub(crate) struct TakeState {
    iterated: IteratorRecord,
    remaining: f64,
}

/// The state for a `drop` iterator helper.
#[derive(Debug, Trace, Finalize)]
pub(crate) struct DropState {
    iterated: IteratorRecord,
    remaining: f64,
}

/// The state for a `flatMap` iterator helper.
#[derive(Debug, Trace, Finalize)]
pub(crate) struct FlatMapState {
    iterated: IteratorRecord,
    mapper: JsObject,
    counter: f64,
    /// The current inner iterator, if one is active.
    inner: Option<IteratorRecord>,
}

/// Internal state for each `%IteratorHelper%` object.
#[derive(Debug, Trace, Finalize)]
pub(crate) enum IteratorHelperState {
    Map(MapState),
    Filter(FilterState),
    Take(TakeState),
    Drop(DropState),
    FlatMap(FlatMapState),
    // Note: Done state is represented by `None` in the `IteratorHelper.state` Option.
}

/// The `JsData` wrapper stored inside a `%IteratorHelper%` object.
#[derive(Debug, Trace, Finalize, JsData)]
pub(crate) struct IteratorHelper {
    /// `None` means the helper is exhausted (done).
    state: Option<IteratorHelperState>,
}

impl IteratorHelper {
    /// Create a new `%IteratorHelper%` object for `map`.
    fn create_map(
        iterated: IteratorRecord,
        mapper: JsObject,
        context: &Context,
    ) -> JsObject {
        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .iterator_helper(),
            Self { state: Some(IteratorHelperState::Map(MapState { iterated, mapper, counter: 0.0 })) },
        ).upcast()
    }

    /// Create a new `%IteratorHelper%` object for `filter`.
    fn create_filter(
        iterated: IteratorRecord,
        predicate: JsObject,
        context: &Context,
    ) -> JsObject {
        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .iterator_helper(),
            Self { state: Some(IteratorHelperState::Filter(FilterState { iterated, predicate, counter: 0.0 })) },
        ).upcast()
    }

    /// Create a new `%IteratorHelper%` object for `take`.
    fn create_take(
        iterated: IteratorRecord,
        remaining: f64,
        context: &Context,
    ) -> JsObject {
        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .iterator_helper(),
            Self { state: Some(IteratorHelperState::Take(TakeState { iterated, remaining })) },
        ).upcast()
    }

    /// Create a new `%IteratorHelper%` object for `drop`.
    fn create_drop(
        iterated: IteratorRecord,
        remaining: f64,
        context: &Context,
    ) -> JsObject {
        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .iterator_helper(),
            Self { state: Some(IteratorHelperState::Drop(DropState { iterated, remaining })) },
        ).upcast()
    }

    /// Create a new `%IteratorHelper%` object for `flatMap`.
    fn create_flat_map(
        iterated: IteratorRecord,
        mapper: JsObject,
        context: &Context,
    ) -> JsObject {
        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .iterator_helper(),
            Self { state: Some(IteratorHelperState::FlatMap(FlatMapState {
                iterated,
                mapper,
                counter: 0.0,
                inner: None,
            })) },
        ).upcast()
    }
}

/// `%IteratorHelperPrototype%`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-%iteratorhelperprototype%-object
pub(crate) struct IteratorHelperPrototype;

impl IntrinsicObject for IteratorHelperPrototype {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .iterator(),
            )
            .static_method(Self::next, js_string!("next"), 0)
            .static_method(Self::r#return, js_string!("return"), 0)
            .static_property(
                JsSymbol::to_string_tag(),
                StaticJsStrings::ITERATOR_HELPER,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().iterator_helper()
    }
}

impl IteratorHelperPrototype {
    /// `%IteratorHelperPrototype%.next ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-%iteratorhelperprototype%.next
    pub(crate) fn next(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be this value.
        let Some(obj) = this.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("`this` is not an IteratorHelper object")
                .into());
        };

        // Drive the appropriate helper state machine.
        let mut helper = obj.downcast_mut::<IteratorHelper>().ok_or_else(|| {
            JsNativeError::typ().with_message("`this` is not an IteratorHelper object")
        })?;

        match &mut helper.state {
            None => {
                // Already exhausted.
                return Ok(create_iter_result_object(JsValue::undefined(), true, context));
            }
            Some(IteratorHelperState::Map(state)) => {
                // Spec: closure for map
                loop {
                    // a. Let value be ? IteratorStepValue(iterated).
                    let Some(value) = state.iterated.step_value(context)? else {
                        // b. If value is done, return undefined.
                        drop(helper);
                        obj.downcast_mut::<IteratorHelper>().expect("same obj").state = None;
                        return Ok(create_iter_result_object(JsValue::undefined(), true, context));
                    };
                    let counter = state.counter;
                    // c. Let mapped be Completion(Call(mapper, undefined, « value, 𝔽(counter) »)).
                    let mapped =
                        state.mapper.call(&JsValue::undefined(), &[value, counter.into()], context);
                    // d. IfAbruptCloseIterator(mapped, iterated).
                    let mapped = if_abrupt_close_iterator!(mapped, state.iterated, context);
                    // e. Set counter to counter + 1.
                    state.counter += 1.0;
                    return Ok(create_iter_result_object(mapped, false, context));
                }
            }

            Some(IteratorHelperState::Filter(state)) => {
                loop {
                    // a. Let value be ? IteratorStepValue(iterated).
                    let Some(value) = state.iterated.step_value(context)? else {
                        drop(helper);
                        obj.downcast_mut::<IteratorHelper>().expect("same obj").state = None;
                        return Ok(create_iter_result_object(JsValue::undefined(), true, context));
                    };
                    let counter = state.counter;
                    // b. Let selected be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).
                    let selected = state.predicate.call(
                        &JsValue::undefined(),
                        &[value.clone(), counter.into()],
                        context,
                    );
                    // c. IfAbruptCloseIterator(selected, iterated).
                    let selected = if_abrupt_close_iterator!(selected, state.iterated, context);
                    // d. Set counter to counter + 1.
                    state.counter += 1.0;
                    // e. If ToBoolean(selected) is true, perform Yield(value).
                    if selected.to_boolean() {
                        return Ok(create_iter_result_object(value, false, context));
                    }
                }
            }

            Some(IteratorHelperState::Take(state)) => {
                // a. If remaining is 0, then close and return done.
                if state.remaining <= 0.0 {
                    let iterated = state.iterated.clone();
                    drop(helper);
                    obj.downcast_mut::<IteratorHelper>().expect("same obj").state = None;
                    iterated.close(Ok(JsValue::undefined()), context)?;
                    return Ok(create_iter_result_object(JsValue::undefined(), true, context));
                }
                // b. Set remaining to remaining - 1.
                state.remaining -= 1.0;
                // c. Let value be ? IteratorStepValue(iterated).
                let Some(value) = state.iterated.step_value(context)? else {
                    drop(helper);
                    obj.downcast_mut::<IteratorHelper>().expect("same obj").state = None;
                    return Ok(create_iter_result_object(JsValue::undefined(), true, context));
                };
                if state.remaining <= 0.0 {
                    // Last value — close the outer iterator after yielding.
                    let iterated = state.iterated.clone();
                    drop(helper);
                    obj.downcast_mut::<IteratorHelper>().expect("same obj").state = None;
                    iterated.close(Ok(JsValue::undefined()), context)?;
                }
                Ok(create_iter_result_object(value, false, context))
            }

            Some(IteratorHelperState::Drop(state)) => {
                // a. Repeat while remaining > 0,
                while state.remaining > 0.0 {
                    // i. Let next be ? IteratorStepValue(iterated).
                    let Some(_) = state.iterated.step_value(context)? else {
                        drop(helper);
                        obj.downcast_mut::<IteratorHelper>().expect("same obj").state = None;
                        return Ok(create_iter_result_object(JsValue::undefined(), true, context));
                    };
                    state.remaining -= 1.0;
                }
                // b. Let value be ? IteratorStepValue(iterated).
                let Some(value) = state.iterated.step_value(context)? else {
                    drop(helper);
                    obj.downcast_mut::<IteratorHelper>().expect("same obj").state = None;
                    return Ok(create_iter_result_object(JsValue::undefined(), true, context));
                };
                Ok(create_iter_result_object(value, false, context))
            }

            Some(IteratorHelperState::FlatMap(state)) => {
                loop {
                    // If we have an active inner iterator, drain it first.
                    if let Some(ref mut inner) = state.inner {
                        let Some(inner_value) = inner.step_value(context)? else {
                            // Inner exhausted — move to next outer value.
                            state.inner = None;
                            state.counter += 1.0;
                            continue;
                        };
                        return Ok(create_iter_result_object(inner_value, false, context));
                    }

                    // Get next outer value.
                    let Some(value) = state.iterated.step_value(context)? else {
                        drop(helper);
                        obj.downcast_mut::<IteratorHelper>().expect("same obj").state = None;
                        return Ok(create_iter_result_object(JsValue::undefined(), true, context));
                    };
                    let counter = state.counter;
                    // mapped = Call(mapper, undefined, « value, 𝔽(counter) »)
                    let mapped = state.mapper.call(
                        &JsValue::undefined(),
                        &[value, counter.into()],
                        context,
                    );
                    let mapped = if_abrupt_close_iterator!(mapped, state.iterated, context);

                    // GetIteratorFlattenable: reject strings, require object.
                    let mapped_obj = mapped.as_object().ok_or_else(|| {
                        JsNativeError::typ()
                            .with_message("flatMap mapper must return an object")
                    })?;

                    // Get inner iterator via @@iterator or directly.
                    let inner = if let Some(method) =
                        mapped_obj.get_method(JsSymbol::iterator(), context)?
                    {
                        let iter_val = method.call(&mapped.clone(), &[], context)?;
                        let iter_obj = iter_val.as_object().ok_or_else(|| {
                            JsNativeError::typ()
                                .with_message("@@iterator must return an object")
                        })?;
                        let next_method = iter_obj.get(js_string!("next"), context)?;
                        IteratorRecord::new(iter_obj.clone(), next_method)
                    } else {
                        // No @@iterator — treat the object itself as an iterator (GetIteratorDirect).
                        let next_method = mapped_obj.get(js_string!("next"), context)?;
                        IteratorRecord::new(mapped_obj.clone(), next_method)
                    };

                    state.inner = Some(inner);
                    // counter incremented when inner is exhausted
                }
            }
        }
    }

    /// `%IteratorHelperPrototype%.return ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-%iteratorhelperprototype%.return
    pub(crate) fn r#return(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be this value.
        let Some(obj) = this.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("`this` is not an IteratorHelper object")
                .into());
        };

        let mut helper = obj.downcast_mut::<IteratorHelper>().ok_or_else(|| {
            JsNativeError::typ().with_message("`this` is not an IteratorHelper object")
        })?;

        // Mark as done and close the underlying iterator.
        let underlying = match helper.state.take() {
            Some(IteratorHelperState::Map(ref s)) => Some(s.iterated.clone()),
            Some(IteratorHelperState::Filter(ref s)) => Some(s.iterated.clone()),
            Some(IteratorHelperState::Take(ref s)) => Some(s.iterated.clone()),
            Some(IteratorHelperState::Drop(ref s)) => Some(s.iterated.clone()),
            Some(IteratorHelperState::FlatMap(ref s)) => Some(s.iterated.clone()),
            None => None,
        };
        drop(helper);

        if let Some(iterated) = underlying {
            // 4. Perform ? IteratorClose(O.[[UnderlyingIterator]], NormalCompletion(unused)).
            iterated.close(Ok(JsValue::undefined()), context)?;
        }

        // 5. Return CreateIterResultObject(undefined, true).
        Ok(create_iter_result_object(JsValue::undefined(), true, context))
    }
}

/// `%WrapForValidIteratorPrototype%`
///
/// Wraps a non-`%Iterator%`-derived object for use with `Iterator.from`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-wrapforvaliditeratorprototype-object
#[derive(Debug, Trace, Finalize, JsData)]
pub(crate) struct WrapForValidIterator {
    /// `[[Iterated]]`
    iterated: IteratorRecord,
}

pub(crate) struct WrapForValidIteratorPrototype;

impl IntrinsicObject for WrapForValidIteratorPrototype {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .iterator(),
            )
            .static_method(Self::next, js_string!("next"), 0)
            .static_method(Self::r#return, js_string!("return"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics
            .objects()
            .iterator_prototypes()
            .wrap_for_valid_iterator()
    }
}

impl WrapForValidIteratorPrototype {
    /// `%WrapForValidIteratorPrototype%.next ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-%wrapforvaliditeratorprototype%.next
    pub(crate) fn next(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be this value.
        // 2. Perform ? RequireInternalSlot(O, [[Iterated]]).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("`this` is not a WrapForValidIterator object")
        })?;
        let mut wrapper = obj
            .downcast_mut::<WrapForValidIterator>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("`this` is not a WrapForValidIterator object")
            })?;
        // 3. Let iteratorRecord be O.[[Iterated]].
        // 4. Return ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]]).
        wrapper.iterated.next(None, context).map(|r| r.object().clone().into())
    }

    /// `%WrapForValidIteratorPrototype%.return ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-%wrapforvaliditeratorprototype%.return
    pub(crate) fn r#return(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be this value.
        // 2. Perform ? RequireInternalSlot(O, [[Iterated]]).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("`this` is not a WrapForValidIterator object")
        })?;
        let wrapper = obj
            .downcast_ref::<WrapForValidIterator>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("`this` is not a WrapForValidIterator object")
            })?;
        // 3. Let iterator be O.[[Iterated]].[[Iterator]].
        let iterator = wrapper.iterated.iterator().clone();
        drop(wrapper);
        // 4. Assert: iterator is an Object.
        // 5. Let returnMethod be ? GetMethod(iterator, "return").
        let return_method = iterator.get_method(js_string!("return"), context)?;
        // 6. If returnMethod is undefined, return CreateIterResultObject(undefined, true).
        let Some(return_method) = return_method else {
            return Ok(create_iter_result_object(JsValue::undefined(), true, context));
        };
        // 7. Return ? Call(returnMethod, iterator).
        return_method.call(&iterator.into(), &[], context)
    }
}

/// `%Iterator%` constructor
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iterator-constructor
pub(crate) struct IteratorConstructor;

impl IntrinsicObject for IteratorConstructor {
    fn init(realm: &Realm) {
        // %Iterator.prototype% is the shared object all iterators (Array/Map/Set/etc.) inherit from.
        // We must add our methods directly to it.

        // Build get/set accessor functions for Iterator.prototype[@@toStringTag].
        let get_to_string_tag =
            BuiltInBuilder::callable(realm, Self::get_to_string_tag)
                .name(StaticJsStrings::GET_ITERATOR_PROTOTYPE_TO_STRING_TAG)
                .build();
        let set_to_string_tag =
            BuiltInBuilder::callable(realm, Self::set_to_string_tag)
                .name(StaticJsStrings::SET_ITERATOR_PROTOTYPE_TO_STRING_TAG)
                .build();

        // Build get/set accessor functions for Iterator.prototype.constructor.
        let get_constructor =
            BuiltInBuilder::callable(realm, Self::get_constructor)
                .name(js_string!("get constructor"))
                .build();
        let set_constructor =
            BuiltInBuilder::callable(realm, Self::set_constructor)
                .name(js_string!("set constructor"))
                .build();

        // Add all prototype methods directly to %IteratorPrototype%.
        BuiltInBuilder::with_intrinsic::<crate::builtins::iterable::Iterator>(realm)
            .static_method(Self::map, js_string!("map"), 1)
            .static_method(Self::filter, js_string!("filter"), 1)
            .static_method(Self::take, js_string!("take"), 1)
            .static_method(Self::drop, js_string!("drop"), 1)
            .static_method(Self::flat_map, js_string!("flatMap"), 1)
            .static_method(Self::reduce, js_string!("reduce"), 1)
            .static_method(Self::to_array, js_string!("toArray"), 0)
            .static_method(Self::for_each, js_string!("forEach"), 1)
            .static_method(Self::some, js_string!("some"), 1)
            .static_method(Self::every, js_string!("every"), 1)
            .static_method(Self::find, js_string!("find"), 1)
            .build();

        // Add @@toStringTag and constructor accessors to %IteratorPrototype% directly.
        let iterator_proto = realm
            .intrinsics()
            .objects()
            .iterator_prototypes()
            .iterator();

        iterator_proto.insert(
            JsSymbol::to_string_tag(),
            crate::property::PropertyDescriptor::builder()
                .get(get_to_string_tag)
                .set(set_to_string_tag)
                .enumerable(false)
                .configurable(true)
                .build(),
        );

        // Set up %Iterator% as a standard constructor with Iterator.from() as static method.
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(Self::from, js_string!("from"), 1)
            .build();

        // Override the default 'constructor' data property with our custom accessor.
        let iterator_constructor = realm.intrinsics().constructors().iterator().constructor();
        iterator_proto.insert(
            js_string!("constructor"),
            crate::property::PropertyDescriptor::builder()
                .get(get_constructor)
                .set(set_constructor)
                .enumerable(false)
                .configurable(true)
                .build(),
        );

        // Wire: Iterator.prototype (the standard constructor slot) = %IteratorPrototype%.
        // The standard constructor setup created a separate prototype object; we override it.
        iterator_constructor.insert(
            js_string!("prototype"),
            crate::property::PropertyDescriptor::builder()
                .value(iterator_proto)
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
        );
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IteratorConstructor {
    const NAME: JsString = StaticJsStrings::ITERATOR;
}

impl BuiltInConstructor for IteratorConstructor {
    /// 11 methods only (constructor and @@toStringTag added post-build directly)
    const PROTOTYPE_STORAGE_SLOTS: usize = 11;
    /// 1 static method (from)
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 1;
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::iterator;

    /// `Iterator ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iterator-constructor
    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined or the active function object, throw a TypeError.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Iterator cannot be called as a function")
                .into());
        }
        let active_fn = context.active_function_object();
        if let Some(active) = active_fn {
            if let Some(nt_obj) = new_target.as_object() {
                if JsObject::equals(&nt_obj, &active) {
                    return Err(JsNativeError::typ()
                        .with_message(
                            "Iterator cannot be directly constructed; use a subclass",
                        )
                        .into());
                }
            }
        }
        // 2. Return ? OrdinaryCreateFromConstructor(NewTarget, "%Iterator.prototype%").
        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::iterator,
            context,
        )?;
        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            crate::builtins::OrdinaryObject,
        )
        .into())
    }
}

impl IteratorConstructor {
    // -------------------------------------------------------------------------
    // Abstract operations

    /// `GetIteratorDirect ( obj )`
    ///
    /// Creates an `IteratorRecord` directly from `obj` without calling `@@iterator`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-getiteratordirect
    fn get_iterator_direct(obj: JsObject, context: &mut Context) -> JsResult<IteratorRecord> {
        // 1. Let nextMethod be ? Get(obj, "next").
        let next_method = obj.get(js_string!("next"), context)?;
        // 2. Let iteratorRecord be Record { [[Iterator]]: obj, [[NextMethod]]: nextMethod, [[Done]]: false }.
        // 3. Return iteratorRecord.
        Ok(IteratorRecord::new(obj, next_method))
    }

    /// `GetIteratorFlattenable ( obj, stringHandling )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-getiteratorflattenable
    fn get_iterator_flattenable(
        obj: &JsValue,
        reject_strings: bool,
        context: &mut Context,
    ) -> JsResult<IteratorRecord> {
        // 1. If obj is not an Object, then
        if !obj.is_object() {
            // a. If stringHandling is reject-strings or obj is not a String, throw TypeError.
            if reject_strings || !obj.is_string() {
                return Err(JsNativeError::typ()
                    .with_message("Iterator.from requires an object or iterable")
                    .into());
            }
        }
        // 2. Let method be ? GetMethod(obj, @@iterator).
        let method = obj.get_method(JsSymbol::iterator(), context)?;
        let iterator = if let Some(method) = method {
            // 3. If method is not undefined, let iterator be ? Call(method, obj).
            let result = method.call(obj, &[], context)?;
            // 4. If iterator is not an Object, throw TypeError.
            result.as_object().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("@@iterator must return an object")
            })?.clone()
        } else {
            // 4. Else let iterator be obj.
            obj.as_object().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("value must be an object or iterable")
            })?.clone()
        };
        // 5. Return ? GetIteratorDirect(iterator).
        Self::get_iterator_direct(iterator, context)
    }

    // -------------------------------------------------------------------------
    // Static methods

    /// `Iterator.from ( O )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iterator.from
    pub(crate) fn from(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let o = args.get_or_undefined(0);
        // 1. Let iteratorRecord be ? GetIteratorFlattenable(O, iterate-strings).
        let iterator_record = Self::get_iterator_flattenable(o, false, context)?;
        // 2. Let hasInstance be ? OrdinaryHasInstance(%Iterator%, iteratorRecord.[[Iterator]]).
        let iterator_constructor = context.intrinsics().constructors().iterator().constructor();
        let iterator_obj = iterator_record.iterator().clone();
        let has_instance = JsValue::ordinary_has_instance(
            &iterator_constructor.into(),
            &iterator_obj.clone().into(),
            context,
        )?;
        // 3. If hasInstance is true, return iteratorRecord.[[Iterator]].
        if has_instance {
            return Ok(iterator_obj.into());
        }
        // 4. Let wrapper be OrdinaryObjectCreate(%WrapForValidIteratorPrototype%, « [[Iterated]] »).
        // 5. Set wrapper.[[Iterated]] to iteratorRecord.
        // 6. Return wrapper.
        let wrapper = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .wrap_for_valid_iterator(),
            WrapForValidIterator {
                iterated: iterator_record,
            },
        ).upcast().into();
        Ok(wrapper)
    }

    // -------------------------------------------------------------------------
    // %Iterator.prototype% accessor helpers

    /// `get Iterator.prototype [ @@toStringTag ]`
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-get-iteratorprototype-@@tostringtag
    fn get_to_string_tag(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Return "Iterator".
        Ok(StaticJsStrings::ITERATOR.into())
    }

    /// `set Iterator.prototype [ @@toStringTag ]`
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-set-iteratorprototype-@@tostringtag
    fn set_to_string_tag(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // SetterThatIgnoresPrototypeProperties(this, %Iterator.prototype%, %Symbol.toStringTag%, v)
        let v = args.get_or_undefined(0).clone();
        // 1. If this is not an Object, throw TypeError.
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype[@@toStringTag] setter called on non-object")
        })?;
        // 2. If this is the home (Iterator.prototype), throw TypeError (non-writable simulation).
        let iterator_proto = context.intrinsics().constructors().iterator().prototype();
        if JsObject::equals(&this_obj, &iterator_proto) {
            return Err(JsNativeError::typ()
                .with_message("Cannot assign to read-only property 'Symbol(Symbol.toStringTag)' of object '[object Iterator]'")
                .into());
        }
        // 3. If this has own property p, Set; else CreateDataProperty.
        this_obj.set(JsSymbol::to_string_tag(), v, true, context)?;
        Ok(JsValue::undefined())
    }

    /// `get Iterator.prototype.constructor`
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-get-iteratorprototype-constructor
    fn get_constructor(
        _this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Return %Iterator%.
        Ok(context
            .intrinsics()
            .constructors()
            .iterator()
            .constructor()
            .into())
    }

    /// `set Iterator.prototype.constructor`
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-set-iteratorprototype-constructor
    fn set_constructor(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let v = args.get_or_undefined(0).clone();
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Iterator.prototype.constructor setter called on non-object")
        })?;
        let iterator_proto = context.intrinsics().constructors().iterator().prototype();
        if JsObject::equals(&this_obj, &iterator_proto) {
            return Err(JsNativeError::typ()
                .with_message("Cannot assign to read-only property 'constructor' of object '[object Iterator]'")
                .into());
        }
        this_obj.set(js_string!("constructor"), v, true, context)?;
        Ok(JsValue::undefined())
    }

    // -------------------------------------------------------------------------
    // Lazy %Iterator.prototype% methods

    /// `Iterator.prototype.map ( mapper )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.map
    pub(crate) fn map(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.map called on non-object")
        })?;
        // 3. If IsCallable(mapper) is false, throw a TypeError.
        let mapper = args.get_or_undefined(0);
        let mapper_obj = mapper.as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.map requires a callable mapper")
        })?;
        // 4. Let iterated be ? GetIteratorDirect(O).
        let iterated = Self::get_iterator_direct(o.clone(), context)?;
        // 5-16. Create IteratorHelper with Map state.
        let helper = IteratorHelper::create_map(iterated, mapper_obj.clone(), context);
        Ok(helper.into())
    }

    /// `Iterator.prototype.filter ( predicate )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.filter
    pub(crate) fn filter(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.filter called on non-object")
        })?;
        // 3. If IsCallable(predicate) is false, throw a TypeError.
        let predicate = args.get_or_undefined(0);
        let predicate_obj = predicate.as_callable().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Iterator.prototype.filter requires a callable predicate")
        })?;
        // 4. Let iterated be ? GetIteratorDirect(O).
        let iterated = Self::get_iterator_direct(o.clone(), context)?;
        // 5-16. Create IteratorHelper with Filter state.
        let helper = IteratorHelper::create_filter(iterated, predicate_obj.clone(), context);
        Ok(helper.into())
    }

    /// `Iterator.prototype.take ( limit )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.take
    pub(crate) fn take(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.take called on non-object")
        })?;
        // 3. Let numLimit be ? ToNumber(limit).
        let num_limit = args
            .get_or_undefined(0)
            .to_number(context)?;
        // 4. If numLimit is NaN, throw a RangeError.
        if num_limit.is_nan() {
            return Err(JsNativeError::range()
                .with_message("Iterator.prototype.take requires a finite limit")
                .into());
        }
        // 5. Let intLimit be ! ToIntegerOrInfinity(limit). If intLimit < 0, throw RangeError.
        let int_limit = num_limit.floor();
        if int_limit < 0.0 {
            return Err(JsNativeError::range()
                .with_message("Iterator.prototype.take requires a non-negative limit")
                .into());
        }
        // 6. Let iterated be ? GetIteratorDirect(O).
        let iterated = Self::get_iterator_direct(o.clone(), context)?;
        // 7-15. Create IteratorHelper with Take state.
        let helper = IteratorHelper::create_take(iterated, int_limit, context);
        Ok(helper.into())
    }

    /// `Iterator.prototype.drop ( limit )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.drop
    pub(crate) fn drop(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.drop called on non-object")
        })?;
        // 3. Let numLimit be ? ToNumber(limit).
        let num_limit = args
            .get_or_undefined(0)
            .to_number(context)?;
        // 4. If numLimit is NaN, throw a RangeError.
        if num_limit.is_nan() {
            return Err(JsNativeError::range()
                .with_message("Iterator.prototype.drop requires a finite limit")
                .into());
        }
        // 5. Let intLimit be ! ToIntegerOrInfinity(limit). If intLimit < 0, throw RangeError.
        let int_limit = num_limit.floor();
        if int_limit < 0.0 {
            return Err(JsNativeError::range()
                .with_message("Iterator.prototype.drop requires a non-negative limit")
                .into());
        }
        // 6. Let iterated be ? GetIteratorDirect(O).
        let iterated = Self::get_iterator_direct(o.clone(), context)?;
        // 7-15. Create IteratorHelper with Drop state.
        let helper = IteratorHelper::create_drop(iterated, int_limit, context);
        Ok(helper.into())
    }

    /// `Iterator.prototype.flatMap ( mapper )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.flatmap
    pub(crate) fn flat_map(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.flatMap called on non-object")
        })?;
        // 3. If IsCallable(mapper) is false, throw a TypeError.
        let mapper = args.get_or_undefined(0);
        let mapper_obj = mapper.as_callable().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Iterator.prototype.flatMap requires a callable mapper")
        })?;
        // 4. Let iterated be ? GetIteratorDirect(O).
        let iterated = Self::get_iterator_direct(o.clone(), context)?;
        // 5-. Create IteratorHelper with FlatMap state.
        let helper = IteratorHelper::create_flat_map(iterated, mapper_obj.clone(), context);
        Ok(helper.into())
    }

    // -------------------------------------------------------------------------
    // Eager %Iterator.prototype% methods

    /// `Iterator.prototype.reduce ( reducer [ , initialValue ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.reduce
    pub(crate) fn reduce(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.reduce called on non-object")
        })?;
        // 3. If IsCallable(reducer) is false, throw a TypeError.
        let reducer = args.get_or_undefined(0);
        let reducer_obj = reducer.as_callable().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Iterator.prototype.reduce requires a callable reducer")
        })?;
        // 4. Let iterated be ? GetIteratorDirect(O).
        let mut iterated = Self::get_iterator_direct(o.clone(), context)?;
        // 5. If initialValue is not present, then
        let (mut accumulator, mut counter) = if args.len() < 2 {
            // a. Let next be ? IteratorStepValue(iterated).
            let Some(first) = iterated.step_value(context)? else {
                // b. If next is done, throw a TypeError.
                return Err(JsNativeError::typ()
                    .with_message("Iterator.prototype.reduce called on empty iterator without initialValue")
                    .into());
            };
            // c. Set accumulator to next.
            (first, 1.0f64)
        } else {
            (args[1].clone(), 0.0f64)
        };
        // 6. Repeat,
        loop {
            // a. Let value be ? IteratorStepValue(iterated).
            let Some(value) = iterated.step_value(context)? else {
                // b. If value is done, return accumulator.
                return Ok(accumulator);
            };
            // c. Let result be Completion(Call(reducer, undefined, « accumulator, value, 𝔽(counter) »)).
            let result = reducer_obj.call(
                &JsValue::undefined(),
                &[accumulator, value, counter.into()],
                context,
            );
            // d. IfAbruptCloseIterator(result, iterated).
            accumulator = if_abrupt_close_iterator!(result, iterated, context);
            // e. Set counter to counter + 1.
            counter += 1.0;
        }
    }

    /// `Iterator.prototype.toArray ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.toarray
    pub(crate) fn to_array(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.toArray called on non-object")
        })?;
        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = Self::get_iterator_direct(o.clone(), context)?;
        // 4. Let items be a new empty List.
        // 5. Repeat, let value be ? IteratorStepValue(iterated). If done, return CreateArrayFromList(items).
        let items = iterated.into_list(context)?;
        Ok(Array::create_array_from_list(items, context).into())
    }

    /// `Iterator.prototype.forEach ( fn )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.foreach
    pub(crate) fn for_each(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.forEach called on non-object")
        })?;
        // 3. If IsCallable(fn) is false, throw a TypeError.
        let fn_arg = args.get_or_undefined(0);
        let fn_obj = fn_arg.as_callable().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Iterator.prototype.forEach requires a callable function")
        })?;
        // 4. Let iterated be ? GetIteratorDirect(O).
        let mut iterated = Self::get_iterator_direct(o.clone(), context)?;
        // 5. Let counter be 0. 6. Repeat,
        let mut counter = 0.0f64;
        loop {
            // a. Let value be ? IteratorStepValue(iterated).
            let Some(value) = iterated.step_value(context)? else {
                // b. If done, return undefined.
                return Ok(JsValue::undefined());
            };
            // c. Let result be Completion(Call(fn, undefined, « value, 𝔽(counter) »)).
            let result =
                fn_obj.call(&JsValue::undefined(), &[value, counter.into()], context);
            // d. IfAbruptCloseIterator(result, iterated).
            if_abrupt_close_iterator!(result, iterated, context);
            // e. Set counter to counter + 1.
            counter += 1.0;
        }
    }

    /// `Iterator.prototype.some ( predicate )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.some
    pub(crate) fn some(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.some called on non-object")
        })?;
        // 3. If IsCallable(predicate) is false, throw a TypeError.
        let predicate = args.get_or_undefined(0);
        let predicate_obj = predicate.as_callable().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Iterator.prototype.some requires a callable predicate")
        })?;
        // 4. Let iterated be ? GetIteratorDirect(O).
        let mut iterated = Self::get_iterator_direct(o.clone(), context)?;
        // 5. Let counter be 0. 6. Repeat,
        let mut counter = 0.0f64;
        loop {
            // a. Let value be ? IteratorStepValue(iterated).
            let Some(value) = iterated.step_value(context)? else {
                // b. If done, return false.
                return Ok(JsValue::new(false));
            };
            // c. Let result be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).
            let result = predicate_obj.call(
                &JsValue::undefined(),
                &[value, counter.into()],
                context,
            );
            // d. IfAbruptCloseIterator(result, iterated).
            let result = if_abrupt_close_iterator!(result, iterated, context);
            // e. If ToBoolean(result) is true, return ? IteratorClose(iterated, NormalCompletion(true)).
            if result.to_boolean() {
                return iterated
                    .close(Ok(JsValue::new(true)), context)
                    .map(|_| JsValue::new(true));
            }
            // f. Set counter to counter + 1.
            counter += 1.0;
        }
    }

    /// `Iterator.prototype.every ( predicate )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.every
    pub(crate) fn every(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.every called on non-object")
        })?;
        // 3. If IsCallable(predicate) is false, throw a TypeError.
        let predicate = args.get_or_undefined(0);
        let predicate_obj = predicate.as_callable().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Iterator.prototype.every requires a callable predicate")
        })?;
        // 4. Let iterated be ? GetIteratorDirect(O).
        let mut iterated = Self::get_iterator_direct(o.clone(), context)?;
        // 5. Let counter be 0. 6. Repeat,
        let mut counter = 0.0f64;
        loop {
            // a. Let value be ? IteratorStepValue(iterated).
            let Some(value) = iterated.step_value(context)? else {
                // b. If done, return true.
                return Ok(JsValue::new(true));
            };
            // c. Let result be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).
            let result = predicate_obj.call(
                &JsValue::undefined(),
                &[value, counter.into()],
                context,
            );
            // d. IfAbruptCloseIterator(result, iterated).
            let result = if_abrupt_close_iterator!(result, iterated, context);
            // e. If ToBoolean(result) is false, return ? IteratorClose(iterated, NormalCompletion(false)).
            if !result.to_boolean() {
                return iterated
                    .close(Ok(JsValue::new(false)), context)
                    .map(|_| JsValue::new(false));
            }
            // f. Set counter to counter + 1.
            counter += 1.0;
        }
    }

    /// `Iterator.prototype.find ( predicate )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.find
    pub(crate) fn find(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.find called on non-object")
        })?;
        // 3. If IsCallable(predicate) is false, throw a TypeError.
        let predicate = args.get_or_undefined(0);
        let predicate_obj = predicate.as_callable().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Iterator.prototype.find requires a callable predicate")
        })?;
        // 4. Let iterated be ? GetIteratorDirect(O).
        let mut iterated = Self::get_iterator_direct(o.clone(), context)?;
        // 5. Let counter be 0. 6. Repeat,
        let mut counter = 0.0f64;
        loop {
            // a. Let value be ? IteratorStepValue(iterated).
            let Some(value) = iterated.step_value(context)? else {
                // b. If done, return undefined.
                return Ok(JsValue::undefined());
            };
            // c. Let result be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).
            let result = predicate_obj.call(
                &JsValue::undefined(),
                &[value.clone(), counter.into()],
                context,
            );
            // d. IfAbruptCloseIterator(result, iterated).
            let result = if_abrupt_close_iterator!(result, iterated, context);
            // e. If ToBoolean(result) is true, return ? IteratorClose(iterated, NormalCompletion(value)).
            if result.to_boolean() {
                return iterated
                    .close(Ok(value.clone()), context)
                    .map(|_| value);
            }
            // f. Set counter to counter + 1.
            counter += 1.0;
        }
    }
}
