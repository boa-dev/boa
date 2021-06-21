use crate::builtins::typed_arrays::typed_array::{TypedArrayInstance, TypedArrayStorageClass};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Int16Array;

impl TypedArrayInstance for Int16Array {
    const BYTES_PER_ELEMENT: usize = 2;
    const NAME: &'static str = "Int16Array";

    fn get_storage_class(capacity: usize) -> TypedArrayStorageClass {
        TypedArrayStorageClass::I16(Vec::with_capacity(capacity))
    }
}
