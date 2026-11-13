use super::{
    JsPrototype, ObjectStorage, PropertyKey,
    shape::{
        ChangeTransitionAction, RootShape, Shape, UniqueShape,
        property_table::PropertyTableInner,
        shared_shape::TransitionKey,
        slot::{Slot, SlotAttributes},
    },
};
use crate::{JsValue, object::JsFunction};
use crate::{property::CompletePropertyDescriptor, value::JsVariant};
use boa_gc::{Finalize, Trace};
use rustc_hash::FxHashMap;
use std::{collections::hash_map, iter::FusedIterator};
use thin_vec::ThinVec;

/// This represents all the indexed properties.
///
/// The index properties can be stored in two storage methods:
///
/// ## Dense Storage
///
/// Dense storage holds a contiguous array of properties where the index in the array is the key of the property.
/// These are known to be data descriptors with a value field, writable field set to `true`, configurable field set to `true`, enumerable field set to `true`.
///
/// Since we know the properties of the property descriptors (and they are all the same) we can omit it and just store only
/// the value field and construct the data property descriptor on demand.
///
/// ## Sparse Storage
///
/// This storage is used as a backup if the element keys are not continuous or the property descriptors
/// are not data descriptors with a value field, writable field set to `true`, configurable field set to `true`, enumerable field set to `true`.
///
/// This method uses more space, since we also have to store the property descriptors, not just the value.
/// It is also slower because we need to do a hash lookup.
#[derive(Debug, Trace, Finalize)]
pub enum IndexedProperties {
    /// Dense [`i32`] storage.
    DenseI32 {
        /// The contiguous array of i32 values representing the dense storage.
        vec: ThinVec<i32>,
        /// The length of the dense storage.
        length: u32,
        /// Whether the length of the dense storage is writable.
        length_writable: bool,
    },

    /// Dense [`f64`] storage.
    DenseF64 {
        /// The contiguous array of f64 values representing the dense storage.
        vec: ThinVec<f64>,
        /// The length of the dense storage.
        length: u32,
        /// Whether the length of the dense storage is writable.
        length_writable: bool,
    },

    /// Dense [`JsValue`] storage.
    DenseElement {
        /// The contiguous array of `JsValue` representing the dense storage.
        vec: ThinVec<JsValue>,
        /// The length of the dense storage.
        length: u32,
        /// Whether the length of the dense storage is writable.
        length_writable: bool,
    },

    /// Sparse storage that only keeps element values.
    SparseElement {
        /// The hash map storing the sparse element values.
        map: Box<FxHashMap<u32, JsValue>>,
        /// The length of the sparse storage.
        length: u32,
        /// Whether the length of the sparse storage is writable.
        length_writable: bool,
    },

    /// Sparse storage that keeps full property descriptors.
    SparseProperty {
        /// The hash map storing the sparse property descriptors.
        map: Box<FxHashMap<u32, CompletePropertyDescriptor>>,
        /// The length of the sparse storage.
        length: u32,
        /// Whether the length of the sparse storage is writable.
        length_writable: bool,
    },
}

impl Default for IndexedProperties {
    #[inline]
    fn default() -> Self {
        Self::DenseI32 {
            vec: ThinVec::new(),
            length: 0,
            length_writable: true,
        }
    }
}

impl IndexedProperties {
    pub(crate) fn new(length: u32) -> Self {
        Self::DenseI32 {
            length,
            vec: ThinVec::new(),
            length_writable: true,
        }
    }

    pub(crate) fn from_dense_js_value(elements: ThinVec<JsValue>) -> Self {
        if elements.is_empty() {
            return Self::default();
        }
        Self::DenseElement {
            vec: elements,
            length: 0,
            length_writable: true,
        }
    }

    #[must_use]
    pub(crate) fn array_length(&self) -> u32 {
        match self {
            IndexedProperties::DenseI32 { length, .. }
            | IndexedProperties::DenseF64 { length, .. }
            | IndexedProperties::DenseElement { length, .. }
            | IndexedProperties::SparseElement { length, .. }
            | IndexedProperties::SparseProperty { length, .. } => *length,
        }
    }

    pub(crate) fn set_array_length(&mut self, new_length: u32) -> bool {
        match self {
            IndexedProperties::DenseI32 { vec, length, .. } => {
                *length = new_length;
                vec.truncate(*length as usize);
            }
            IndexedProperties::DenseF64 { vec, length, .. } => {
                *length = new_length;
                vec.truncate(*length as usize);
            }
            IndexedProperties::DenseElement { vec, length, .. } => {
                *length = new_length;
                vec.truncate(*length as usize);
            }
            IndexedProperties::SparseElement { map, length, .. } => {
                if *length <= new_length {
                    *length = new_length;
                    return true;
                }

                map.retain(|&key, _| key < new_length);

                *length = new_length;
                return true;
            }
            IndexedProperties::SparseProperty { map, length, .. } => {
                if *length == new_length {
                    return true;
                }

                if *length < new_length {
                    *length = new_length;
                    return true;
                }

                let mut success = true;
                let mut highest_index = 0;

                let mut highest_non_configurable_index = 0;
                for (key, descriptor) in map.as_ref() {
                    if !descriptor.configurable() {
                        highest_non_configurable_index = highest_non_configurable_index.max(*key);
                    }
                }

                map.retain(|&key, descriptor| {
                    if key >= new_length {
                        if descriptor.configurable() {
                            if key <= highest_non_configurable_index {
                                return true;
                            }

                            return false;
                        }

                        success = false;
                    }

                    highest_index = highest_index.max(key);
                    true
                });

                if success {
                    *length = new_length;
                } else {
                    *length = highest_index.max(highest_non_configurable_index) + 1;
                }
                return success;
            }
        }

        true
    }

