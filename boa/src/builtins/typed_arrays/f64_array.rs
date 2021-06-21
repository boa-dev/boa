use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Float64Array;

impl TypedArrayInstance for Float64Array {
    const BYTES_PER_ELEMENT: usize = 8;
    const NAME: &'static str = "Float64Array";
}
