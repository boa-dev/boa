use gc::{Finalize, Trace};

use crate::builtins::BigInt;
use crate::{Context, Value};

#[derive(Debug, Clone, PartialOrd, PartialEq, Trace, Finalize)]
pub(crate) enum TypedArrayStorageClass {
    I8(Vec<i8>),
    U8(Vec<u8>),
    I16(Vec<i16>),
    U16(Vec<u16>),
    I32(Vec<i32>),
    U32(Vec<u32>),
    F32(Vec<f32>),
    F64(Vec<f64>),
    BigInt64(Vec<BigInt>),
}

impl TypedArrayStorageClass {
    pub(crate) fn get_typed_array_content_type(&self) -> TypedArrayContentType {
        match self {
            Self::BigInt64(_) => TypedArrayContentType::BigInt,
            _ => TypedArrayContentType::Number,
        }
    }

    pub(crate) unsafe fn set_length(&mut self, len: usize) {
        use TypedArrayStorageClass::*;
        match self {
            I8(value) => {
                value.set_len(len);
                for i in 0..len {
                    value[i] = 0
                }
            }
            U8(value) => {
                value.set_len(len);
                for i in 0..len {
                    value[i] = 0
                }
            }
            I16(value) => {
                value.set_len(len);
                for i in 0..len {
                    value[i] = 0
                }
            }
            U16(value) => {
                value.set_len(len);
                for i in 0..len {
                    value[i] = 0
                }
            }
            I32(value) => {
                value.set_len(len);
                for i in 0..len {
                    value[i] = 0
                }
            }
            U32(value) => {
                value.set_len(len);
                for i in 0..len {
                    value[i] = 0
                }
            }
            F32(value) => {
                value.set_len(len);
                for i in 0..len {
                    value[i] = 0.0
                }
            }
            F64(value) => {
                value.set_len(len);
                for i in 0..len {
                    value[i] = 0.0
                }
            }
            BigInt64(value) => {
                value.set_len(len);
                for i in 0..len {
                    value[i] = BigInt::from(0);
                }
            }
        }
    }

    pub(crate) fn set_value_at_index(
        &mut self,
        index: u32,
        value: Value,
        context: &mut Context,
    ) -> crate::Result<Value> {
        use crate::value::Numeric;
        use TypedArrayStorageClass::*;
        let numeric = value.to_numeric(context)?;
        //  TODO: implement the type conversion methods
        let index = index as usize;
        // TODO: Handle out of bounds exceptions here
        // match (self, numeric) {
        //     (I8(value), Numeric::Number(number)) => value[index] = number as i8,
        //     (U8(value), Numeric::Number(number)) => value[index] = number as u8,
        //     (I16(value), Numeric::Number(number)) => value[index] = number as i16,
        //     (U16(value), Numeric::Number(number)) => value[index] = number as u16,
        //     (I32(value), Numeric::Number(number)) => value[index] = number as i32,
        //     (U32(value), Numeric::Number(number)) => value[index] = number as u32,
        //     (F32(value), Numeric::Number(number)) => value[index] = number as f32,
        //     (F64(value), Numeric::Number(number)) => value[index] = number,
        //     (BigInt64(value), Numeric::BigInt(big_int)) => {
        //         value[index] = big_int.as_inner().clone()
        //     }
        //     _ => return context.throw_type_error("Must set numeric value for typed array"),
        // };

        Ok(Value::undefined())
    }

    pub(crate) fn get_value_at_index(&self, index: u32) -> Option<Value> {
        use crate::value::Numeric;
        use TypedArrayStorageClass::*;
        let numeric = match self {
            I8(value) => value.get(index as usize).cloned().map(Numeric::from),
            U8(value) => value.get(index as usize).cloned().map(Numeric::from),
            I16(value) => value.get(index as usize).cloned().map(Numeric::from),
            U16(value) => value.get(index as usize).cloned().map(Numeric::from),
            I32(value) => value.get(index as usize).cloned().map(Numeric::from),
            U32(value) => value.get(index as usize).cloned().map(Numeric::from),
            F32(value) => value
                .get(index as usize)
                .cloned()
                .map(|v| Numeric::from(v as f64)),
            F64(value) => value.get(index as usize).cloned().map(Numeric::from),
            BigInt64(value) => value.get(index as usize).cloned().map(Numeric::from),
        };

        Some(numeric.map(Value::from).unwrap_or(Value::undefined()))
    }

    pub(crate) fn length(&self) -> usize {
        use TypedArrayStorageClass::*;
        let capacity = match self {
            I8(value) => value.capacity(),
            U8(value) => value.capacity(),
            I16(value) => value.capacity(),
            U16(value) => value.capacity(),
            I32(value) => value.capacity(),
            U32(value) => value.capacity(),
            F32(value) => value.capacity(),
            F64(value) => value.capacity(),
            BigInt64(value) => value.capacity(),
        };

        capacity
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Trace, Finalize)]
pub(crate) enum TypedArrayContentType {
    Number,
    BigInt,
}
