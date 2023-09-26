use std::{
    cell::{Cell, RefCell},
    hash::Hash,
    mem::size_of,
    rc::Rc,
};

use boa_gc::{Gc, Trace};
use indexmap::{IndexMap, IndexSet};
use rustc_hash::FxHashMap;
use thin_vec::ThinVec;

use crate::{object::shape::SharedShape, Context, JsBigInt, JsObject, JsString, JsSymbol};

use super::{Header, Snapshot, SnapshotError, SnapshotResult};

/// TODO: doc
pub trait Serialize {
    /// Serialize type
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()>;
}

/// TODO: doc
struct Reference {
    is_inlined: u8,
    index: u32,
}

impl Reference {
    fn new(is_inlined: bool, index: u32) -> Self {
        Self {
            is_inlined: if is_inlined { b'I' } else { b'R' },
            index,
        }
    }
}

impl Serialize for Reference {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        self.is_inlined.serialize(s)?;
        self.index.serialize(s)?;
        Ok(())
    }
}

/// TODO: doc
pub struct SnapshotSerializer {
    bytes: Vec<u8>,
    objects: IndexSet<JsObject>,
    strings: IndexMap<usize, JsString>,
    symbols: IndexMap<u64, JsSymbol>,
    bigints: IndexSet<JsBigInt>,
    shared_shapes: IndexMap<usize, SharedShape>,

    pub(crate) internal_reference: IndexMap<usize, u32>,
    external_references: IndexSet<usize>,
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
            shared_shapes: IndexMap::default(),
            internal_reference: IndexMap::default(),
            external_references: IndexSet::default(),
        }
    }

    /// Serialize the given [`Context`].
    pub fn serialize(mut self, context: &mut Context<'_>) -> Result<Snapshot, SnapshotError> {
        // Remove any garbage objects before serialization.
        boa_gc::force_collect();
        boa_gc::force_collect();
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

        Ok(Snapshot {
            bytes: self.bytes,
            external_references: self.external_references,
        })
    }

    /// TODO: doc
    pub fn write_bool(&mut self, v: bool) -> SnapshotResult<()> {
        Ok(self.write_u8(if v { 1 } else { 0 })?)
    }
    /// TODO: doc
    pub fn write_u8(&mut self, v: u8) -> SnapshotResult<()> {
        Ok(self.write_bytes(&[v])?)
    }
    /// TODO: doc
    pub fn write_i8(&mut self, v: i8) -> SnapshotResult<()> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }

    /// TODO: doc
    pub fn write_u16(&mut self, v: u16) -> SnapshotResult<()> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_i16(&mut self, v: i16) -> SnapshotResult<()> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }

    /// TODO: doc
    pub fn write_u32(&mut self, v: u32) -> SnapshotResult<()> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_i32(&mut self, v: i32) -> SnapshotResult<()> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }

    /// TODO: doc
    pub fn write_f32(&mut self, v: f32) -> SnapshotResult<()> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_f64(&mut self, v: f64) -> SnapshotResult<()> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }

    /// TODO: doc
    pub fn write_u64(&mut self, v: u64) -> SnapshotResult<()> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_i64(&mut self, v: i64) -> SnapshotResult<()> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_u128(&mut self, v: u128) -> SnapshotResult<()> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_i128(&mut self, v: i128) -> SnapshotResult<()> {
        Ok(self.write_bytes(&v.to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_usize(&mut self, v: usize) -> SnapshotResult<()> {
        Ok(self.write_bytes(&(v as u64).to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_isize(&mut self, v: isize) -> SnapshotResult<()> {
        Ok(self.write_bytes(&(v as i64).to_le_bytes())?)
    }
    /// TODO: doc
    pub fn write_string(&mut self, v: &str) -> SnapshotResult<()> {
        let asb = v.as_bytes();
        self.write_usize(asb.len())?;
        self.bytes.extend_from_slice(asb);
        Ok(())
    }
    /// TODO: doc
    pub fn write_bytes(&mut self, v: &[u8]) -> SnapshotResult<()> {
        self.bytes.extend_from_slice(v);
        Ok(())
    }

    /// TODO: doc
    pub fn reference_or<F>(&mut self, ptr: usize, f: F) -> SnapshotResult<()>
    where
        F: FnOnce(&mut SnapshotSerializer) -> SnapshotResult<()>,
    {
        match self.internal_reference.entry(ptr) {
            indexmap::map::Entry::Occupied(entry) => {
                let index = *entry.get();
                Reference::new(false, index).serialize(self)?;
                return Ok(());
            }
            indexmap::map::Entry::Vacant(entry) => {
                let index =
                    *entry.insert((self.bytes.len() + size_of::<u8>() + size_of::<u32>()) as u32);
                Reference::new(true, index).serialize(self)?;
            }
        }

        f(self)
    }
}

impl Serialize for bool {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_bool(*self)
    }
}

impl Serialize for u8 {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_u8(*self)
    }
}

impl Serialize for i8 {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_i8(*self)
    }
}

impl Serialize for u16 {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_u16(*self)
    }
}

impl Serialize for i16 {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_i16(*self)
    }
}

