use crate::builtins::typed_arrays::typed_array::{TypedArrayInstance, TypedArrayStorageClass};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Uint16Array;

impl TypedArrayInstance for Uint16Array {
    const BYTES_PER_ELEMENT: usize = 2;
    const NAME: &'static str = "Uint16Array";

    fn get_storage_class(capacity: usize) -> TypedArrayStorageClass {
        TypedArrayStorageClass::U16(Vec::with_capacity(capacity))
    }
}
