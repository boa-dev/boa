use crate::builtins::typed_arrays::storage_class::TypedArrayElement;
use crate::builtins::typed_arrays::typed_array::TypedArrayInstance;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Float64Array;

impl TypedArrayInstance for Float64Array {
    const NAME: &'static str = "Float64Array";
    const ELEMENT_KIND: TypedArrayElement = TypedArrayElement::F64;
}
