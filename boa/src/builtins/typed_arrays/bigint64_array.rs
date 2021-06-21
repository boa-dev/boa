use crate::builtins::typed_arrays::storage_class::TypedArrayElement;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct BigInt64Array;

impl TypedArrayInstance for BigInt64Array {
    const ELEMENT_KIND: TypedArrayElement = TypedArrayElement::BigInt;
    const NAME: &'static str = "BigInt64Array";
}
