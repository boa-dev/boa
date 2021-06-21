use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Uint32Array;

impl TypedArrayInstance for Uint32Array {
    const BYTES_PER_ELEMENT: usize = 2;
    const NAME: &'static str = "Uint32Array";
}