    #[must_use]
    pub(crate) fn array_length_writable(&self) -> bool {
        match self {
            IndexedProperties::DenseI32 {
                length_writable, ..
            }
            | IndexedProperties::DenseF64 {
                length_writable, ..
            }
            | IndexedProperties::DenseElement {
                length_writable, ..
            }
            | IndexedProperties::SparseElement {
                length_writable, ..
            }
            | IndexedProperties::SparseProperty {
                length_writable, ..
            } => *length_writable,
        }
    }

    pub(crate) fn set_array_length_writable(&mut self, writable: bool) {
        match self {
            IndexedProperties::DenseI32 {
                length_writable, ..
            }
            | IndexedProperties::DenseF64 {
                length_writable, ..
            }
            | IndexedProperties::DenseElement {
                length_writable, ..
            }
            | IndexedProperties::SparseElement {
                length_writable, ..
            }
            | IndexedProperties::SparseProperty {
                length_writable, ..
            } => *length_writable = writable,
        }
    }

    #[inline]
    fn property_simple_value(property: &CompletePropertyDescriptor) -> Option<&JsValue> {
        if let CompletePropertyDescriptor::Data {
            value,
            writable: true,
            enumerable: true,
            configurable: true,
        } = property
        {
            return Some(value);
        }
        None
    }

    /// Get a property descriptor if it exists.
    pub(crate) fn get(&self, key: u32) -> Option<CompletePropertyDescriptor> {
        let value = match self {
            Self::DenseI32 { vec, .. } => vec.get(key as usize).copied()?.into(),
            Self::DenseF64 { vec, .. } => vec.get(key as usize).copied()?.into(),
            Self::DenseElement { vec, .. } => vec.get(key as usize)?.clone(),
            Self::SparseElement { map, .. } => map.get(&key)?.clone(),
            Self::SparseProperty { map, .. } => return map.get(&key).cloned(),
        };

        Some(CompletePropertyDescriptor::Data {
            value,
            writable: true,
            enumerable: true,
            configurable: true,
        })
    }

    /// Helper function for converting from a dense storage type to sparse storage type.
    fn convert_dense_to_sparse_element<T>(vec: &mut ThinVec<T>) -> FxHashMap<u32, JsValue>
    where
        T: Into<JsValue>,
    {
        let data = std::mem::take(vec);

        data.into_iter()
            .enumerate()
            .map(|(index, value)| (index as u32, value.into()))
            .collect()
    }

    /// Helper function for converting from a dense storage type to sparse storage type.
    fn convert_dense_to_sparse_property<T>(
        vec: &mut ThinVec<T>,
    ) -> FxHashMap<u32, CompletePropertyDescriptor>
    where
        T: Into<JsValue>,
    {
        let data = std::mem::take(vec);

        data.into_iter()
            .enumerate()
            .map(|(index, value)| {
                (
                    index as u32,
                    CompletePropertyDescriptor::Data {
                        value: value.into(),
                        writable: true,
                        enumerable: true,
                        configurable: true,
                    },
                )
            })
            .collect()
    }

    fn convert_to_sparse_and_insert(
        &mut self,
        key: u32,
        property: CompletePropertyDescriptor,
    ) -> bool {
        let (mut map, mut length, length_writable) = match self {
            Self::DenseI32 {
                vec,
                length,
                length_writable,
            } => (
                Self::convert_dense_to_sparse_property(vec),
                *length,
                *length_writable,
            ),
            Self::DenseF64 {
                vec,
                length,
                length_writable,
            } => (
                Self::convert_dense_to_sparse_property(vec),
                *length,
                *length_writable,
            ),
            Self::DenseElement {
                vec,
                length,
                length_writable,
            } => (
                Self::convert_dense_to_sparse_property(vec),
                *length,
                *length_writable,
            ),
            Self::SparseElement {
                map,
                length,
                length_writable,
            } => (
                std::mem::take(map)
                    .into_iter()
                    .map(|(key, value)| {
                        (
                            key,
                            CompletePropertyDescriptor::Data {
                                value,
                                writable: true,
                                enumerable: true,
                                configurable: true,
                            },
                        )
                    })
                    .collect(),
                *length,
                *length_writable,
            ),
            Self::SparseProperty {
                map,
                length,
                length_writable,
            } => {
                if *length_writable && *length <= key {
                    *length = key + 1;
                }
                return map.insert(key, property).is_some();
            }
        };

        map.insert(key, property);

        if length_writable && length <= key {
            length = key + 1;
        }
        *self = Self::SparseProperty {
            map: Box::new(map),
            length,
            length_writable,
        };
        false
    }

