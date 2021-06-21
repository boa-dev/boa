use crate::builtins::typed_arrays::storage_class::TypedArrayElement;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct BigUint64Array;

impl TypedArrayInstance for BigUint64Array {
    const NAME: &'static str = "BigUint64Array";
    const ELEMENT_KIND: TypedArrayElement = TypedArrayElement::BigUint;
}
