use super::{Deserialize, Serialize, SnapshotDeserializer, SnapshotResult, SnapshotSerializer};

/// TODO: doc
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub(crate) signature: [u8; 4],
    pub(crate) version: u32,
    // checksum: u64,
}

impl Serialize for Header {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_bytes(&self.signature)?;
        s.write_u32(self.version)?;
        Ok(())
    }
}

impl Deserialize for Header {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> SnapshotResult<Self> {
        let signature = d.read_bytes(4)?;
        let signature = [signature[0], signature[1], signature[2], signature[3]];

        let version = d.read_u32()?;

        Ok(Self { signature, version })
    }
}