    /// Inserts a property descriptor with the specified key.
    fn insert_property(&mut self, key: u32, property: CompletePropertyDescriptor) -> bool {
        let Some(value) = Self::property_simple_value(&property) else {
            return self.convert_to_sparse_and_insert(key, property);
        };

        self.insert_value(key, value)
    }

    pub(crate) fn insert_value(&mut self, key: u32, value: &JsValue) -> bool {
        match self {
            // Fast Path: continues array access.
            Self::DenseI32 {
                vec,
                length,
                length_writable,
            } if key <= vec.len() as u32 => {
                let len = vec.len() as u32;

                if let Some(val) = value.as_i32() {
                    // If the key is pointing one past the last element, we push it!
                    //
                    // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                    if key == len {
                        vec.push(val);
                        if *length_writable && *length <= key {
                            *length = key + 1;
                        }
                        return false;
                    }

                    // If the key points at an already taken index, set it.
                    vec[key as usize] = val;
                    return true;
                }

                if let Some(num) = value.as_number() {
                    let mut vec = vec.iter().copied().map(f64::from).collect::<ThinVec<_>>();

                    // If the key is pointing one past the last element, we push it!
                    //
                    // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                    if key == len {
                        vec.push(num);
                        if *length_writable && *length <= key {
                            *length = key + 1;
                        }

                        *self = Self::DenseF64 {
                            vec,
                            length: *length,
                            length_writable: *length_writable,
                        };
                        return false;
                    }

                    // If the key points at an already taken index, set it.
                    vec[key as usize] = num;
                    *self = Self::DenseF64 {
                        vec,
                        length: *length,
                        length_writable: *length_writable,
                    };
                    return true;
                }

                let mut vec = vec
                    .iter()
                    .copied()
                    .map(JsValue::from)
                    .collect::<ThinVec<_>>();

                // If the key is pointing one past the last element, we push it!
                //
                // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                if key == len {
                    vec.push(value.clone());
                    if *length_writable && *length <= key {
                        *length = key + 1;
                    }
                    *self = Self::DenseElement {
                        vec,
                        length: *length,
                        length_writable: *length_writable,
                    };
                    return false;
                }

