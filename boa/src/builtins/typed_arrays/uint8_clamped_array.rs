use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Uint8ClampedArray;

impl TypedArrayInstance for Uint8ClampedArray {
    const BYTES_PER_ELEMENT: usize = 1;
    const NAME: &'static str = "Uint8ClampedArray";
}
