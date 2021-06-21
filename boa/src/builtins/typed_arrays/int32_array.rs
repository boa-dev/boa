use crate::builtins::typed_arrays::storage_class::TypedArrayStorageClass;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Int32Array;

impl TypedArrayInstance for Int32Array {
    const BYTES_PER_ELEMENT: usize = 4;
    const NAME: &'static str = "Int32Array";

    fn get_storage_class(capacity: usize) -> TypedArrayStorageClass {
        TypedArrayStorageClass::I32(Vec::with_capacity(capacity))
    }
}