                // If it the key points in at a already taken index, swap and return it.
                vec[key as usize] = value.clone();
                *self = Self::DenseElement {
                    vec,
                    length: *length,
                    length_writable: *length_writable,
                };
                true
            }
            Self::DenseF64 {
                vec,
                length,
                length_writable,
            } if key <= vec.len() as u32 => {
                let len = vec.len() as u32;

                if let Some(value) = value.as_number() {
                    // If the key is pointing one past the last element, we push it!
                    //
                    // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                    if key == len {
                        vec.push(value);
                        if *length_writable && *length <= key {
                            *length = key + 1;
                        }
                        return false;
                    }

                    // If the key points at an already taken index, set it.
                    vec[key as usize] = value;
                    return true;
                }

                let mut vec = vec
                    .iter()
                    .copied()
                    .map(JsValue::from)
                    .collect::<ThinVec<JsValue>>();

                // If the key is pointing one past the last element, we push it!
                //
                // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                if key == len {
                    if *length_writable && *length <= key {
                        *length = key + 1;
                    }

                    vec.push(value.clone());
                    *self = Self::DenseElement {
                        vec,
                        length: *length,
                        length_writable: *length_writable,
                    };
                    return false;
                }

                // If it the key points in at a already taken index, set it.
                vec[key as usize] = value.clone();
                *self = Self::DenseElement {
                    vec,
                    length: *length,
                    length_writable: *length_writable,
                };
                true
            }
            Self::DenseElement {
                vec,
                length,
                length_writable,
            } if key <= vec.len() as u32 => {
                let len = vec.len() as u32;

                // If the key is pointing one past the last element, we push it!
                //
                // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                if key == len {
                    vec.push(value.clone());

                    if *length_writable && *length <= key {
                        *length = key + 1;
                    }
                    return false;
                }

                // If the key points at an already taken index, set it.
                vec[key as usize] = value.clone();
                true
            }
            // Creating a hole: transition dense storage into sparse element storage.
            Self::DenseI32 {
                vec,
                length,
                length_writable,
            } => {
                if *length_writable && *length <= key {
                    *length = key + 1;
                }

                let mut values = Self::convert_dense_to_sparse_element(vec);
                let replaced = values.insert(key, value.clone()).is_some();
                *self = Self::SparseElement {
                    map: Box::new(values),
                    length: *length,
                    length_writable: *length_writable,
                };
                replaced
            }
            Self::DenseF64 {
                vec,
                length,
                length_writable,
            } => {
                if *length_writable && *length <= key {
                    *length = key + 1;
                }

                let mut values = Self::convert_dense_to_sparse_element(vec);
                let replaced = values.insert(key, value.clone()).is_some();
                *self = Self::SparseElement {
                    map: Box::new(values),
                    length: *length,
                    length_writable: *length_writable,
                };
                replaced
            }
            Self::DenseElement {
                vec,
                length,
                length_writable,
            } => {
                if *length_writable && *length <= key {
                    *length = key + 1;
                }

                let mut values = Self::convert_dense_to_sparse_element(vec);
                let replaced = values.insert(key, value.clone()).is_some();
                *self = Self::SparseElement {
                    map: Box::new(values),
                    length: *length,
                    length_writable: *length_writable,
                };
                replaced
            }
            Self::SparseElement {
                map,
                length,
                length_writable,
            } => {
                if *length_writable && *length <= key {
                    *length = key + 1;
                }
                map.insert(key, value.clone()).is_some()
            }
            Self::SparseProperty {
                map,
                length,
                length_writable,
            } => {
                if *length_writable && *length <= key {
                    *length = key + 1;
                }
                map.insert(
                    key,
                    CompletePropertyDescriptor::Data {
                        value: value.clone(),
                        writable: true,
                        enumerable: true,
                        configurable: true,
                    },
                )
                .is_some()
            }
        }
    }

    fn convert_to_sparse_and_remove(&mut self, key: u32) -> bool {
        let (mut map, length, length_writable) = match self {
            IndexedProperties::DenseI32 {
                vec,
                length,
                length_writable,
            } => (
                Self::convert_dense_to_sparse_element(vec),
                *length,
                *length_writable,
            ),
            IndexedProperties::DenseF64 {
                vec,
                length,
                length_writable,
            } => (
                Self::convert_dense_to_sparse_element(vec),
                *length,
                *length_writable,
            ),
            IndexedProperties::DenseElement {
                vec,
                length,
                length_writable,
            } => (
                Self::convert_dense_to_sparse_element(vec),
                *length,
                *length_writable,
            ),
            IndexedProperties::SparseElement { map, .. } => return map.remove(&key).is_some(),
            IndexedProperties::SparseProperty { map, .. } => return map.remove(&key).is_some(),
        };

        let removed = map.remove(&key).is_some();
        *self = Self::SparseElement {
            map: Box::new(map),
            length,
            length_writable,
        };
        removed
    }

    /// Removes a property descriptor with the specified key.
    fn remove(&mut self, key: u32) -> bool {
        match self {
            // Fast Paths: contiguous storage.
            //
            // If the key is pointing at the last element, then we pop it.
            Self::DenseI32 { vec, .. } if (key + 1) == vec.len() as u32 => {
                vec.pop();
                true
            }
            // If the key is out of range then don't do anything.
            Self::DenseI32 { vec, .. } if key >= vec.len() as u32 => false,
            Self::DenseF64 { vec, .. } if (key + 1) == vec.len() as u32 => {
                vec.pop();
                true
            }
            Self::DenseF64 { vec, .. } if key >= vec.len() as u32 => false,
            Self::DenseElement { vec, .. } if (key + 1) == vec.len() as u32 => {
                vec.pop();
                true
            }
            Self::DenseElement { vec, .. } if key >= vec.len() as u32 => false,
            // Slow Paths: non-contiguous storage.
            Self::SparseElement { map, .. } => map.remove(&key).is_some(),
            Self::SparseProperty { map, .. } => map.remove(&key).is_some(),
            _ => self.convert_to_sparse_and_remove(key),
        }
    }

    /// Check if we contain the key to a property descriptor.
    fn contains_key(&self, key: u32) -> bool {
        match self {
            Self::DenseI32 { vec, .. } => (0..vec.len() as u32).contains(&key),
            Self::DenseF64 { vec, .. } => (0..vec.len() as u32).contains(&key),
            Self::DenseElement { vec, .. } => (0..vec.len() as u32).contains(&key),
            Self::SparseElement { map, .. } => map.contains_key(&key),
            Self::SparseProperty { map, .. } => map.contains_key(&key),
        }
    }

    /// Transform this array into a sparse array if it isn't so already.
    pub(crate) fn transform_to_sparse(&mut self) {
        let (map, length, length_writable) = match self {
            Self::DenseI32 {
                vec,
                length,
                length_writable,
            } => (
                Self::convert_dense_to_sparse_element(vec),
                *length,
                *length_writable,
            ),
            Self::DenseF64 {
                vec,
                length,
                length_writable,
            } => (
                Self::convert_dense_to_sparse_element(vec),
                *length,
                *length_writable,
            ),
            Self::DenseElement {
                vec,
                length,
                length_writable,
            } => (
                Self::convert_dense_to_sparse_element(vec),
                *length,
                *length_writable,
            ),
            _ => {
                return;
            }
        };

        *self = Self::SparseElement {
            map: Box::new(map),
            length,
            length_writable,
        };
    }

    fn iter(&self) -> IndexProperties<'_> {
        match self {
            Self::DenseI32 { vec, .. } => IndexProperties::DenseI32(vec.iter().enumerate()),
            Self::DenseF64 { vec, .. } => IndexProperties::DenseF64(vec.iter().enumerate()),
            Self::DenseElement { vec, .. } => IndexProperties::DenseElement(vec.iter().enumerate()),
            Self::SparseElement { map, .. } => IndexProperties::SparseElement(map.iter()),
            Self::SparseProperty { map, .. } => IndexProperties::SparseProperty(map.iter()),
        }
    }

    fn keys(&self) -> IndexPropertyKeys<'_> {
        match self {
            Self::DenseI32 { vec, .. } => IndexPropertyKeys::Dense(0..vec.len() as u32),
            Self::DenseF64 { vec, .. } => IndexPropertyKeys::Dense(0..vec.len() as u32),
            Self::DenseElement { vec, .. } => IndexPropertyKeys::Dense(0..vec.len() as u32),
            Self::SparseElement { map, .. } => IndexPropertyKeys::SparseElement(map.keys()),
            Self::SparseProperty { map, .. } => IndexPropertyKeys::SparseProperty(map.keys()),
        }
    }

    fn values(&self) -> IndexPropertyValues<'_> {
        match self {
            Self::DenseI32 { vec, .. } => IndexPropertyValues::DenseI32(vec.iter()),
            Self::DenseF64 { vec, .. } => IndexPropertyValues::DenseF64(vec.iter()),
            Self::DenseElement { vec, .. } => IndexPropertyValues::DenseElement(vec.iter()),
            Self::SparseElement { map, .. } => IndexPropertyValues::SparseElement(map.values()),
            Self::SparseProperty { map, .. } => IndexPropertyValues::SparseProperty(map.values()),
        }
    }
}

