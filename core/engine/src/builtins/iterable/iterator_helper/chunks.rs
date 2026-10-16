use std::cell::Cell;

use boa_gc::{Finalize, Trace};

use crate::{
    builtins::{array::Array, iterable::IteratorRecord},
    native_function::{CoroutineBranch, CoroutineState, NativeCoroutine},
};

/// [`Iterator.prototype.chunks(chunkSize)`][spec]
///
/// Yields non-overlapping arrays of up to `chunkSize` elements from the
/// underlying iterator.
///
/// [spec]: https://tc39.es/proposal-iterator-chunking/#sec-iterator.prototype.chunks
#[derive(Trace, Finalize, Default)]
#[boa_gc(unsafe_no_drop)]
pub(crate) enum Chunks {
    #[default]
    Completed,
    Yielding {
        iterated: IteratorRecord,
        size: u32,
    },
}

impl Chunks {
    #[allow(
        clippy::new_ret_no_self,
        reason = "slightly cleaner to have this be a `new` method"
    )]
    pub(crate) fn new(iterated: IteratorRecord, size: u32) -> NativeCoroutine {
        // 7. Let closure be a new Abstract Closure with no parameters that
        //    captures iterated and chunkSize and performs the following steps
        //    when called:
        NativeCoroutine::from_copy_closure_with_captures(
            |completion, state, context| {
                let (mut iterated, size) = match state.take() {
                    Self::Completed => return CoroutineState::Break(Ok(())),
                    Self::Yielding { iterated, size } => {
                        // 7.b.iv.2. IfAbruptCloseIterator(completion, iterated).
                        iterated.if_abrupt_close_iterator(completion, context)?;
                        (iterated, size)
                    }
                };

                // 7.a. Let buffer be a new empty List.
                // 7.b. Repeat,
                let mut buffer = Vec::new();
                while buffer.len() < size as usize {
                    // 7.b.i. Let value be ? IteratorStepValue(iterated).
                    match iterated.step_value(context).branch()? {
                        Some(value) => {
                            // 7.b.iii. Append value to buffer.
                            buffer.push(value);
                        }
                        // 7.b.ii. If value is done, then
                        None => {
                            return if buffer.is_empty() {
                                // 7.b.ii.2. Return ReturnCompletion(undefined).
                                CoroutineState::Break(Ok(()))
                            } else {
                                // 7.b.ii.1. If buffer is not empty, then
                                //     a. Perform Completion(Yield(CreateArrayFromList(buffer))).
                                // Do not restore Yielding state: the next resume
                                // must complete without IfAbruptCloseIterator.
                                CoroutineState::Continue(
                                    Array::create_array_from_list(buffer, context).into(),
                                )
                            };
                        }
                    }
                }

                // 7.b.iv. If the number of elements in buffer is ℝ(chunkSize), then
                //     1. Let completion be Completion(Yield(CreateArrayFromList(buffer))).
                state.set(Self::Yielding { iterated, size });
                CoroutineState::Continue(Array::create_array_from_list(buffer, context).into())
            },
            Cell::new(Self::Yielding { iterated, size }),
        )
    }
}
