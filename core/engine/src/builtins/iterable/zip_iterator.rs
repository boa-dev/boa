//! This module implements the `ZipIterator` object backing `Iterator.zip` and `Iterator.zipKeyed`.
//!
//! More information:
//!  - [TC39 proposal][proposal]
//!
//! [proposal]: https://tc39.es/proposal-joint-iteration/

use crate::property::PropertyKey;
use crate::{
    Context, JsData, JsResult, JsValue,
    builtins::{
        Array, BuiltInBuilder, IntrinsicObject,
        iterable::{IteratorRecord, create_iter_result_object},
    },
    context::intrinsics::Intrinsics,
    error::{JsNativeError, PanicError},
    js_string,
    native_function::{CoroutineState, NativeCoroutine},
    object::JsObject,
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
    vm::CompletionRecord,
};
use boa_gc::{Finalize, Trace};
use std::cell::Cell;
use std::ops::ControlFlow;

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
    pub(crate) coroutine: Option<NativeCoroutine>,
}

impl IntrinsicObject for ZipIterator {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(realm.intrinsics().constructors().iterator().prototype())
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
        intrinsics.objects().iterator_prototypes().zip_iterator()
    }
}

impl ZipIterator {
    /// Creates a `ZipIterator` JS object and wraps it as a `JsValue`.
    pub(crate) fn create_zip_iterator(
        iters: Vec<IteratorRecord>,
        mode: ZipMode,
        padding: Vec<JsValue>,
        result_kind: ZipResultKind,
        context: &mut Context,
    ) -> JsValue {
        let op = Zip::new(iters, mode, padding, result_kind);
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .zip_iterator(),
            Self {
                coroutine: Some(op),
            },
        );
        obj.into()
    }

    #[track_caller]
    pub(crate) fn generator_validate(this: &JsValue) -> JsResult<JsObject<ZipIterator>> {
        this.as_object()
            .and_then(|o| o.downcast::<Self>().ok())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("ZipIterator method called on non-object")
                    .into()
            })
    }

    /// `%ZipIteratorPrototype%.next()`
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zip_iter = Self::generator_validate(this)?;
        let coroutine = zip_iter
            .borrow_mut()
            .data_mut()
            .coroutine
            .take()
            .ok_or_else(|| JsNativeError::typ().with_message("ZipIterator is already executing"))?;

        let result = match coroutine.call(CompletionRecord::Normal(JsValue::undefined()), context) {
            ControlFlow::Continue(value) => Ok(create_iter_result_object(value, false, context)),
            ControlFlow::Break(Ok(())) => Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            )),
            ControlFlow::Break(Err(err)) => Err(err),
        };

        zip_iter.borrow_mut().data_mut().coroutine = Some(coroutine);
        result
    }

    /// `%ZipIteratorPrototype%.return()`
    pub(crate) fn r#return(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let zip_iter = Self::generator_validate(this)?;
        let coroutine = zip_iter
            .borrow_mut()
            .data_mut()
            .coroutine
            .take()
            .ok_or_else(|| JsNativeError::typ().with_message("ZipIterator is already executing"))?;

        let result = match coroutine.call(CompletionRecord::Return(JsValue::undefined()), context) {
            ControlFlow::Continue(_) => {
                Err(PanicError::new("ZipIterator cannot yield after return").into())
            }
            ControlFlow::Break(Ok(())) => Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            )),
            ControlFlow::Break(Err(err)) => Err(err),
        };

        zip_iter.borrow_mut().data_mut().coroutine = Some(coroutine);
        result
    }
}

#[derive(Trace, Finalize, Default)]
#[boa_gc(unsafe_no_drop)]
pub(crate) enum Zip {
    #[default]
    Completed,
    Yielding {
        iters: Vec<Option<IteratorRecord>>,
        iter_count: usize,
        open_iters: Vec<usize>,
        mode: ZipMode,
        padding: Vec<JsValue>,
        result_kind: ZipResultKind,
    },
}

impl Zip {
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
        let iter_count = iters.len();
        let open_iters: Vec<usize> = (0..iter_count).collect();
        let iters = iters.into_iter().map(Some).collect();

