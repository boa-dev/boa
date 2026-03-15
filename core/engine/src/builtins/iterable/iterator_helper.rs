//! Boa's implementation of the `Iterator Helper` objects.
//!
//! An Iterator Helper object is an ordinary object that represents a lazy transformation
//! of some specific source iterator object. There is not a named constructor for
//! Iterator Helper objects. Instead, Iterator Helper objects are created by calling
//! certain methods of Iterator instance objects.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-iterator-helper-objects

use crate::{
    Context, JsData, JsResult, JsValue,
    builtins::{BuiltInBuilder, IntrinsicObject, iterable::create_iter_result_object},
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};

use super::IteratorRecord;

/// The type of lazy iterator helper operation.
#[derive(Debug, Trace, Finalize)]
pub(crate) enum IteratorHelperOp {
    /// `Iterator.prototype.map(mapper)` — yields `mapper(value, counter)`.
    Map { mapper: JsObject, counter: u64 },
    /// `Iterator.prototype.filter(predicate)` — yields values where `predicate(value, counter)` is truthy.
    Filter { predicate: JsObject, counter: u64 },
    /// `Iterator.prototype.take(limit)` — yields at most `limit` values.
    Take { remaining: u64 },
    /// `Iterator.prototype.drop(limit)` — skips `limit` values, then yields the rest.
    Drop { remaining: u64, done_dropping: bool },
    /// `Iterator.prototype.flatMap(mapper)` — yields values from the iterators returned by `mapper`.
    FlatMap {
        mapper: JsObject,
        counter: u64,
        /// The inner iterator from the most recent call to `mapper`, if any.
        inner_iterator: Option<IteratorRecord>,
    },
}

/// Represents the state of an `IteratorHelper`'s internal generator.
#[derive(Debug, Trace, Finalize)]
pub(crate) enum IteratorHelperState {
    /// The helper has been created but `.next()` has not yet been called.
    SuspendedStart,
    /// The helper has yielded a value and is waiting for the next `.next()` call.
    SuspendedYield,
    /// The helper's `.next()` is currently executing.
    Executing,
    /// The helper has finished (either exhausted the underlying iterator or was closed).
    Completed,
}

/// The internal representation of an `Iterator Helper` object.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator-helper-objects
#[derive(Debug, Finalize, Trace, JsData)]
pub(crate) struct IteratorHelper {
    /// `[[UnderlyingIterator]]` — the iterator record for the source iterator.
    pub(crate) underlying_iterator: IteratorRecord,

    /// `[[GeneratorState]]` — tracks the state of this helper's internal generator.
    pub(crate) state: IteratorHelperState,

    /// The specific lazy operation this helper performs.
    pub(crate) op: IteratorHelperOp,
}

impl IntrinsicObject for IteratorHelper {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(realm.intrinsics().constructors().iterator().prototype())
            .static_method(Self::next, js_string!("next"), 0)
            .static_method(Self::r#return, js_string!("return"), 0)
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("Iterator Helper"),
                Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().iterator_helper()
    }
}

