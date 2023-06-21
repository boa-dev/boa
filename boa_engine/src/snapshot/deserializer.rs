use indexmap::IndexSet;

use super::SnapshotError;

/// TODO: doc
pub trait Deserialize: Sized {
    /// TODO: doc
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> Result<Self, SnapshotError>;
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
