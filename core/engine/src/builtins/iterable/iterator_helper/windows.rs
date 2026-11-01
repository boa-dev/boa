use std::{cell::Cell, collections::VecDeque};

use boa_gc::{Finalize, Trace};

use crate::{
    JsValue,
    builtins::{array::Array, iterable::IteratorRecord},
    native_function::{CoroutineBranch, CoroutineState, NativeCoroutine},
};

/// [`Iterator.prototype.windows(windowSize, undersized)`][spec].
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
                        iterated.if_abrupt_close_iterator(completion, context)?;
                        (iterated, size, allow_partial, buffer)
                    }
                };

                loop {
                    match iterated.step_value(context).branch()? {
                        Some(value) => {
                            if buffer.len() == size as usize {
                                buffer.pop_front();
                            }
                            buffer.push_back(value);
                            if buffer.len() == size as usize {
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
                        None => {
                            return if allow_partial
                                && !buffer.is_empty()
                                && buffer.len() < size as usize
                            {
                                CoroutineState::Continue(
                                    Array::create_array_from_list(buffer, context).into(),
                                )
                            } else {
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
