use gc::{Finalize, Trace};

use crate::Value;

#[derive(Debug, Clone, PartialOrd, PartialEq, Trace, Finalize)]
pub(crate) struct TypedArrayStorage {
    buffer: Vec<u8>,
    element_kind: TypedArrayElement,
}

impl TypedArrayStorage {
    fn create_buffer(capacity: usize) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(capacity);
        // SAFETY: Set len must always be the same as the capacity of the vector
        for _ in 0..capacity {
            buffer.push(0)
        }
        buffer
    }

    pub(crate) fn with_capacity_and_kind(capacity: usize, element_kind: TypedArrayElement) -> Self {
        Self {
            buffer: Self::create_buffer(capacity * element_kind.element_size()),
            element_kind,
        }
    }

    pub(crate) fn get_value_at_offset(&self, offset: u32) -> Option<Value> {
        let start = self.element_kind.offset(offset as usize);
        let end = start + self.element_kind.element_size();
        if end > self.buffer.capacity() {
            None
        } else {
            let ref slice = self.buffer[start..end];
            Some(self.element_kind.to_value_from_bytes(slice))
        }
    }
    pub(crate) fn capacity(&self) -> usize {
        self.buffer.capacity() / self.element_kind.element_size()
    }

    pub(crate) fn insert_value_at_offset(&mut self, offset: u32, value: Value) -> Option<Value> {
        let bytes = self.element_kind.as_bytes_from_value(value.clone());

        if let Some(bytes) = bytes {
            self.insert_bytes_at_offset(offset as usize, &bytes);
            Some(value)
        } else {
            None
        }
    }

    fn insert_bytes_at_offset(&mut self, offset: usize, value: &[u8]) {
        let start = self.element_kind.offset(offset);
        let end = start + self.element_kind.element_size();

        if end > self.buffer.capacity() {
            return;
        }

        let index = self.buffer.get_mut(start..end).unwrap();

        if index.len() != value.len() {
            panic!("Could not copy value in buffer. Sizes differ");
        }

        index.copy_from_slice(value);
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Finalize, Trace)]
pub(crate) enum TypedArrayElement {
    I8,
    U8,
    U8C,
    I16,
    U16,
    I32,
    U32,
    F32,
    F64,
    BigInt,
    BigUint,
}

impl TypedArrayElement {
    fn as_bytes_from_value(&self, value: Value) -> Option<Vec<u8>> {
        match self {
            Self::BigInt => {
                todo!()
            }
            Self::BigUint => {
                todo!()
            }
            kind => {
                let numeric = value.as_number();
                if let Some(number) = numeric {
                    // The behavior with unsigned ints is that we always put out bytes as if they were
                    // unsinged. Setting a negative number on an unsigned array results in a valid
                    // positive value (with the first bit set to 1)
                    let vec = match kind {
                        TypedArrayElement::I8 => {
                            Vec::from(((number % i8::MAX as f64) as i8).to_be_bytes())
                        }
                        TypedArrayElement::U8 => {
                            if number.is_sign_negative() {
                                Vec::from(((number % u8::MAX as f64) as i8).to_be_bytes())
                            } else {
                                Vec::from(((number % u8::MAX as f64) as u8).to_be_bytes())
                            }
                        }
                        TypedArrayElement::U8C => {
                            if number.is_sign_negative() {
                                Vec::from((number.clamp(0.0, u16::MAX as f64) as i8).to_be_bytes())
                            } else {
                                Vec::from((number.clamp(0.0, u16::MAX as f64) as u16).to_be_bytes())
                            }
                        }
                        TypedArrayElement::I16 => {
                            Vec::from(((number % i16::MAX as f64) as i16).to_be_bytes())
                        }
                        TypedArrayElement::U16 => {
                            if number.is_sign_negative() {
                                Vec::from(((number % u16::MAX as f64) as i16).to_be_bytes())
                            } else {
                                Vec::from(((number % u16::MAX as f64) as u16).to_be_bytes())
                            }
                        }
                        TypedArrayElement::I32 => {
                            Vec::from(((number % i32::MAX as f64) as i32).to_be_bytes())
                        }
                        TypedArrayElement::U32 => {
                            if number.is_sign_negative() {
                                Vec::from(((number % u32::MAX as f64) as i32).to_be_bytes())
                            } else {
                                Vec::from(((number % u32::MAX as f64) as u32).to_be_bytes())
                            }
                        }
                        TypedArrayElement::F32 => {
                            Vec::from(((number % f32::MAX as f64) as f32).to_be_bytes())
                        }
                        TypedArrayElement::F64 => Vec::from(number.to_be_bytes()),
                        _ => vec![],
                    };
                    Some(vec)
                } else {
                    None
                }
            }
        }
    }

    fn to_value_from_bytes(&self, bytes: &[u8]) -> Value {
        match self {
            Self::U8 => Value::from(u8::from_be_bytes([bytes[0]]) as u32),
            Self::U16 => Value::from(u16::from_be_bytes([bytes[0], bytes[1]]) as u32),
            _ => todo!(),
        }
    }

    fn offset(&self, index: usize) -> usize {
        self.element_size() * index
    }

    pub(crate) fn element_size(&self) -> usize {
        match self {
            Self::U8 | Self::I8 | Self::U8C => 1,
            Self::U16 | Self::I16 => 2,
            Self::F32 | Self::U32 | Self::I32 => 4,
            Self::BigUint | Self::BigInt | Self::F64 => 8,
        }
    }
}
