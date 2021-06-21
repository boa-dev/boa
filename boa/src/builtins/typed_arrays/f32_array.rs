use crate::builtins::typed_arrays::typed_array::{TypedArrayInstance, TypedArrayStorageClass};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Float32Array;

impl TypedArrayInstance for Float32Array {
    const BYTES_PER_ELEMENT: usize = 4;
    const NAME: &'static str = "Float32Array";

    fn get_storage_class(capacity: usize) -> TypedArrayStorageClass {
        TypedArrayStorageClass::F32(Vec::with_capacity(capacity))
    }
}
