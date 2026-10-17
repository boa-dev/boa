use std::cell::Cell;

use boa_gc::{Finalize, Trace};

use crate::{
    builtins::{array::Array, iterable::IteratorRecord},
    native_function::{CoroutineBranch, CoroutineState, NativeCoroutine},
};

/// [`Iterator.prototype.chunks(chunkSize)`][spec].
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
        NativeCoroutine::from_copy_closure_with_captures(
            |completion, state, context| {
                let (mut iterated, size) = match state.take() {
                    Self::Completed => return CoroutineState::Break(Ok(())),
                    Self::Yielding { iterated, size } => {
                        iterated.if_abrupt_close_iterator(completion, context)?;
                        (iterated, size)
                    }
                };

                let mut buffer = Vec::new();
                while buffer.len() < size as usize {
                    match iterated.step_value(context).branch()? {
                        Some(value) => buffer.push(value),
                        None => {
                            return if buffer.is_empty() {
                                CoroutineState::Break(Ok(()))
                            } else {
                                CoroutineState::Continue(
                                    Array::create_array_from_list(buffer, context).into(),
                                )
                            };
                        }
                    }
                }

                state.set(Self::Yielding { iterated, size });
                CoroutineState::Continue(Array::create_array_from_list(buffer, context).into())
            },
            Cell::new(Self::Yielding { iterated, size }),
        )
    }
}
