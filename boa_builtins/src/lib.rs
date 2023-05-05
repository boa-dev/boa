use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
};

use bitflags::bitflags;
use phf::PhfHash;
use phf_shared::PhfBorrow;

bitflags! {
    /// This struct constains the property flags as described in the ECMAScript specification.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Attribute: u8 {
        /// The `Writable` attribute decides whether the value associated with the property can be changed or not, from its initial value.
        const WRITABLE = 0b0000_0001;

        /// If the property can be enumerated by a `for-in` loop.
        const ENUMERABLE = 0b0000_0010;

        /// If the property descriptor can be changed later.
        const CONFIGURABLE = 0b0000_0100;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StaticString {
    index: u16,
}

impl StaticString {
    #[inline]
    pub fn index(self) -> u16 {
        self.index
    }
}

#[derive(Clone, Copy)]
pub struct EncodedStaticPropertyKey(u16);

impl EncodedStaticPropertyKey {
    #[inline]
    pub fn decode(&self) -> StaticPropertyKey {
        let value = self.0 >> 1;
        if self.0 & 1 == 0 {
            StaticPropertyKey::String(value)
        } else {
            StaticPropertyKey::Symbol(value as u8)
        }
    }
}

const fn string(index: u16) -> EncodedStaticPropertyKey {
    debug_assert!(index < 2u16.pow(15));

    EncodedStaticPropertyKey(index << 1)
}

const fn symbol(index: u8) -> EncodedStaticPropertyKey {
    EncodedStaticPropertyKey(((index as u16) << 1) | 1)
}

impl Debug for EncodedStaticPropertyKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.decode().fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StaticPropertyKey {
    String(u16),
    Symbol(u8),
}

impl StaticPropertyKey {
    #[inline]
    pub fn encode(self) -> EncodedStaticPropertyKey {
        match self {
            StaticPropertyKey::String(x) => string(x),
            StaticPropertyKey::Symbol(x) => symbol(x),
        }
    }
}

impl Debug for StaticPropertyKey {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            StaticPropertyKey::String(index) => {
                let string = RAW_STATICS[index as usize];
                let string = String::from_utf16_lossy(string);
                write!(f, "String(\"{string}\")")
            }
            StaticPropertyKey::Symbol(symbol) => {
                write!(f, "Symbol({symbol})")
            }
        }
    }
}

impl Eq for EncodedStaticPropertyKey {}

impl PartialEq for EncodedStaticPropertyKey {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for EncodedStaticPropertyKey {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PhfHash for EncodedStaticPropertyKey {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

impl PhfBorrow<EncodedStaticPropertyKey> for EncodedStaticPropertyKey {
    #[inline]
    fn borrow(&self) -> &EncodedStaticPropertyKey {
        self
    }
}

pub type Slot = (u8, Attribute);

#[derive(Debug)]
pub struct StaticShape {
    pub property_table: phf::OrderedMap<EncodedStaticPropertyKey, Slot>,

    pub storage_len: usize,

    /// \[\[Prototype\]\]
    pub prototype: Option<&'static StaticShape>,
}

impl StaticShape {
    #[inline]
    pub fn get(&self, key: StaticPropertyKey) -> Option<Slot> {
        // SAFETY: only used to extend the lifetime, so we are able to call get.
        self.property_table
            .get(&key.encode())
            .map(|(index, attributes)| (*index, *attributes))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.property_table.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.property_table.len()
    }

    #[inline]
    pub fn get_string_key_expect(&self, index: usize) -> StaticString {
        match self
            .property_table
            .index(index)
            .expect("there should be a key at the given index")
            .0
            .decode()
        {
            StaticPropertyKey::String(index) => StaticString { index },
            StaticPropertyKey::Symbol(s) => {
                panic!("The key should be a string at position {index}, but symbol {s}")
            }
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/static_shapes_codegen.rs"));

// static NUMBER_BUITIN_OBJECT_STATIC_SHAPE_REF: &StaticShape = &NUMBER_BUITIN_OBJECT_STATIC_SHAPE;
