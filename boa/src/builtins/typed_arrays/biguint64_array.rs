use crate::builtins::typed_arrays::storage_class::TypedArrayStorageClass;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct BigUint64Array;

impl TypedArrayInstance for BigUint64Array {
    const BYTES_PER_ELEMENT: usize = 8;
    const NAME: &'static str = "BigUint64Array";

    fn get_storage_class(capacity: usize) -> TypedArrayStorageClass {
        TypedArrayStorageClass::BigInt64(Vec::with_capacity(capacity))
    }
}
