use crate::builtins::typed_arrays::storage_class::TypedArrayElement;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Int16Array;

impl TypedArrayInstance for Int16Array {
    const NAME: &'static str = "Int16Array";
    const ELEMENT_KIND: TypedArrayElement = TypedArrayElement::I16;
}
