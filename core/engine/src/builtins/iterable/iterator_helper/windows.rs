use std::{cell::Cell, collections::VecDeque};

use boa_gc::{Finalize, Trace};

use crate::{
    JsValue,
    builtins::{array::Array, iterable::IteratorRecord},
    native_function::{CoroutineBranch, CoroutineState, NativeCoroutine},
};

/// [`Iterator.prototype.windows(windowSize[, undersized])`][spec]
///
/// Yields overlapping arrays of `windowSize` elements from the underlying
/// iterator. When `allow_partial` is true, a trailing undersized window is
/// also yielded.
///
/// [spec]: https://tc39.es/proposal-iterator-chunking/#sec-iterator.prototype.windows
#[derive(Trace, Finalize, Default)]
#[boa_gc(unsafe_no_drop)]
pub(crate) enum Windows {
    #[default]
    Completed,
    Yielding {
        iterated: IteratorRecord,
        size: u32,
        allow_partial: bool,
        buffer: VecDeque<JsValue>,
    },
}

impl Windows {
    #[allow(
        clippy::new_ret_no_self,
        reason = "slightly cleaner to have this be a `new` method"
    )]
    pub(crate) fn new(iterated: IteratorRecord, size: u32, allow_partial: bool) -> NativeCoroutine {
        // 9. Let closure be a new Abstract Closure with no parameters that
        //    captures iterated, windowSize, and undersized and performs the
        //    following steps when called:
        NativeCoroutine::from_copy_closure_with_captures(
            |completion, state, context| {
                let (mut iterated, size, allow_partial, mut buffer) = match state.take() {
                    Self::Completed => return CoroutineState::Break(Ok(())),
                    Self::Yielding {
                        iterated,
                        size,
                        allow_partial,
                        buffer,
                    } => {
                        // 9.b.v.2. IfAbruptCloseIterator(completion, iterated).
                        iterated.if_abrupt_close_iterator(completion, context)?;
                        (iterated, size, allow_partial, buffer)
                    }
                };

                // 9.a. Let buffer be a new empty List. (kept in state across yields)
                // 9.b. Repeat,
                loop {
                    // 9.b.i. Let value be ? IteratorStepValue(iterated).
                    match iterated.step_value(context).branch()? {
                        Some(value) => {
                            // 9.b.iii. If the number of elements in buffer is ℝ(windowSize), then
                            //     1. Remove the first element from buffer.
                            if buffer.len() == size as usize {
                                buffer.pop_front();
                            }
                            // 9.b.iv. Append value to buffer.
                            buffer.push_back(value);
                            // 9.b.v. If the number of elements in buffer is ℝ(windowSize), then
                            if buffer.len() == size as usize {
                                // 9.b.v.1. Let completion be Completion(Yield(CreateArrayFromList(buffer))).
                                let array =
                                    Array::create_array_from_list(buffer.iter().cloned(), context);
                                state.set(Self::Yielding {
                                    iterated,
                                    size,
                                    allow_partial,
                                    buffer,
                                });
                                return CoroutineState::Continue(array.into());
                            }
                        }
                        // 9.b.ii. If value is done, then
                        None => {
                            return if allow_partial
                                && !buffer.is_empty()
                                && buffer.len() < size as usize
                            {
                                // 9.b.ii.1. If undersized is "allow-partial", buffer is not
                                //    empty, and the number of elements in buffer < ℝ(windowSize), then
                                //     a. Perform Completion(Yield(CreateArrayFromList(buffer))).
                                CoroutineState::Continue(
                                    Array::create_array_from_list(buffer, context).into(),
                                )
                            } else {
                                // 9.b.ii.2. Return ReturnCompletion(undefined).
                                CoroutineState::Break(Ok(()))
                            };
                        }
                    }
                }
            },
            Cell::new(Self::Yielding {
                iterated,
                size,
                allow_partial,
                buffer: VecDeque::new(),
            }),
        )
    }
}