impl<'a> IntoIterator for &'a IndexedProperties {
    type IntoIter = IndexProperties<'a>;
    type Item = (u32, CompletePropertyDescriptor);
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A [`PropertyMap`] contains all the properties of an object.
///
/// The property values are stored in different data structures based on keys.
#[derive(Default, Debug, Trace, Finalize)]
pub struct PropertyMap {
    /// Properties stored with integers as keys.
    pub(crate) indexed_properties: IndexedProperties,

    pub(crate) shape: Shape,
    pub(crate) storage: ObjectStorage,
}

impl PropertyMap {
    /// Create a new [`PropertyMap`].
    #[must_use]
    #[inline]
    pub fn new(shape: Shape, indexed_properties: IndexedProperties) -> Self {
        Self {
            indexed_properties,
            shape,
            storage: ThinVec::default(),
        }
    }

    /// Construct a [`PropertyMap`] with the given prototype with a unique [`Shape`].
    #[must_use]
    #[inline]
    pub fn from_prototype_unique_shape(prototype: JsPrototype) -> Self {
        Self {
            indexed_properties: IndexedProperties::default(),
            shape: UniqueShape::new(prototype, PropertyTableInner::default()).into(),
            storage: ThinVec::default(),
        }
    }

    /// Construct a [`PropertyMap`] with the given prototype with a shared shape [`Shape`].
    #[must_use]
    #[inline]
    pub fn from_prototype_with_shared_shape(
        root_shape: &RootShape,
        prototype: JsPrototype,
    ) -> Self {
        let shape = root_shape.shape().change_prototype_transition(prototype);
        Self {
            indexed_properties: IndexedProperties::default(),
            shape: shape.into(),
            storage: ThinVec::default(),
        }
    }

    /// Get the property with the given key from the [`PropertyMap`].
    #[must_use]
    pub fn get(&self, key: &PropertyKey) -> Option<CompletePropertyDescriptor> {
        if let PropertyKey::Index(index) = key {
            return self.indexed_properties.get(index.get());
        }
        if let Some(slot) = self.shape.lookup(key) {
            return Some(self.get_storage(slot));
        }

        None
    }

    /// Get the property with the given key from the [`PropertyMap`].
    #[must_use]
    pub(crate) fn get_with_slot(
        &self,
        key: &PropertyKey,
        out_slot: &mut Slot,
    ) -> Option<CompletePropertyDescriptor> {
        if let PropertyKey::Index(index) = key {
            return self.indexed_properties.get(index.get());
        }
        if let Some(slot) = self.shape.lookup(key) {
            out_slot.index = slot.index;

            // Remove all descriptor attributes, but keep inline caching bits.
            out_slot.attributes = (out_slot.attributes & SlotAttributes::INLINE_CACHE_BITS)
                | slot.attributes
                | SlotAttributes::FOUND;
            return Some(self.get_storage(slot));
        }

        None
    }

