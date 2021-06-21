use crate::builtins::typed_arrays::storage_class::TypedArrayElement;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Int8Array;

impl TypedArrayInstance for Int8Array {
    const NAME: &'static str = "Int8Array";
    const ELEMENT_KIND: TypedArrayElement = TypedArrayElement::I8;
}
