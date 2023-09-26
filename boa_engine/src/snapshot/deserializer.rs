use std::hash::Hash;

use indexmap::IndexSet;
use rustc_hash::FxHashMap;
use thin_vec::ThinVec;

use super::{SnapshotError, SnapshotResult};

/// TODO: doc
pub trait Deserialize: Sized {
    /// TODO: doc
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self>;
}

/// TODO: doc
pub struct SnapshotDeserializer<'snapshot> {
    pub(super) bytes: &'snapshot [u8],
    pub(super) index: usize,
    pub(super) external_references: &'snapshot IndexSet<usize>,
}

impl SnapshotDeserializer<'_> {
    /// TODO: doc
    pub fn read_bool(&mut self) -> Result<bool, SnapshotError> {
        let byte = self.read_u8()?;
        assert!(byte == 0 || byte == 1);
        Ok(byte == 1)
    }
    /// TODO: doc
    pub fn read_u8(&mut self) -> Result<u8, SnapshotError> {
        let byte = self.bytes[self.index];
        self.index += 1;
        Ok(byte)
    }
    /// TODO: doc
    pub fn read_i8(&mut self) -> Result<i8, SnapshotError> {
        let byte = self.bytes[self.index];
        self.index += 1;
        Ok(byte as i8)
    }

    /// TODO: doc
    pub fn read_u16(&mut self) -> Result<u16, SnapshotError> {
        let bytes = self.read_bytes(std::mem::size_of::<u16>())?;
        let value = u16::from_le_bytes([bytes[0], bytes[1]]);
        Ok(value)
    }
    /// TODO: doc
    pub fn read_i16(&mut self) -> Result<i16, SnapshotError> {
        let value = self.read_u16()?;
        Ok(value as i16)
    }

    /// TODO: doc
    pub fn read_u32(&mut self) -> Result<u32, SnapshotError> {
        let bytes = self.read_bytes(4)?;
        let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(value)
    }
    /// TODO: doc
    pub fn read_i32(&mut self) -> Result<i32, SnapshotError> {
        let value = self.read_u32()?;
        Ok(value as i32)
    }

    /// TODO: doc
    pub fn read_f32(&mut self) -> Result<f32, SnapshotError> {
        let value = self.read_u32()?;
        Ok(f32::from_bits(value))
    }
    /// TODO: doc
    pub fn read_f64(&mut self) -> Result<f64, SnapshotError> {
        let value = self.read_u64()?;
        Ok(f64::from_bits(value))
    }

    /// TODO: doc
    pub fn read_u64(&mut self) -> Result<u64, SnapshotError> {
        let bytes = self.read_bytes(std::mem::size_of::<u64>())?;
        let value = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        Ok(value)
    }
    /// TODO: doc
    pub fn read_i64(&mut self) -> Result<i64, SnapshotError> {
        let value = self.read_u64()?;
        Ok(value as i64)
    }

    /// TODO: doc
    pub fn read_usize(&mut self) -> Result<usize, SnapshotError> {
        let value = self.read_u64()?;
        // TODO: handle error.
        Ok(usize::try_from(value).unwrap())
    }
    /// TODO: doc
    pub fn read_isize(&mut self) -> Result<isize, SnapshotError> {
        let value = self.read_usize()?;
        Ok(value as isize)
    }
    /// TODO: doc
    pub fn read_string(&mut self) -> Result<&str, SnapshotError> {
        let len = self.read_usize()?;
        let bytes = self.read_bytes(len)?;
        // TODO: handle error
        Ok(std::str::from_utf8(bytes).unwrap())
    }
    /// TODO: doc
    pub fn read_bytes(&mut self, count: usize) -> Result<&[u8], SnapshotError> {
        let index = self.index;
        self.index += count;
        // TODO: use .get() so we can handle the error.
        let bytes = &self.bytes[index..(index + count)];
        Ok(bytes)
    }
}

impl Deserialize for bool {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_bool()
    }
}

impl Deserialize for u8 {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_u8()
    }
}

impl Deserialize for i8 {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_i8()
    }
}

impl Deserialize for u16 {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_u16()
    }
}

impl Deserialize for i16 {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_i16()
    }
}

impl Deserialize for u32 {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_u32()
    }
}

impl Deserialize for i32 {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_i32()
    }
}

impl Deserialize for u64 {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_u64()
    }
}

impl Deserialize for i64 {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_i64()
    }
}

impl Deserialize for usize {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_usize()
    }
}

impl Deserialize for isize {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_isize()
    }
}

impl Deserialize for f32 {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_f32()
    }
}

impl Deserialize for f64 {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        d.read_f64()
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let is_some = bool::deserialize(d)?;
        if is_some {
            return Ok(Some(T::deserialize(d)?));
        }

        Ok(None)
    }
}

impl<T: Deserialize, E: Deserialize> Deserialize for Result<T, E> {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let is_ok = bool::deserialize(d)?;
        Ok(if is_ok {
            Ok(T::deserialize(d)?)
        } else {
            Err(E::deserialize(d)?)
        })
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let len = usize::deserialize(d)?;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            let value = T::deserialize(d)?;
            values.push(value);
        }
        Ok(values)
    }
}

impl<T: Deserialize> Deserialize for ThinVec<T> {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let len = usize::deserialize(d)?;
        let mut values = ThinVec::with_capacity(len);
        for _ in 0..len {
            let value = T::deserialize(d)?;
            values.push(value);
        }
        Ok(values)
    }
}

impl<T: Deserialize> Deserialize for Box<T> {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let value = T::deserialize(d)?;
        Ok(Box::new(value))
    }
}

impl Deserialize for Box<str> {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let len = usize::deserialize(d)?;
        let bytes = d.read_bytes(len)?;
        Ok(String::from_utf8(bytes.into()).unwrap().into_boxed_str())
    }
}

impl Deserialize for () {
    fn deserialize(_d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        // Deserialize nothing, zero size type.
        Ok(())
    }
}

impl<T1: Deserialize> Deserialize for (T1,) {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let v1 = T1::deserialize(d)?;
        Ok((v1,))
    }
}

impl<T1: Deserialize, T2: Deserialize> Deserialize for (T1, T2) {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let v1 = T1::deserialize(d)?;
        let v2 = T2::deserialize(d)?;
        Ok((v1, v2))
    }
}

impl<T1: Deserialize, T2: Deserialize, T3: Deserialize> Deserialize for (T1, T2, T3) {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let v1 = T1::deserialize(d)?;
        let v2 = T2::deserialize(d)?;
        let v3 = T3::deserialize(d)?;
        Ok((v1, v2, v3))
    }
}

impl<T1: Deserialize, T2: Deserialize, T3: Deserialize, T4: Deserialize> Deserialize
    for (T1, T2, T3, T4)
{
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let v1 = T1::deserialize(d)?;
        let v2 = T2::deserialize(d)?;
        let v3 = T3::deserialize(d)?;
        let v4 = T4::deserialize(d)?;
        Ok((v1, v2, v3, v4))
    }
}

impl<K: Deserialize + PartialEq + Eq + Hash, V: Deserialize> Deserialize for FxHashMap<K, V> {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let len = usize::deserialize(d)?;

        let mut result = Self::default();
        for _ in 0..len {
            let key = K::deserialize(d)?;
            let value = V::deserialize(d)?;
            let ret = result.insert(key, value);

            assert!(ret.is_none());
        }
        Ok(result)
    }
}
