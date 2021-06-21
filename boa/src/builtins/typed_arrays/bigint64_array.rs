use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct BigInt64Array;

impl TypedArrayInstance for BigInt64Array {
    const BYTES_PER_ELEMENT: usize = 8;
    const NAME: &'static str = "BigInt64Array";
}
