//! TODO: doc

#![allow(missing_debug_implementations)]
#![allow(dead_code)]

mod deserializer;
mod error;
mod header;
mod serializer;

pub use deserializer::*;
pub use error::*;
pub use header::*;
pub use serializer::*;

use crate::Context;
use indexmap::IndexSet;

/// TODO: doc
pub struct Snapshot {
    bytes: Vec<u8>,
    external_references: IndexSet<usize>,
}

impl Snapshot {
    /// TODO: doc
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            external_references: IndexSet::default(),
        }
    }

    /// TODO: doc
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// TODO: doc
    pub fn deserialize<'a>(&self) -> Result<Context<'a>, SnapshotError> {
        let mut deserializer = SnapshotDeserializer {
            index: 0,
            bytes: &self.bytes,
            external_references: &self.external_references,
        };

        let header = Header::deserialize(&mut deserializer)?;

        // TODO: Do error handling and snapshot integrity checks.
        assert_eq!(&header.signature, b".boa");
        assert_eq!(header.version, 42);

        let context = Context::deserialize(&mut deserializer)?;

        // Assert that all bytes are consumed.
        // assert_eq!(deserializer.index, deserializer.bytes.len());

        Ok(context)
    }
}
