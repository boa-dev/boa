//! TODO: doc

#![allow(missing_debug_implementations)]
#![allow(dead_code)]

use crate::{Context, JsBigInt, JsObject, JsString, JsSymbol};
use indexmap::{IndexMap, IndexSet};
use std::fmt::{Debug, Display};

/// TODO: doc
#[derive(Debug, Clone, Copy)]
pub struct Header {
    signature: [u8; 4],
    version: u32,
    // checksum: u64,
}

/// TODO: doc
pub trait Serialize {
    /// Serialize type
    fn serialize(&self, s: &mut SnapshotSerializer) -> Result<(), SnapshotError>;
}

impl Serialize for Header {
    fn serialize(&self, s: &mut SnapshotSerializer) -> Result<(), SnapshotError> {
        s.write_bytes(&self.signature)?;
        s.write_u32(self.version)?;
        Ok(())
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
    header: Header,
    bytes: Vec<u8>,
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

    /// TODO: doc
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Serialize the given [`Context`].
    pub fn serialize(&mut self, context: &mut Context<'_>) -> Result<(), SnapshotError> {
        // Remove any garbage objects before serialization.
        boa_gc::force_collect();

        // boa_gc::walk_gc_alloc_pointers(|address| {

        // });

        let header = Header {
            signature: *b".boa",
            version: 42,
        };

        header.serialize(self)?;
        context.serialize(self)?;

        for i in 0..self.objects.len() {
            let object = self
                .objects
                .get_index(i)
                .expect("There should be an object")
                .clone();
            object.inner().serialize(self)?;
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
                desc.serialize(self)?;
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
            string.serialize(self)?;

            self.write_bool(string.is_static())?;
            self.write_usize(string.len())?;
            for elem in string.as_slice() {
                self.write_u16(*elem)?;
            }
        }

        Ok(())
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
