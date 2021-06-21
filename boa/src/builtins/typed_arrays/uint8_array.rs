use crate::builtins::typed_arrays::storage_class::TypedArrayElement;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Uint8Array;

impl TypedArrayInstance for Uint8Array {
    const NAME: &'static str = "Uint8Array";
    const ELEMENT_KIND: TypedArrayElement = TypedArrayElement::U8;
}