        NativeCoroutine::from_copy_closure_with_captures(
            |completion, state, context| {
                let mut st = state.take();
                match &mut st {
                    Self::Yielding {
                        iters, open_iters, ..
                    } => {
                        let is_abrupt = matches!(
                            &completion,
                            CompletionRecord::Return(_) | CompletionRecord::Throw(_)
                        );
                        if is_abrupt {
                            let err_result: JsResult<()> = match completion {
                                CompletionRecord::Throw(err) => Err(err),
                                _ => Ok(()),
                            };
                            let mut final_result = err_result;
                            for &idx in open_iters.iter() {
                                if let Some(iter) = &mut iters[idx] {
                                    let close_result =
                                        iter.close(Ok(JsValue::undefined()), context);
                                    if final_result.is_ok() && close_result.is_err() {
                                        final_result = close_result.map(|_| ());
                                    }
                                }
                            }
                            return ControlFlow::Break(final_result);
                        }
                    }
                    Self::Completed => return ControlFlow::Break(Ok(())),
                }

                let Self::Yielding {
                    mut iters,
                    iter_count,
                    mut open_iters,
                    mode,
                    padding,
                    result_kind,
                } = st
                else {
                    unreachable!()
                };

                // 2. If iterCount = 0, return NormalCompletion(undefined).
                if iter_count == 0 {
                    return CoroutineState::Break(Ok(()));
                }

                // 3. Let results be a new empty List.
                let mut results: Vec<JsValue> = Vec::with_capacity(iter_count);

                // 4. For each integer i such that 0 <= i < iterCount, in ascending order, do
                for i in 0..iter_count {
                    // a. If iters[i] is empty, then
                    //     i. Append padding[i] to results.
                    if iters[i].is_none() {
                        results.push(padding.get(i).cloned().unwrap_or(JsValue::undefined()));
                        continue;
                    }

                    // b. Else,
                    //     i. Let iter be iters[i].
                    let iter = iters[i].as_mut().expect("present");
                    //     ii. Let step be ? Call(iter.[[NextMethod]], iter.[[Iterator]]).
                    let step_result = iter.step_value(context);

                    match step_result {
                        Err(err) => {
                            open_iters.retain(|&idx| idx != i);
                            iters[i] = None;
                            let mut result = Err(err);
                            for &idx in &open_iters {
                                if let Some(it) = &mut iters[idx] {
                                    let close_rc = it.close(Ok(JsValue::undefined()), context);
                                    if result.is_ok() && close_rc.is_err() {
                                        result = close_rc;
                                    }
                                }
                            }
                            return CoroutineState::Break(result.map(|_| ()));
                        }
                        Ok(None) => {
                            // v. If IteratorComplete(step) is true, then
                            //     1. Set iters[i] to empty.
                            open_iters.retain(|&idx| idx != i);
                            iters[i] = None;

                            match mode {
                                // 2. If mode is "shortest", then
                                //     a. Return ? IteratorCloseAll(iters, NormalCompletion(undefined)).
                                ZipMode::Shortest => {
                                    let mut result = Ok(());
                                    for &idx in &open_iters {
                                        if let Some(it) = &mut iters[idx] {
                                            let close_rc =
                                                it.close(Ok(JsValue::undefined()), context);
                                            if result.is_ok() && close_rc.is_err() {
                                                result = close_rc.map(|_| ());
                                            }
                                        }
                                    }
                                    return CoroutineState::Break(result);
                                }
                                // 3. If mode is "strict", then
                                ZipMode::Strict => {
                                    // a. If i > 0, then
                                    //     i. Return ? IteratorCloseAll(iters, ThrowCompletion(TypeError)).
                                    if i != 0 {
                                        let mut result = Ok(());
                                        for &idx in &open_iters {
                                            if let Some(it) = &mut iters[idx] {
                                                let close_rc =
                                                    it.close(Ok(JsValue::undefined()), context);
                                                if result.is_ok() && close_rc.is_err() {
                                                    result = close_rc.map(|_| ());
                                                }
                                            }
                                        }
                                        if result.is_ok() {
                                            return CoroutineState::Break(Err(JsNativeError::typ(
                                            )
                                            .with_message(
                                                "iterators have different lengths in strict mode",
                                            )
                                            .into()));
                                        }
                                        return CoroutineState::Break(result);
                                    }

                                    for k in 1..iter_count {
                                        if iters[k].is_none() {
                                            continue;
                                        }
                                        let other = iters[k].as_mut().expect("present");
                                        let step = other.step(context);
                                        match step {
                                            Err(err) => {
                                                open_iters.retain(|&idx| idx != k);
                                                iters[k] = None;
                                                let mut result = Err(err);
                                                for &idx in &open_iters {
                                                    if let Some(it) = &mut iters[idx] {
                                                        let close_rc = it.close(
                                                            Ok(JsValue::undefined()),
                                                            context,
                                                        );
                                                        if result.is_ok() && close_rc.is_err() {
                                                            result = close_rc;
                                                        }
                                                    }
                                                }
                                                return CoroutineState::Break(result.map(|_| ()));
                                            }
                                            Ok(is_done) => {
                                                if is_done {
                                                    open_iters.retain(|&idx| idx != k);
                                                    iters[k] = None;
                                                } else {
                                                    let mut result = Ok(());
                                                    for &idx in &open_iters {
                                                        if let Some(it) = &mut iters[idx] {
                                                            let close_rc = it.close(
                                                                Ok(JsValue::undefined()),
                                                                context,
                                                            );
                                                            if result.is_ok() && close_rc.is_err() {
                                                                result = close_rc.map(|_| ());
                                                            }
                                                        }
                                                    }
                                                    if result.is_ok() {
                                                        return CoroutineState::Break(Err(JsNativeError::typ().with_message("iterators have different lengths in strict mode").into()));
                                                    }
                                                    return CoroutineState::Break(result);
                                                }
                                            }
                                        }
                                    }
                                    // d. Return NormalCompletion(undefined).
                                    return CoroutineState::Break(Ok(()));
                                }
                                // 4. If mode is "longest", then ...
                                ZipMode::Longest => {
                                    if open_iters.is_empty() {
                                        return CoroutineState::Break(Ok(()));
                                    }
                                    results.push(
                                        padding.get(i).cloned().unwrap_or(JsValue::undefined()),
                                    );
                                }
                            }
                        }
                        Ok(Some(value)) => {
                            results.push(value);
                        }
                    }
                }

                let finished = match &result_kind {
                    ZipResultKind::Array => Array::create_array_from_list(results, context).into(),
                    ZipResultKind::Keyed(keys) => {
                        let obj = JsObject::with_null_proto();
                        for (i, key) in keys.iter().enumerate() {
                            if let Some(val) = results.get(i) {
                                let prop_key: PropertyKey =
                                    key.to_string(context).unwrap_or_default().into();
                                obj.set(prop_key, val.clone(), false, context)
                                    .expect("new object");
                            }
                        }
                        obj.into()
                    }
                };

                state.set(Self::Yielding {
                    iters,
                    iter_count,
                    open_iters,
                    mode,
                    padding,
                    result_kind,
                });
                CoroutineState::Continue(finished)
            },
            Cell::new(Self::Yielding {
                iters,
                iter_count,
                open_iters,
                mode,
                padding,
                result_kind,
            }),
        )
    }
}