impl Serialize for u32 {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_u32(*self)
    }
}

impl Serialize for i32 {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_i32(*self)
    }
}

impl Serialize for u64 {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_u64(*self)
    }
}

impl Serialize for i64 {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_i64(*self)
    }
}

impl Serialize for usize {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_usize(*self)
    }
}

impl Serialize for isize {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_isize(*self)
    }
}

impl Serialize for f32 {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_f32(*self)
    }
}

impl Serialize for f64 {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_f64(*self)
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        if let Some(value) = self {
            s.write_bool(true)?;
            value.serialize(s)?
        } else {
            s.write_bool(false)?;
        }
        Ok(())
    }
}

impl<T: Serialize, E: Serialize> Serialize for Result<T, E> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        match self {
            Ok(value) => {
                s.write_bool(true)?;
                value.serialize(s)?;
            }
            Err(err) => {
                s.write_bool(false)?;
                err.serialize(s)?;
            }
        }
        Ok(())
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_usize(self.len())?;
        for element in self {
            element.serialize(s)?;
        }
        Ok(())
    }
}

impl<T: Serialize> Serialize for ThinVec<T> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        s.write_usize(self.len())?;
        for element in self {
            element.serialize(s)?;
        }
        Ok(())
    }
}

impl<T: Serialize> Serialize for Box<T> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        T::serialize(&self, s)
    }
}

impl Serialize for Box<str> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        self.len().serialize(s)?;
        s.write_bytes(self.as_bytes())?;
        Ok(())
    }
}

impl Serialize for String {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        self.len().serialize(s)?;
        s.write_bytes(self.as_bytes())?;
        Ok(())
    }
}

impl Serialize for () {
    fn serialize(&self, _s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        // Serialize nothing
        Ok(())
    }
}

impl<T1: Serialize> Serialize for (T1,) {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        self.0.serialize(s)?;
        Ok(())
    }
}

impl<T1: Serialize, T2: Serialize> Serialize for (T1, T2) {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        self.0.serialize(s)?;
        self.1.serialize(s)?;
        Ok(())
    }
}

impl<T1: Serialize, T2: Serialize, T3: Serialize> Serialize for (T1, T2, T3) {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        self.0.serialize(s)?;
        self.1.serialize(s)?;
        self.2.serialize(s)?;
        Ok(())
    }
}

impl<T1: Serialize, T2: Serialize, T3: Serialize, T4: Serialize> Serialize for (T1, T2, T3, T4) {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        self.0.serialize(s)?;
        self.1.serialize(s)?;
        self.2.serialize(s)?;
        self.3.serialize(s)?;
        Ok(())
    }
}

impl<K: Serialize + PartialEq + Eq + Hash, V: Serialize> Serialize for FxHashMap<K, V> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        self.len().serialize(s)?;
        for (key, value) in self {
            key.serialize(s)?;
            value.serialize(s)?;
        }
        Ok(())
    }
}

impl Serialize for JsBigInt {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        let ptr: *const _ = self.as_inner();
        s.reference_or(ptr as usize, |s| {
            let (sign, bytes) = self.as_inner().to_bytes_le();

            match sign {
                num_bigint::Sign::Minus => b'-',
                num_bigint::Sign::NoSign => b' ',
                num_bigint::Sign::Plus => b'+',
            }
            .serialize(s)?;

            bytes.serialize(s)
        })
    }
}

impl Serialize for JsObject {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        let ptr: *const _ = self.inner();
        s.reference_or(ptr as usize, |s| self.inner().serialize(s))
    }
}

impl<T: Serialize + Trace> Serialize for Gc<T> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        let ptr: *const _ = &*self;
        s.reference_or(ptr as usize, |s| T::serialize(&*self, s))
    }
}

impl<T: Serialize> Serialize for Rc<T> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        let ptr: *const _ = &*self;
        s.reference_or(ptr as usize, |s| T::serialize(&*self, s))
    }
}

impl<T: Serialize> Serialize for RefCell<T> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        self.borrow().serialize(s)
    }
}

impl<T: Serialize + Copy> Serialize for Cell<T> {
    fn serialize(&self, s: &mut SnapshotSerializer) -> SnapshotResult<()> {
        self.get().serialize(s)
    }
}
