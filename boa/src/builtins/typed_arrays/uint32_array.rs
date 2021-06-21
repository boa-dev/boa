use crate::builtins::typed_arrays::typed_array::{TypedArrayInstance, TypedArrayStorageClass};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Uint32Array;

impl TypedArrayInstance for Uint32Array {
    const BYTES_PER_ELEMENT: usize = 2;
    const NAME: &'static str = "Uint32Array";

    fn get_storage_class(capacity: usize) -> TypedArrayStorageClass {
        TypedArrayStorageClass::U32(Vec::with_capacity(capacity))
    }
}
