use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Int32Array;

impl TypedArrayInstance for Int32Array {
    const BYTES_PER_ELEMENT: usize = 4;
    const NAME: &'static str = "Int32Array";
}
