use crate::builtins::typed_arrays::storage_class::TypedArrayElement;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Int32Array;

impl TypedArrayInstance for Int32Array {
    const NAME: &'static str = "Int32Array";
    const ELEMENT_KIND: TypedArrayElement = TypedArrayElement::I32;
}