    /// Get the property with the given key from the [`PropertyMap`].
    #[must_use]
    pub(crate) fn get_storage(
        &self,
        Slot { index, attributes }: Slot,
    ) -> CompletePropertyDescriptor {
        let index = index as usize;
        if attributes.is_accessor_descriptor() {
            CompletePropertyDescriptor::Accessor {
                set: self.storage[index + 1]
                    .as_object()
                    .map(JsFunction::from_object_unchecked),
                get: self.storage[index]
                    .as_object()
                    .map(JsFunction::from_object_unchecked),
                configurable: attributes.contains(SlotAttributes::CONFIGURABLE),
                enumerable: attributes.contains(SlotAttributes::ENUMERABLE),
            }
        } else {
            CompletePropertyDescriptor::Data {
                value: self.storage[index].clone(),
                writable: attributes.contains(SlotAttributes::WRITABLE),
                configurable: attributes.contains(SlotAttributes::CONFIGURABLE),
                enumerable: attributes.contains(SlotAttributes::ENUMERABLE),
            }
        }
    }

    /// Insert the given property descriptor with the given key [`PropertyMap`].
    pub fn insert(&mut self, key: &PropertyKey, property: CompletePropertyDescriptor) -> bool {
        let mut dummy_slot = Slot::new();
        self.insert_with_slot(key, property, &mut dummy_slot)
    }

    /// Insert the given property descriptor with the given key [`PropertyMap`].
    pub(crate) fn insert_with_slot(
        &mut self,
        key: &PropertyKey,
        property: CompletePropertyDescriptor,
        out_slot: &mut Slot,
    ) -> bool {
        if let PropertyKey::Index(index) = key {
            return self
                .indexed_properties
                .insert_property(index.get(), property);
        }

        let attributes = property.to_slot_attributes();

        if let Some(slot) = self.shape.lookup(key) {
            let index = slot.index as usize;

            if slot.attributes != attributes {
                let key = TransitionKey {
                    property_key: key.clone(),
                    attributes,
                };
                let transition = self.shape.change_attributes_transition(key);
                self.shape = transition.shape;
                match transition.action {
                    ChangeTransitionAction::Nothing => {}
                    ChangeTransitionAction::Remove => {
                        self.storage.remove(slot.index as usize + 1);
                    }
                    ChangeTransitionAction::Insert => {
                        // insert after index which is (index + 1).
                        self.storage.insert(index, JsValue::undefined());
                    }
                }
            }

            match property {
                CompletePropertyDescriptor::Data { value, .. } => {
                    self.storage[index] = value;
                }
                CompletePropertyDescriptor::Accessor { get, set, .. } => {
                    self.storage[index + 1] = set.map(JsValue::new).unwrap_or_default();
                    self.storage[index] = get.map(JsValue::new).unwrap_or_default();
                }
            }
            out_slot.index = slot.index;
            out_slot.attributes =
                (out_slot.attributes & SlotAttributes::INLINE_CACHE_BITS) | attributes;
            return true;
        }

        let transition_key = TransitionKey {
            property_key: key.clone(),
            attributes,
        };
        self.shape = self.shape.insert_property_transition(transition_key);

        // Make Sure that if we are inserting, it has the correct slot index.
        debug_assert_eq!(
            self.shape.lookup(key),
            Some(Slot {
                index: self.storage.len() as u32,
                attributes
            })
        );

        out_slot.index = self.storage.len() as u32;
        out_slot.attributes =
            (out_slot.attributes & SlotAttributes::INLINE_CACHE_BITS) | attributes;

        match property {
            CompletePropertyDescriptor::Data { value, .. } => {
                self.storage.push(value);
            }
            CompletePropertyDescriptor::Accessor { get, set, .. } => {
                self.storage.push(get.map(JsValue::new).unwrap_or_default());
                self.storage.push(set.map(JsValue::new).unwrap_or_default());
            }
        }

        false
    }

    /// Remove the property with the given key from the [`PropertyMap`].
    pub fn remove(&mut self, key: &PropertyKey) -> bool {
        if let PropertyKey::Index(index) = key {
            return self.indexed_properties.remove(index.get());
        }
        if let Some(slot) = self.shape.lookup(key) {
            // shift all elements when removing.
            if slot.attributes.is_accessor_descriptor() {
                self.storage.remove(slot.index as usize + 1);
            }
            self.storage.remove(slot.index as usize);

            self.shape = self.shape.remove_property_transition(key);
            return true;
        }

        false
    }

    /// Overrides all the indexed properties, setting it to dense storage.
    pub(crate) fn override_indexed_properties(&mut self, properties: ThinVec<JsValue>) {
        let length = properties.len() as u32;
        self.indexed_properties = IndexedProperties::DenseElement {
            vec: properties,
            length,
            length_writable: true,
        };
    }

    pub(crate) fn get_dense_property(&self, index: u32) -> Option<JsValue> {
        let index = index as usize;
        match &self.indexed_properties {
            IndexedProperties::DenseI32 { vec, .. } => vec.get(index).copied().map(JsValue::from),
            IndexedProperties::DenseF64 { vec, .. } => vec.get(index).copied().map(JsValue::from),
            IndexedProperties::DenseElement { vec, .. } => vec.get(index).cloned(),
            IndexedProperties::SparseProperty { .. } | IndexedProperties::SparseElement { .. } => {
                None
            }
        }
    }

