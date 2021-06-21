use crate::builtins::typed_arrays::storage_class::TypedArrayElement;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Uint32Array;

impl TypedArrayInstance for Uint32Array {
    const NAME: &'static str = "Uint32Array";
    const ELEMENT_KIND: TypedArrayElement = TypedArrayElement::U32;
}
