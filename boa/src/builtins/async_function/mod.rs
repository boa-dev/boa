use crate::gc::{empty_trace, Finalize, Trace};

#[derive(Clone, Copy, Finalize, Debug)]
pub struct AsyncFunction;

unsafe impl Trace for AsyncFunction {
    empty_trace!();
}
