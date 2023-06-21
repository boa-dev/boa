//! TODO: doc

#![allow(missing_debug_implementations)]
#![allow(dead_code)]

use crate::{Context, JsBigInt, JsObject, JsString, JsSymbol};
use indexmap::{IndexMap, IndexSet};
use std::fmt::{Debug, Display};

/// TODO: doc
pub trait Serialize {
    /// Serialize type
    fn serialize(&self, s: &mut SnapshotSerializer) -> Result<(), SnapshotError>;
}

/// TODO: doc
pub trait Deserialize: Sized {
    /// TODO: doc
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> Result<Self, SnapshotError>;
}

/// TODO: doc
#[derive(Debug, Clone, Copy)]
pub struct Header {
    signature: [u8; 4],
    version: u32,
    // checksum: u64,
}

impl Serialize for Header {
    fn serialize(&self, s: &mut SnapshotSerializer) -> Result<(), SnapshotError> {
        s.write_bytes(&self.signature)?;
        s.write_u32(self.version)?;
        Ok(())
    }
}

impl Deserialize for Header {
    fn deserialize(d: &mut SnapshotDeserializer<'_>) -> Result<Self, SnapshotError> {
        let signature = d.read_bytes(4)?;
        let signature = [signature[0], signature[1], signature[2], signature[3]];

        let version = d.read_u32()?;

        Ok(Self { signature, version })
    }
}

impl Serialize for JsObject {
    fn serialize(&self, s: &mut SnapshotSerializer) -> Result<(), SnapshotError> {
        let value = s.objects.insert_full(self.clone()).0;

        s.write_u32(value as u32)?;
        Ok(())
    }
}

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

/// TODO: doc
pub struct SnapshotDeserializer<'snapshot> {
    bytes: &'snapshot [u8],
    index: usize,
    external_references: &'snapshot IndexSet<usize>,
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

/// TODO: doc
pub struct SnapshotSerializer {
    bytes: Vec<u8>,
    objects: IndexSet<JsObject>,
    strings: IndexMap<usize, JsString>,
    symbols: IndexMap<u64, JsSymbol>,
    bigints: IndexSet<JsBigInt>,
    external_references: IndexSet<usize>,
}

/// TODO: doc
#[derive(Debug)]
pub enum SnapshotError {
    /// Input/output error.
    ///
    /// See: [`std::io::Error`].
    Io(std::io::Error),
}

impl Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: implement better formatting
        <Self as Debug>::fmt(self, f)
    }
}

impl std::error::Error for SnapshotError {}

impl From<std::io::Error> for SnapshotError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl SnapshotSerializer {
    /// TODO: doc
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            objects: IndexSet::default(),
            strings: IndexMap::default(),
            symbols: IndexMap::default(),
            bigints: IndexSet::default(),
            external_references: IndexSet::default(),
        }
    }

    /// Serialize the given [`Context`].
    pub fn serialize(mut self, context: &mut Context<'_>) -> Result<Snapshot, SnapshotError> {
        // Remove any garbage objects before serialization.
        boa_gc::force_collect();

        // boa_gc::walk_gc_alloc_pointers(|address| {
        // });

        let header = Header {
            signature: *b".boa",
            version: 42,
        };

        header.serialize(&mut self)?;
        context.serialize(&mut self)?;

        for i in 0..self.objects.len() {
            let object = self
                .objects
                .get_index(i)
                .expect("There should be an object")
                .clone();
            object.inner().serialize(&mut self)?;
        }

        for i in 0..self.symbols.len() {
            let (hash, symbol) = self
                .symbols
                .get_index(i)
                .map(|(hash, symbol)| (*hash, symbol.clone()))
                .expect("There should be an object");

            self.write_u64(hash)?;
            if let Some(desc) = symbol.description() {
                self.write_bool(true)?;
                desc.serialize(&mut self)?;
            } else {
                self.write_bool(false)?;
            }
        }

        for i in 0..self.strings.len() {
            let string = self
                .strings
                .get_index(i)
                .expect("There should be an string")
                .1
                .clone();
            // string.
            string.serialize(&mut self)?;

            self.write_bool(string.is_static())?;
            self.write_usize(string.len())?;
            for elem in string.as_slice() {
                self.write_u16(*elem)?;
            }
        }

        Ok(Snapshot {
            bytes: self.bytes,
            external_references: self.external_references,
        })
    }

    /// TODO: doc
    pub fn write_bool(&mut self, v: bool) -> Result<(), SnapshotError> {
        Ok(self.write_u8(if v { 1 } else { 0 })?)
    }
    /// TODO: doc
    pub fn write_u8(&mut self, v: u8) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&[v])?)
    }
    /// TODO: doc
    pub fn write_i8(&mut self, v: i8) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }

    /// TODO: doc
    pub fn write_u16(&mut self, v: u16) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_i16(&mut self, v: i16) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }

    /// TODO: doc
    pub fn write_u32(&mut self, v: u32) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_i32(&mut self, v: i32) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }

    /// TODO: doc
    pub fn write_f32(&mut self, v: f32) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_f64(&mut self, v: f64) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }

    /// TODO: doc
    pub fn write_u64(&mut self, v: u64) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_i64(&mut self, v: i64) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_u128(&mut self, v: u128) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_i128(&mut self, v: i128) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_usize(&mut self, v: usize) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&(v as u64).to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_isize(&mut self, v: isize) -> Result<(), SnapshotError> {
        Ok(self.write_bytes(&(v as i64).to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_string(&mut self, v: &str) -> Result<(), SnapshotError> {
        let asb = v.as_bytes();
        self.write_usize(asb.len())?;
        self.bytes.extend_from_slice(asb);
        Ok(())
    }
    /// TODO: doc
    pub fn write_bytes(&mut self, v: &[u8]) -> Result<(), SnapshotError> {
        self.bytes.extend_from_slice(v);
        Ok(())
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> Result<(), SnapshotError> {
        s.write_usize(self.len())?;
        for element in self {
            element.serialize(s)?;
        }
        Ok(())
    }
}

impl Serialize for JsString {
    fn serialize(&self, s: &mut SnapshotSerializer) -> Result<(), SnapshotError> {
        let index = s.strings.insert_full(self.ptr.addr(), self.clone()).0;

        s.write_u32(index as u32)?;
        Ok(())
    }
}

impl Serialize for JsSymbol {
    fn serialize(&self, s: &mut SnapshotSerializer) -> Result<(), SnapshotError> {
        let index = s.symbols.insert_full(self.hash(), self.clone()).0;

        s.write_u32(index as u32)?;
        Ok(())
    }
}

impl Serialize for JsBigInt {
    fn serialize(&self, s: &mut SnapshotSerializer) -> Result<(), SnapshotError> {
        let index = s.bigints.insert_full(self.clone()).0;
        s.write_u32(index as u32)?;
        Ok(())
    }
}