impl IteratorHelper {
    /// `%IteratorHelperPrototype%.next ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%25iteratorhelperprototype%25.next
    pub(crate) fn next(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Return ? GeneratorResume(this value, undefined, "Iterator Helper").
        let object = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator Helper method called on non-object")
        })?;

        let mut helper = object.downcast_mut::<Self>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Iterator Helper method called on incompatible object")
        })?;

        match helper.state {
            IteratorHelperState::Executing => {
                return Err(JsNativeError::typ()
                    .with_message("Iterator Helper is already executing")
                    .into());
            }
            IteratorHelperState::Completed => {
                return Ok(create_iter_result_object(
                    JsValue::undefined(),
                    true,
                    context,
                ));
            }
            IteratorHelperState::SuspendedStart | IteratorHelperState::SuspendedYield => {}
        }

        // Set state to Executing (re-entrancy guard).
        helper.state = IteratorHelperState::Executing;
        drop(helper);

        // Execute the closure based on the operation type.
        // Returns Ok((result_object, done)) so we can update state without re-reading
        // the "done" field via a property getter (which could trigger arbitrary JS).
        let result = Self::execute_next(&object, context);

        // Post-execution: update state based on result.
        let mut helper = object
            .downcast_mut::<Self>()
            .expect("object type already verified");

        match &result {
            Ok((_, done)) => {
                if *done {
                    helper.state = IteratorHelperState::Completed;
                } else {
                    helper.state = IteratorHelperState::SuspendedYield;
                }
            }
            Err(_) => {
                helper.state = IteratorHelperState::Completed;
            }
        }

        result.map(|(val, _)| val)
    }

    /// Execute one step of the iterator helper closure.
    /// Returns `Ok((result_object, done))` where `done` is `true` when the iteration
    /// is complete. Using a dedicated flag avoids re-reading the `done` property via
    /// a property getter, which would trigger arbitrary user code.
    fn execute_next(object: &JsObject, context: &mut Context) -> JsResult<(JsValue, bool)> {
        // Map arm: extract + clone what we need from op, then separately borrow underlying_iterator
        {
            let mut helper = object
                .downcast_mut::<Self>()
                .expect("object type already verified");
            if let IteratorHelperOp::Map { mapper, counter } = &mut helper.op {
                let mapper = mapper.clone();
                let count = *counter;
                *counter += 1;
                drop(helper);

                let mut helper = object
                    .downcast_mut::<Self>()
                    .expect("object type already verified");
                let iterated = &mut helper.underlying_iterator;
                let value = iterated.step_value(context)?;
                match value {
                    None => {
                        return Ok((
                            create_iter_result_object(JsValue::undefined(), true, context),
                            true,
                        ));
                    }
                    Some(value) => {
                        let mapper_result = mapper.call(
                            &JsValue::undefined(),
                            &[value, JsValue::new(count)],
                            context,
                        );
                        return match mapper_result {
                            Ok(result_value) => Ok((
                                create_iter_result_object(result_value, false, context),
                                false,
                            )),
                            Err(err) => {
                                drop(iterated.close(Err(err.clone()), context));
                                Err(err)
                            }
                        };
                    }
                }
            }
        }

        // Filter arm
        {
            let helper = object
                .downcast_mut::<Self>()
                .expect("object type already verified");
            if let IteratorHelperOp::Filter { predicate, .. } = &helper.op {
                let predicate = predicate.clone();
                // We'll loop; snapshot/increment counter each iteration.
                // We can't hold a borrow across context calls, so snapshot counter now,
                // then re-borrow each iteration.
                drop(helper);

                loop {
                    let count = {
                        let mut h = object
                            .downcast_mut::<Self>()
                            .expect("object type already verified");
                        let IteratorHelperOp::Filter { counter, .. } = &mut h.op else {
                            unreachable!()
                        };
                        let c = *counter;
                        *counter += 1;
                        c
                    };

                    let mut helper = object
                        .downcast_mut::<Self>()
                        .expect("object type already verified");
                    let iterated = &mut helper.underlying_iterator;
                    let value = iterated.step_value(context)?;
                    match value {
                        None => {
                            return Ok((
                                create_iter_result_object(JsValue::undefined(), true, context),
                                true,
                            ));
                        }
                        Some(value) => {
                            let selected_result = predicate.call(
                                &JsValue::undefined(),
                                &[value.clone(), JsValue::new(count)],
                                context,
                            );
                            match selected_result {
                                Ok(selected) => {
                                    if selected.to_boolean() {
                                        return Ok((
                                            create_iter_result_object(value, false, context),
                                            false,
                                        ));
                                    }
                                }
                                Err(err) => {
                                    drop(iterated.close(Err(err.clone()), context));
                                    return Err(err);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Take arm
        {
            let mut helper = object
                .downcast_mut::<Self>()
                .expect("object type already verified");
            if let IteratorHelperOp::Take { remaining } = &mut helper.op {
                if *remaining == 0 {
                    drop(helper);
                    let helper = object
                        .downcast_mut::<Self>()
                        .expect("object type already verified");
                    let close_result = helper
                        .underlying_iterator
                        .close(Ok(JsValue::undefined()), context);
                    drop(helper);
                    close_result?;
                    return Ok((
                        create_iter_result_object(JsValue::undefined(), true, context),
                        true,
                    ));
                }
                *remaining -= 1;
                drop(helper);

                let mut helper = object
                    .downcast_mut::<Self>()
                    .expect("object type already verified");
                let value = helper.underlying_iterator.step_value(context)?;
                return match value {
                    None => Ok((
                        create_iter_result_object(JsValue::undefined(), true, context),
                        true,
                    )),
                    Some(v) => Ok((create_iter_result_object(v, false, context), false)),
                };
            }
        }

        // Drop arm
        {
            let mut helper = object
                .downcast_mut::<Self>()
                .expect("object type already verified");
            if let IteratorHelperOp::Drop {
                remaining,
                done_dropping,
            } = &mut helper.op
            {
                let should_skip = !*done_dropping;
                let skip_count = *remaining;
                if should_skip {
                    *done_dropping = true;
                    *remaining = 0;
                }
                drop(helper);

                if should_skip {
                    for _ in 0..skip_count {
                        let mut helper = object
                            .downcast_mut::<Self>()
                            .expect("object type already verified");
                        let value = helper.underlying_iterator.step_value(context)?;
                        if value.is_none() {
                            return Ok((
                                create_iter_result_object(JsValue::undefined(), true, context),
                                true,
                            ));
                        }
                    }
                }

                let mut helper = object
                    .downcast_mut::<Self>()
                    .expect("object type already verified");
                let value = helper.underlying_iterator.step_value(context)?;
                return match value {
                    None => Ok((
                        create_iter_result_object(JsValue::undefined(), true, context),
                        true,
                    )),
                    Some(v) => Ok((create_iter_result_object(v, false, context), false)),
                };
            }
        }

        // FlatMap arm
        {
            loop {
                // Try to get value from active inner iterator first
                let has_inner = {
                    let helper = object
                        .downcast_mut::<Self>()
                        .expect("object type already verified");
                    if let IteratorHelperOp::FlatMap { inner_iterator, .. } = &helper.op {
                        inner_iterator.is_some()
                    } else {
                        false
                    }
                };

                if has_inner {
                    let mut helper = object
                        .downcast_mut::<Self>()
                        .expect("object type already verified");
                    let IteratorHelperOp::FlatMap { inner_iterator, .. } = &mut helper.op else {
                        unreachable!()
                    };
                    let inner = inner_iterator.as_mut().expect("checked above");
                    let inner_value = inner.step_value(context)?;
                    if let Some(val) = inner_value {
                        return Ok((create_iter_result_object(val, false, context), false));
                    }
                    // Inner exhausted — clear it.
                    drop(helper);
                    let mut helper = object
                        .downcast_mut::<Self>()
                        .expect("object type already verified");
                    let IteratorHelperOp::FlatMap { inner_iterator, .. } = &mut helper.op else {
                        unreachable!()
                    };
                    *inner_iterator = None;
                }

                // Get data from op then drop borrow, then access underlying_iterator
                let (mapper, count) = {
                    let mut helper = object
                        .downcast_mut::<Self>()
                        .expect("object type already verified");
                    let IteratorHelperOp::FlatMap {
                        mapper, counter, ..
                    } = &mut helper.op
                    else {
                        unreachable!()
                    };
                    let c = *counter;
                    *counter += 1;
                    (mapper.clone(), c)
                };

                let mut helper = object
                    .downcast_mut::<Self>()
                    .expect("object type already verified");
                let iterated = &mut helper.underlying_iterator;
                let value = iterated.step_value(context)?;

                match value {
                    None => {
                        return Ok((
                            create_iter_result_object(JsValue::undefined(), true, context),
                            true,
                        ));
                    }
                    Some(value) => {
                        let mapper_result = mapper.call(
                            &JsValue::undefined(),
                            &[value, JsValue::new(count)],
                            context,
                        );

                        let inner_value = match mapper_result {
                            Ok(m) => m,
                            Err(err) => {
                                drop(iterated.close(Err(err.clone()), context));
                                return Err(err);
                            }
                        };
                        drop(helper);

                        // Get inner iterator
                        let inner_record =
                            match super::get_iterator_flattenable(&inner_value, false, context) {
                                Ok(record) => record,
                                Err(err) => {
                                    let helper = object
                                        .downcast_mut::<Self>()
                                        .expect("object type already verified");
                                    drop(
                                        helper.underlying_iterator.close(Err(err.clone()), context),
                                    );
                                    return Err(err);
                                }
                            };

                        let mut helper = object
                            .downcast_mut::<Self>()
                            .expect("object type already verified");
                        let IteratorHelperOp::FlatMap { inner_iterator, .. } = &mut helper.op
                        else {
                            unreachable!()
                        };
                        *inner_iterator = Some(inner_record);
                        // Loop to get a value from the new inner iterator.
                    }
                }
            }
        }
    }

    /// `%IteratorHelperPrototype%.return ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%25iteratorhelperprototype%25.return
    pub(crate) fn r#return(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator Helper method called on non-object")
        })?;

        let mut helper = object.downcast_mut::<Self>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Iterator Helper method called on incompatible object")
        })?;

        match helper.state {
            // 4. If O.[[GeneratorState]] is suspended-start, then
            IteratorHelperState::SuspendedStart => {
                // a. Set O.[[GeneratorState]] to completed.
                helper.state = IteratorHelperState::Completed;

                // b. Perform ? IteratorClose(O.[[UnderlyingIterator]], NormalCompletion(unused)).
                let close_result = helper
                    .underlying_iterator
                    .close(Ok(JsValue::undefined()), context);
                drop(helper);
                close_result?;

                // c. Return CreateIterResultObject(undefined, true).
                Ok(create_iter_result_object(
                    JsValue::undefined(),
                    true,
                    context,
                ))
            }
            IteratorHelperState::SuspendedYield => {
                // Set state to completed and close the underlying iterator.
                helper.state = IteratorHelperState::Completed;

                let close_result = helper
                    .underlying_iterator
                    .close(Ok(JsValue::undefined()), context);
                drop(helper);
                close_result?;

                Ok(create_iter_result_object(
                    JsValue::undefined(),
                    true,
                    context,
                ))
            }
            IteratorHelperState::Executing => {
                // Re-entrancy: the closure is currently executing and called .return().
                // Set state to completed and close.
                helper.state = IteratorHelperState::Completed;

                let close_result = helper
                    .underlying_iterator
                    .close(Ok(JsValue::undefined()), context);
                drop(helper);
                close_result?;

                Ok(create_iter_result_object(
                    JsValue::undefined(),
                    true,
                    context,
                ))
            }
            IteratorHelperState::Completed => Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            )),
        }
    }

    /// Creates a new `IteratorHelper` object.
    pub(crate) fn create(
        underlying_iterator: IteratorRecord,
        op: IteratorHelperOp,
        context: &mut Context,
    ) -> JsObject {
        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .iterator_helper(),
            Self {
                underlying_iterator,
                state: IteratorHelperState::SuspendedStart,
                op,
            },
        )
        .upcast()
    }
}
