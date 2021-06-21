use crate::builtins::typed_arrays::typed_array::{TypedArrayInstance, TypedArrayStorageClass};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Int8Array;

impl TypedArrayInstance for Int8Array {
    const BYTES_PER_ELEMENT: usize = 1;
    const NAME: &'static str = "Int8Array";

    fn get_storage_class(capacity: usize) -> TypedArrayStorageClass {
        TypedArrayStorageClass::I8(Vec::with_capacity(capacity))
    }
}
