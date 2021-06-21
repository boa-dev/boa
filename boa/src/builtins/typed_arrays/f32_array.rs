use crate::builtins::typed_arrays::storage_class::TypedArrayElement;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Float32Array;

impl TypedArrayInstance for Float32Array {
    const NAME: &'static str = "Float32Array";
    const ELEMENT_KIND: TypedArrayElement = TypedArrayElement::F32;
}