    pub(crate) fn set_dense_property(&mut self, index: u32, value: &JsValue) -> bool {
        let index = index as usize;

        match &mut self.indexed_properties {
            IndexedProperties::DenseI32 {
                vec,
                length,
                length_writable,
            } => {
                let Some(element) = vec.get_mut(index) else {
                    return false;
                };

                // If it can fit in a i32 and the truncated version is
                // equal to the original then it is an integer.
                let is_rational_integer = |n: f64| n.to_bits() == f64::from(n as i32).to_bits();

                let value = match value.variant() {
                    JsVariant::Integer32(n) => n,
                    JsVariant::Float64(n) if is_rational_integer(n) => n as i32,
                    JsVariant::Float64(value) => {
                        let mut vec = vec.iter().copied().map(f64::from).collect::<ThinVec<_>>();
                        vec[index] = value;
                        self.indexed_properties = IndexedProperties::DenseF64 {
                            vec,
                            length: *length,
                            length_writable: *length_writable,
                        };

                        return true;
                    }
                    _ => {
                        let mut vec = vec
                            .iter()
                            .copied()
                            .map(JsValue::from)
                            .collect::<ThinVec<_>>();
                        vec[index] = value.clone();
                        self.indexed_properties = IndexedProperties::DenseElement {
                            vec,
                            length: *length,
                            length_writable: *length_writable,
                        };

                        return true;
                    }
                };

                *element = value;
                true
            }
            IndexedProperties::DenseF64 {
                vec,
                length,
                length_writable,
            } => {
                let Some(element) = vec.get_mut(index) else {
                    return false;
                };

                let Some(value) = value.as_number() else {
                    let mut properties = vec
                        .iter()
                        .copied()
                        .map(JsValue::from)
                        .collect::<ThinVec<_>>();
                    properties[index] = value.clone();
                    self.indexed_properties = IndexedProperties::DenseElement {
                        vec: properties,
                        length: *length,
                        length_writable: *length_writable,
                    };
                    return true;
                };

                *element = value;
                true
            }
            IndexedProperties::DenseElement { vec, .. } => {
                let Some(element) = vec.get_mut(index) else {
                    return false;
                };
                *element = value.clone();
                true
            }
            IndexedProperties::SparseProperty { .. } | IndexedProperties::SparseElement { .. } => {
                false
            }
        }
    }

    /// Returns the vec of dense indexed properties if they exist.
    pub(crate) fn to_dense_indexed_properties(&self) -> Option<ThinVec<JsValue>> {
        match &self.indexed_properties {
            IndexedProperties::DenseI32 { vec, .. } => {
                Some(vec.iter().copied().map(JsValue::from).collect())
            }
            IndexedProperties::DenseF64 { vec, .. } => {
                Some(vec.iter().copied().map(JsValue::from).collect())
            }
            IndexedProperties::DenseElement { vec, .. } => Some(vec.iter().cloned().collect()),
            IndexedProperties::SparseProperty { .. } | IndexedProperties::SparseElement { .. } => {
                None
            }
        }
    }

    /// An iterator visiting all indexed key-value pairs in arbitrary order. The iterator element type is `(&'a u32, &'a Property)`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    #[must_use]
    pub fn index_properties(&self) -> IndexProperties<'_> {
        self.indexed_properties.iter()
    }

    /// An iterator visiting all index keys in arbitrary order. The iterator element type is `&'a u32`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    #[must_use]
    pub fn index_property_keys(&self) -> IndexPropertyKeys<'_> {
        self.indexed_properties.keys()
    }

    /// An iterator visiting all index values in arbitrary order. The iterator element type is `&'a Property`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    #[must_use]
    pub fn index_property_values(&self) -> IndexPropertyValues<'_> {
        self.indexed_properties.values()
    }

    /// Returns `true` if the given key is contained in the [`PropertyMap`].
    #[inline]
    #[must_use]
    pub fn contains_key(&self, key: &PropertyKey) -> bool {
        if let PropertyKey::Index(index) = key {
            return self.indexed_properties.contains_key(index.get());
        }
        if self.shape.lookup(key).is_some() {
            return true;
        }

        false
    }
}

