// Temp allow
#![allow(dead_code)]
#![allow(clippy::needless_pass_by_value)]

use std::fmt::Debug;

use boa_builtins::{EncodedStaticPropertyKey, StaticPropertyKey};
use boa_gc::{Finalize, Trace};

use crate::{
    object::shape::property_table::PropertyTableInner, property::PropertyKey, symbol::WellKnown,
    JsString, JsSymbol,
};

use super::{
    shared_shape::TransitionKey, slot::SlotAttributes, ChangeTransition, JsPrototype, Shape, Slot,
    UniqueShape,
};

pub(crate) type StaticShapeInner = &'static boa_builtins::StaticShape;

/// TODO: doc
fn from_static_property_key(key: StaticPropertyKey) -> PropertyKey {
    match key {
        boa_builtins::StaticPropertyKey::String(index) => {
            PropertyKey::String(unsafe { JsString::from_index(index) })
        }
        boa_builtins::StaticPropertyKey::Symbol(s) => {
            let symbol = match WellKnown::try_from(s).expect("should be an well known symbol") {
                WellKnown::AsyncIterator => JsSymbol::async_iterator(),
                WellKnown::HasInstance => JsSymbol::has_instance(),
                WellKnown::IsConcatSpreadable => JsSymbol::is_concat_spreadable(),
                WellKnown::Iterator => JsSymbol::iterator(),
                WellKnown::Match => JsSymbol::r#match(),
                WellKnown::MatchAll => JsSymbol::match_all(),
                WellKnown::Replace => JsSymbol::replace(),
                WellKnown::Search => JsSymbol::search(),
                WellKnown::Species => JsSymbol::species(),
                WellKnown::Split => JsSymbol::split(),
                WellKnown::ToPrimitive => JsSymbol::to_primitive(),
                WellKnown::ToStringTag => JsSymbol::to_string_tag(),
                WellKnown::Unscopables => JsSymbol::unscopables(),
            };

            PropertyKey::Symbol(symbol)
        }
    }
}

/// TODO: doc
fn to_static_property_key(key: &PropertyKey) -> Option<StaticPropertyKey> {
    match key {
        PropertyKey::String(s) => Some(StaticPropertyKey::String(s.as_static_string_index()?)),
        PropertyKey::Symbol(s) => Some(StaticPropertyKey::Symbol(s.hash().try_into().ok()?)),
        PropertyKey::Index(_) => None,
    }
}

/// Represents a [`Shape`] that is not shared with any other object.
///
/// This is useful for objects that are inherently unique like,
/// the builtin object.
///
/// Cloning this does a shallow clone.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct StaticShape {
    inner: StaticShapeInner,
}

impl StaticShape {
    /// Create a new [`UniqueShape`].
    pub(crate) const fn new(inner: StaticShapeInner) -> Self {
        Self { inner }
    }

    /// Inserts a new property into the [`UniqueShape`].
    pub(crate) fn insert_property_transition(&self, _key: TransitionKey) -> UniqueShape {
        todo!()
    }

    /// Remove a property from the [`UniqueShape`].
    ///
    /// This will cause the current shape to be invalidated, and a new [`UniqueShape`] will be returned.
    pub(crate) fn remove_property_transition(&self, _key: &PropertyKey) -> UniqueShape {
        todo!()
    }

    /// Does a property lookup on the [`UniqueShape`] returning the [`Slot`] where it's
    /// located or [`None`] otherwise.
    pub(crate) fn lookup(&self, key: &PropertyKey) -> Option<Slot> {
        let key = to_static_property_key(key)?;

        let (index, attributes) = self.inner.get(key)?;

        Some(Slot {
            index: u32::from(index),
            attributes: SlotAttributes::from_bits_retain(attributes.bits()),
        })
    }

    /// Change the attributes of a property from the [`UniqueShape`].
    ///
    /// This will cause the current shape to be invalidated, and a new [`UniqueShape`] will be returned.
    ///
    /// NOTE: This assumes that the property had already been inserted.
    pub(crate) fn change_attributes_transition(
        &self,
        _key: &TransitionKey,
    ) -> ChangeTransition<Shape> {
        todo!()
    }

    /// Change the prototype of the [`UniqueShape`].
    ///
    /// This will cause the current shape to be invalidated, and a new [`UniqueShape`] will be returned.
    pub(crate) fn change_prototype_transition(&self, _prototype: JsPrototype) -> UniqueShape {
        todo!()
    }

    /// Gets all keys first strings then symbols in creation order.
    pub(crate) fn keys(&self) -> Vec<PropertyKey> {
        self.inner
            .property_table
            .keys()
            .map(EncodedStaticPropertyKey::decode)
            .map(from_static_property_key)
            .collect()
    }

    /// TODO: doc
    pub(crate) fn to_unique(&self, prototype: JsPrototype) -> UniqueShape {
        // TODO: optimization, preallocate capacity.
        // TODO: optimization, direct iniitialize the map.
        let mut property_table = PropertyTableInner::default();
        for (key, (_, slot_attributes)) in &self.inner.property_table {
            property_table.insert(
                from_static_property_key(key.decode()),
                SlotAttributes::from_bits_retain(slot_attributes.bits()),
            );
        }
        UniqueShape::new(prototype, property_table)
    }

    /// Return location in memory of the [`UniqueShape`].
    pub(crate) fn to_addr_usize(&self) -> usize {
        let ptr: *const _ = self.inner;
        ptr as usize
    }
}
