use crate::builtins::typed_arrays::typed_array::{TypedArrayInstance, TypedArrayStorageClass};

#[derive(Debug, Clone, Copy)]
pub(crate) struct BigInt64Array;

impl TypedArrayInstance for BigInt64Array {
    const BYTES_PER_ELEMENT: usize = 8;
    const NAME: &'static str = "BigInt64Array";

    fn get_storage_class(capacity: usize) -> TypedArrayStorageClass {
        TypedArrayStorageClass::BigInt64(Vec::with_capacity(capacity))
    }
}
