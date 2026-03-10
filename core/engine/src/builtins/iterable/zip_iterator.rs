//! This module implements the `ZipIterator` object backing `Iterator.zip` and `Iterator.zipKeyed`.
//!
//! More information:
//!  - [TC39 proposal][proposal]
//!
//! [proposal]: https://tc39.es/proposal-joint-iteration/

use crate::{
    Context, JsData, JsResult, JsValue,
    builtins::{
        Array, BuiltInBuilder, IntrinsicObject,
        iterable::{IteratorRecord, create_iter_result_object},
    },
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};
use crate::property::PropertyKey;

/// The mode for zip iteration.
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub(crate) enum ZipMode {
    /// Stops when the shortest iterator is done.
    Shortest,
    /// Continues until the longest iterator is done, padding with `undefined` or user values.
    Longest,
    /// All iterators must have the same length, otherwise throws a `TypeError`.
    Strict,
}

/// The kind of result to produce from the zip iterator.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) enum ZipResultKind {
    /// Produces arrays (for `Iterator.zip`).
    Array,
    /// Produces objects with the given keys (for `Iterator.zipKeyed`).
    Keyed(Vec<JsValue>),
}

/// The `ZipIterator` object represents a joint iteration over multiple iterators.
///
/// It implements the iterator protocol and is returned by `Iterator.zip()` and `Iterator.zipKeyed()`.
///
/// More information:
///  - [TC39 proposal][proposal]
///
/// [proposal]: https://tc39.es/proposal-joint-iteration/
#[derive(Debug, Finalize, Trace, JsData)]
pub(crate) struct ZipIterator {
    /// The list of underlying iterator records. An entry is set to `None` when exhausted
    /// (only relevant in "longest" mode).
    iters: Vec<Option<IteratorRecord>>,

    /// The total number of iterators (does not change when iterators are exhausted).
    iter_count: usize,

    /// The list of iterators that are still open (indices into `iters`).
    /// When this becomes empty in "longest" mode, iteration is done.
    open_iters: Vec<usize>,

    /// The iteration mode.
    #[unsafe_ignore_trace]
    mode: ZipMode,

    /// Padding values for "longest" mode.
    padding: Vec<JsValue>,

    /// What kind of result object to produce.
    result_kind: ZipResultKind,

    /// Whether the iterator has been completed.
    done: bool,
}

impl IntrinsicObject for ZipIterator {
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
                js_string!("Iterator Helper"),
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().iterator()
    }
}

impl ZipIterator {
    /// Creates a new `ZipIterator`.
    pub(crate) fn new(
        iters: Vec<IteratorRecord>,
        mode: ZipMode,
        padding: Vec<JsValue>,
        result_kind: ZipResultKind,
    ) -> Self {
        let iter_count = iters.len();
        let open_iters: Vec<usize> = (0..iter_count).collect();
        let iters = iters.into_iter().map(Some).collect();
        Self {
            iters,
            iter_count,
            open_iters,
            mode,
            padding,
            result_kind,
            done: false,
        }
    }