/// An iterator over the indexed property entries of an `Object`.
#[derive(Debug, Clone)]
pub enum IndexProperties<'a> {
    /// An iterator over dense i32, Vec backed indexed property entries of an `Object`.
    DenseI32(std::iter::Enumerate<std::slice::Iter<'a, i32>>),

    /// An iterator over dense f64, Vec backed indexed property entries of an `Object`.
    DenseF64(std::iter::Enumerate<std::slice::Iter<'a, f64>>),

    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    DenseElement(std::iter::Enumerate<std::slice::Iter<'a, JsValue>>),

    /// An iterator over sparse, `HashMap` backed indexed property entries storing raw values.
    SparseElement(hash_map::Iter<'a, u32, JsValue>),

    /// An iterator over sparse, `HashMap` backed indexed property entries storing descriptors.
    SparseProperty(hash_map::Iter<'a, u32, CompletePropertyDescriptor>),
}

impl Iterator for IndexProperties<'_> {
    type Item = (u32, CompletePropertyDescriptor);

    fn next(&mut self) -> Option<Self::Item> {
        let (index, value) = match self {
            Self::DenseI32(vec) => vec
                .next()
                .map(|(index, value)| (index, JsValue::from(*value)))?,
            Self::DenseF64(vec) => vec
                .next()
                .map(|(index, value)| (index, JsValue::from(*value)))?,
            Self::DenseElement(vec) => vec.next().map(|(index, value)| (index, value.clone()))?,
            Self::SparseProperty(map) => {
                return map.next().map(|(index, value)| (*index, value.clone()));
            }
            Self::SparseElement(map) => map
                .next()
                .map(|(index, value)| (*index as usize, value.clone()))?,
        };

        Some((
            index as u32,
            CompletePropertyDescriptor::Data {
                value,
                writable: true,
                enumerable: true,
                configurable: true,
            },
        ))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::DenseI32(vec) => vec.size_hint(),
            Self::DenseF64(vec) => vec.size_hint(),
            Self::DenseElement(vec) => vec.size_hint(),
            Self::SparseProperty(map) => map.size_hint(),
            Self::SparseElement(map) => map.size_hint(),
        }
    }
}

impl ExactSizeIterator for IndexProperties<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::DenseI32(vec) => vec.len(),
            Self::DenseF64(vec) => vec.len(),
            Self::DenseElement(vec) => vec.len(),
            Self::SparseProperty(map) => map.len(),
            Self::SparseElement(map) => map.len(),
        }
    }
}

impl FusedIterator for IndexProperties<'_> {}

/// An iterator over the index keys (`u32`) of an `Object`.
#[derive(Debug, Clone)]
pub enum IndexPropertyKeys<'a> {
    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    Dense(std::ops::Range<u32>),

    /// An iterator over sparse, `HashMap` backed indexed property entries storing values.
    SparseElement(hash_map::Keys<'a, u32, JsValue>),

    /// An iterator over sparse, `HashMap` backed indexed property entries of an `Object`.
    SparseProperty(hash_map::Keys<'a, u32, CompletePropertyDescriptor>),
}

impl Iterator for IndexPropertyKeys<'_> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Dense(vec) => vec.next(),
            Self::SparseProperty(map) => map.next().copied(),
            Self::SparseElement(map) => map.next().copied(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Dense(vec) => vec.size_hint(),
            Self::SparseProperty(map) => map.size_hint(),
            Self::SparseElement(map) => map.size_hint(),
        }
    }
}

impl ExactSizeIterator for IndexPropertyKeys<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Dense(vec) => vec.len(),
            Self::SparseProperty(map) => map.len(),
            Self::SparseElement(map) => map.len(),
        }
    }
}

impl FusedIterator for IndexPropertyKeys<'_> {}

/// An iterator over the index values (`Property`) of an `Object`.
#[derive(Debug, Clone)]
#[allow(variant_size_differences)]
pub enum IndexPropertyValues<'a> {
    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    DenseI32(std::slice::Iter<'a, i32>),

    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    DenseF64(std::slice::Iter<'a, f64>),

    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    DenseElement(std::slice::Iter<'a, JsValue>),

    /// An iterator over sparse, `HashMap` backed indexed property entries storing values.
    SparseElement(hash_map::Values<'a, u32, JsValue>),

    /// An iterator over sparse, `HashMap` backed indexed property entries of an `Object`.
    SparseProperty(hash_map::Values<'a, u32, CompletePropertyDescriptor>),
}

impl Iterator for IndexPropertyValues<'_> {
    type Item = CompletePropertyDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        let value = match self {
            Self::DenseI32(vec) => vec.next().copied()?.into(),
            Self::DenseF64(vec) => vec.next().copied()?.into(),
            Self::DenseElement(vec) => vec.next().cloned()?,
            Self::SparseProperty(map) => return map.next().cloned(),
            Self::SparseElement(map) => map.next().cloned()?,
        };

        Some(CompletePropertyDescriptor::Data {
            value,
            writable: true,
            enumerable: true,
            configurable: true,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::DenseI32(vec) => vec.size_hint(),
            Self::DenseF64(vec) => vec.size_hint(),
            Self::DenseElement(vec) => vec.size_hint(),
            Self::SparseProperty(map) => map.size_hint(),
            Self::SparseElement(map) => map.size_hint(),
        }
    }
}

impl ExactSizeIterator for IndexPropertyValues<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::DenseI32(vec) => vec.len(),
            Self::DenseF64(vec) => vec.len(),
            Self::DenseElement(vec) => vec.len(),
            Self::SparseProperty(map) => map.len(),
            Self::SparseElement(map) => map.len(),
        }
    }
}
