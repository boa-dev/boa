use std::{cell::Cell, ops::ControlFlow};

use boa_gc::{Finalize, Trace};

use crate::{
    JsExpect, JsObject, JsValue,
    builtins::{
        Array,
        iterable::{IteratorRecord, iterator_close_all},
    },
    js_error,
    native_function::{CoroutineBranch, CoroutineState, NativeCoroutine},
    property::PropertyKey,
    vm::CompletionRecord,
};

/// The mode for zip iteration.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Trace, Finalize)]
#[boa_gc(empty_trace)]
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
    Keyed(Vec<PropertyKey>),
}

#[derive(Trace, Finalize, Default)]
#[boa_gc(unsafe_no_drop)]
pub(crate) enum Zip {
    #[default]
    Completed,
    Yielding {
        iters: Vec<Option<IteratorRecord>>,
        mode: ZipMode,
        padding: Vec<JsValue>,
        result_kind: ZipResultKind,
    },
}

impl Zip {
    /// [`IteratorZip ( iters, mode, padding, finishResults )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorzip
    #[allow(
        clippy::new_ret_no_self,
        reason = "slightly cleaner to have this be a `new` method"
    )]
    pub(crate) fn new(
        iters: Vec<IteratorRecord>,
        mode: ZipMode,
        padding: Vec<JsValue>,
        result_kind: ZipResultKind,
    ) -> NativeCoroutine {
        let iters = iters.into_iter().map(Some).collect();

        NativeCoroutine::from_copy_closure_with_captures(
            |completion, state, context| {
                let st = state.take();
                let (mut iters, mode, padding, result_kind) = match st {
                    Self::Completed => return CoroutineState::Break(Ok(())),
                    Self::Yielding {
                        iters,
                        mode,
                        padding,
                        result_kind,
                    } => (iters, mode, padding, result_kind),
                };
                // 3.a. If iterCount = 0, return ReturnCompletion(undefined).
                if iters.is_empty() {
                    return ControlFlow::Break(Ok(()));
                }
                // 3.b.v. Let completion be Completion(Yield(results)).
                // 3.b.vi. IfAbruptCloseIterators(completion, openIters).
                match completion {
                    CompletionRecord::Return(v) => {
                        return ControlFlow::Break(iterator_close_all(
                            iters.into_iter().flatten(),
                            Ok(v),
                            context,
                        ));
                    }
                    CompletionRecord::Throw(e) => {
                        return ControlFlow::Break(iterator_close_all(
                            iters.into_iter().flatten(),
                            Err(e),
                            context,
                        ));
                    }
                    CompletionRecord::Normal(_) => {}
                }

                // 3.b. Repeat,
                // 3.b.i. Let results be a new empty List.
                // 3.b.ii. Assert: openIters is not empty.
                let mut results = Vec::new();
                // 3.b.iii. For each integer i such that 0 ≤ i < iterCount, in ascending order, do
                for i in 0..iters.len() {
                    // 3.b.iii.1. Let iter be iters[i].
                    let iter = iters
                        .get_mut(i)
                        .js_expect("should be in range of iters")
                        .branch()?;
                    // 3.b.iii.2. If iter is null, then
                    let Some(iter_inner) = iter else {
                        // 3.b.iii.2.a. Assert: mode is "longest".
                        // 3.b.iii.2.b. Let result be padding[i].
                        // 3.b.iii.4. Append result to results.
                        results.push(
                            padding
                                .get(i)
                                .js_expect("should be in range of padding")
                                .branch()?
                                .clone(),
                        );
                        continue;
                    };
                    // 3.b.iii.3. Else,
                    // 3.b.iii.3.a. Let result be Completion(IteratorStepValue(iter)).
                    let result = match iter_inner.step_value(context) {
                        // 3.b.iii.3.c. Set result to ! result.
                        Ok(v) => v,
                        // 3.b.iii.3.b. If result is an abrupt completion, then
                        Err(err) => {
                            // 3.b.iii.3.b.i. Remove iter from openIters.
                            // 3.b.iii.3.b.ii. Return ? IteratorCloseAll(openIters, result).
                            iter.take();
                            return ControlFlow::Break(iterator_close_all(
                                iters.into_iter().flatten(),
                                Err(err),
                                context,
                            ));
                        }
                    };

                    if let Some(result) = result {
                        results.push(result);
                        continue;
                    }

                    // 3.b.iii.3.d. If result is done, then
                    // 3.b.iii.3.d.i. Remove iter from openIters.
                    iter.take();
                    match mode {
                        // 3.b.iii.3.d.ii. If mode is "shortest", then
                        ZipMode::Shortest => {
                            // 3.b.iii.3.d.ii.1. Return ? IteratorCloseAll(openIters, ReturnCompletion(undefined)).
                            return ControlFlow::Break(iterator_close_all(
                                iters.into_iter().flatten(),
                                Ok(JsValue::undefined()),
                                context,
                            ));
                        }
                        // 3.b.iii.3.d.iii. Else if mode is "strict", then
                        ZipMode::Strict => {
                            // 3.b.iii.3.d.iii.1. If i ≠ 0, then
                            if i != 0 {
                                // 3.b.iii.3.d.iii.1.a. Return ?IteratorCloseAll(openIters, ThrowCompletion(a newly created TypeError object)).
                                return ControlFlow::Break(iterator_close_all(
                                    iters.into_iter().flatten(),
                                    Err(js_error!(
                                        TypeError:
                                        r#"zip iterator on "strict" mode requires iterators with the same length"#
                                    )),
                                    context,
                                ));
                            }
                            // 3.b.iii.3.d.iii.2. For each integer k such that 1 ≤ k < iterCount, in ascending order, do
                            for i in 1..iters.len() {
                                // 3.b.iii.3.d.iii.2.a. Assert: iters[k] is not null.
                                let Some(iter) = iters.get_mut(i) else {
                                    continue;
                                };
                                let Some(inner_iter) = iter else {
                                    continue;
                                };
                                // 3.b.iii.3.d.iii.2.b. Let open be Completion(IteratorStep(iters[k])).
                                let open = match inner_iter.step(context) {
                                    Ok(v) => v,
                                    // 3.b.iii.3.d.iii.2.c. If open is an abrupt completion, then
                                    Err(err) => {
                                        // 3.b.iii.3.d.iii.2.c.i. Remove iters[k] from openIters.
                                        iter.take();
                                        // 3.b.iii.3.d.iii.2.c.ii. Return ? IteratorCloseAll(openIters, open).
                                        return ControlFlow::Break(iterator_close_all(
                                            iters.into_iter().flatten(),
                                            Err(err),
                                            context,
                                        ));
                                    }
                                };
                                // 3.b.iii.3.d.iii.2.d. Set open to ! open.
                                // 3.b.iii.3.d.iii.2.e. If open is done, then
                                if open {
                                    // 3.b.iii.3.d.iii.2.e.i. Remove iters[k] from openIters.
                                    iter.take();
                                }
                                // 3.b.iii.3.d.iii.2.f. Else,
                                else {
                                    // 3.b.iii.3.d.iii.2.f.i. Return ? IteratorCloseAll(openIters, ThrowCompletion(a newly created TypeError object)).
                                    return ControlFlow::Break(iterator_close_all(
                                        iters.into_iter().flatten(),
                                        Err(js_error!(
                                            TypeError:
                                            r#"zip iterator on "strict" mode requires iterators with the same length"#
                                        )),
                                        context,
                                    ));
                                }
                            }

                            // 3.b.iii.3.d.iii.3. Return ReturnCompletion(undefined).
                            return ControlFlow::Break(Ok(()));
                        }
                        // 3.b.iii.3.d.iv. Else,
                        ZipMode::Longest => {
                            // 3.b.iii.3.d.iv.1. Assert: mode is "longest".
                            // 3.b.iii.3.d.iv.2. If openIters is empty, return ReturnCompletion(undefined).
                            if iters.iter().all(Option::is_none) {
                                return ControlFlow::Break(Ok(()));
                            }
                            // 3.b.iii.3.d.iv.3. Set iters[i] to null.
                            // 3.b.iii.3.d.iv.4. Set result to padding[i].
                            results.push(
                                padding
                                    .get(i)
                                    .js_expect("should be in range of padding")
                                    .branch()?
                                    .clone(),
                            );
                        }
                    }
                }
                // 3.b.iv. Set results to finishResults(results).
                // 3.b.v. Let completion be Completion(Yield(results)).
                let results = match &result_kind {
                    ZipResultKind::Array => {
                        // `Iterator.zip ( iterables [ , options ] )`
                        // 15.a. Return CreateArrayFromList(results).
                        Array::create_array_from_list(results, context)
                    }
                    ZipResultKind::Keyed(keys) => {
                        // `Iterator.zipKeyed ( iterables [ , options ] )`
                        // 15.a. Let obj be OrdinaryObjectCreate(null).
                        let obj = JsObject::with_null_proto();

                        // 15.b. For each integer i such that 0 ≤ i < iterCount, in ascending order, do
                        for (key, value) in keys.iter().zip(results) {
                            // 15.b.i. Perform ! CreateDataPropertyOrThrow(obj, keys[i], results[i]).
                            obj.create_data_property_or_throw(key.clone(), value, context)
                                .js_expect("cannot fail per the spec")
                                .branch()?;
                        }
                        // 15.c. Return obj.
                        obj
                    }
                };
                state.set(Self::Yielding {
                    iters,
                    mode,
                    padding,
                    result_kind,
                });
                ControlFlow::Continue(results.into())
            },
            Cell::new(Self::Yielding {
                iters,
                mode,
                padding,
                result_kind,
            }),
        )
    }
}