    /// Creates a `ZipIterator` JS object and wraps it as a `JsValue`.
    pub(crate) fn create_zip_iterator(
        iters: Vec<IteratorRecord>,
        mode: ZipMode,
        padding: Vec<JsValue>,
        result_kind: ZipResultKind,
        context: &mut Context,
    ) -> JsValue {
        let zip_iter = Self::new(iters, mode, padding, result_kind);
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .iterator(),
            zip_iter,
        );
        obj.into()
    }

    /// Closes all open iterators with the given completion.
    fn close_all(
        iters: &mut Vec<Option<IteratorRecord>>,
        open_iters: &[usize],
        completion: JsResult<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let mut result = completion;
        for &idx in open_iters {
            if let Some(iter) = iters[idx].take() {
                let close_result = iter.close(Ok(JsValue::undefined()), context);
                if result.is_ok() && close_result.is_err() {
                    result = close_result;
                }
            }
        }
        result
    }

    /// Builds the result value from a list of values based on the `ZipResultKind`.
    fn finish_results(
        results: &[JsValue],
        result_kind: &ZipResultKind,
        context: &mut Context,
    ) -> JsValue {
        match result_kind {
            ZipResultKind::Array => {
                // CreateArrayFromList(results)
                Array::create_array_from_list(results.to_vec(), context).into()
            }
            ZipResultKind::Keyed(keys) => {
                // Create a null-prototype object with keys mapped to results.
                let obj = JsObject::with_null_proto();
                for (i, key) in keys.iter().enumerate() {
                    if let Some(val) = results.get(i) {
                        let prop_key: PropertyKey = key.to_string(context)
                            .unwrap_or_default()
                            .into();
                        obj.set(prop_key, val.clone(), false, context)
                            .expect("setting property on new object should not fail");
                    }
                }
                obj.into()
            }
        }
    }

    /// `%ZipIteratorPrototype%.next()`
    ///
    /// Implements the IteratorZip abstract operation from the TC39 Joint Iteration proposal.
    ///
    /// More information:
    ///  - [TC39 proposal][proposal]
    ///
    /// [proposal]: https://tc39.es/proposal-joint-iteration/#sec-iteratorzip
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("`this` is not a ZipIterator")
        })?;

        let mut zip_iter = obj.downcast_mut::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("`this` is not a ZipIterator")
        })?;

        // If already done, return { value: undefined, done: true }
        if zip_iter.done {
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }

        // Step 1: If iterCount = 0, return done.
        if zip_iter.iter_count == 0 {
            zip_iter.done = true;
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }

        let mode = zip_iter.mode.clone();
        let iter_count = zip_iter.iter_count;

        let mut results: Vec<JsValue> = Vec::with_capacity(iter_count);

        // Step 2: For each integer i such that 0 ≤ i < iterCount, in ascending order, do
        for i in 0..iter_count {
            if zip_iter.iters[i].is_none() {
                // iter is null → assert mode is "longest", use padding[i]
                debug_assert!(mode == ZipMode::Longest);
                results.push(
                    zip_iter
                        .padding
                        .get(i)
                        .cloned()
                        .unwrap_or(JsValue::undefined()),
                );
                continue;
            }

            // Let result be Completion(IteratorStepValue(iter))
            let iter = zip_iter.iters[i].as_mut().unwrap();
            let step_result = iter.step_value(context);

            match step_result {
                Err(err) => {
                    // If result is an abrupt completion:
                    // Remove iter from openIters.
                    zip_iter.open_iters.retain(|&idx| idx != i);
                    zip_iter.iters[i] = None;
                    zip_iter.done = true;
                    // Return ? IteratorCloseAll(openIters, result).
                    let open = zip_iter.open_iters.clone();
                    return Self::close_all(
                        &mut zip_iter.iters,
                        &open,
                        Err(err),
                        context,
                    );
                }
                Ok(None) => {
                    // result is done.
                    // Remove iter from openIters.
                    zip_iter.open_iters.retain(|&idx| idx != i);
                    zip_iter.iters[i] = None;

                    match mode {
                        ZipMode::Shortest => {
                            // Return ? IteratorCloseAll(openIters, ReturnCompletion(undefined)).
                            zip_iter.done = true;
                            let open = zip_iter.open_iters.clone();
                            return Self::close_all(
                                &mut zip_iter.iters,
                                &open,
                                Ok(JsValue::undefined()),
                                context,
                            )
                            .and_then(|_| {
                                Ok(create_iter_result_object(
                                    JsValue::undefined(),
                                    true,
                                    context,
                                ))
                            });
                        }
                        ZipMode::Strict => {
                            if i != 0 {
                                // If i ≠ 0, throw TypeError after closing all.
                                zip_iter.done = true;
                                let open = zip_iter.open_iters.clone();
                                let _ = Self::close_all(
                                    &mut zip_iter.iters,
                                    &open,
                                    Ok(JsValue::undefined()),
                                    context,
                                );
                                return Err(JsNativeError::typ()
                                    .with_message(
                                        "iterators have different lengths in strict mode",
                                    )
                                    .into());
                            }

                            // i == 0: Check that all remaining iterators are also done.
                            for k in 1..iter_count {
                                if zip_iter.iters[k].is_none() {
                                    continue;
                                }
                                let other = zip_iter.iters[k].as_mut().unwrap();
                                let step = other.step(context);
                                match step {
                                    Err(err) => {
                                        zip_iter.open_iters.retain(|&idx| idx != k);
                                        zip_iter.iters[k] = None;
                                        zip_iter.done = true;
                                        let open = zip_iter.open_iters.clone();
                                        return Self::close_all(
                                            &mut zip_iter.iters,
                                            &open,
                                            Err(err),
                                            context,
                                        );
                                    }
                                    Ok(is_done) => {
                                        if is_done {
                                            // done → remove from openIters
                                            zip_iter.open_iters.retain(|&idx| idx != k);
                                            zip_iter.iters[k] = None;
                                        } else {
                                            // Not done → length mismatch, throw TypeError
                                            zip_iter.done = true;
                                            let open = zip_iter.open_iters.clone();
                                            let _ = Self::close_all(
                                                &mut zip_iter.iters,
                                                &open,
                                                Ok(JsValue::undefined()),
                                                context,
                                            );
                                            return Err(JsNativeError::typ()
                                                .with_message(
                                                    "iterators have different lengths in strict mode",
                                                )
                                                .into());
                                        }
                                    }
                                }
                            }
                            // All done → return done.
                            zip_iter.done = true;
                            return Ok(create_iter_result_object(
                                JsValue::undefined(),
                                true,
                                context,
                            ));
                        }
                        ZipMode::Longest => {
                            // If openIters is empty, return done.
                            if zip_iter.open_iters.is_empty() {
                                zip_iter.done = true;
                                return Ok(create_iter_result_object(
                                    JsValue::undefined(),
                                    true,
                                    context,
                                ));
                            }
                            // Set iters[i] to null, use padding[i].
                            results.push(
                                zip_iter
                                    .padding
                                    .get(i)
                                    .cloned()
                                    .unwrap_or(JsValue::undefined()),
                            );
                        }
                    }
                }
                Ok(Some(value)) => {
                    results.push(value);
                }
            }
        }

        // finishResults(results)
        let result_kind = zip_iter.result_kind.clone();
        let finished = Self::finish_results(&results, &result_kind, context);

        // Yield(results) → return { value: results, done: false }
        Ok(create_iter_result_object(finished, false, context))
    }

    /// `%ZipIteratorPrototype%.return()`
    ///
    /// Closes all underlying iterators.
    pub(crate) fn r#return(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("`this` is not a ZipIterator")
        })?;

        let mut zip_iter = obj.downcast_mut::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("`this` is not a ZipIterator")
        })?;

        zip_iter.done = true;
        let open = zip_iter.open_iters.clone();
        Self::close_all(
            &mut zip_iter.iters,
            &open,
            Ok(JsValue::undefined()),
            context,
        )?;
        zip_iter.open_iters.clear();

        Ok(create_iter_result_object(
            JsValue::undefined(),
            true,
            context,
        ))
    }
}
