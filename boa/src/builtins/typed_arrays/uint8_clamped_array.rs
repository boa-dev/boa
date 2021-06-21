use crate::builtins::typed_arrays::storage_class::TypedArrayElement;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Uint8ClampedArray;

impl TypedArrayInstance for Uint8ClampedArray {
    const NAME: &'static str = "Uint8ClampedArray";
    const ELEMENT_KIND: TypedArrayElement = TypedArrayElement::U8C;
}
