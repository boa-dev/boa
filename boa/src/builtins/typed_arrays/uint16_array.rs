use crate::builtins::typed_arrays::storage_class::TypedArrayElement;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Uint16Array;

impl TypedArrayInstance for Uint16Array {
    const NAME: &'static str = "Uint16Array";
    const ELEMENT_KIND: TypedArrayElement = TypedArrayElement::U16;
}
