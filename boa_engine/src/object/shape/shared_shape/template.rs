use boa_gc::{Finalize, Trace};
use thin_vec::ThinVec;

use crate::{
    object::{
        shape::{slot::SlotAttributes, Shape},
        JsObject, Object, ObjectData, PropertyMap,
    },
    property::{Attribute, PropertyKey},
    JsValue,
};

use super::{SharedShape, TransitionKey};

/// Represent a template of an objects properties and prototype.
/// This is used to construct as many objects  as needed from a predefined [`SharedShape`].
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct ObjectTemplate {
    shape: SharedShape,
}

impl ObjectTemplate {
    /// Create a new [`ObjectTemplate`]
    pub(crate) fn new(root_shape: &SharedShape) -> Self {
        Self {
            shape: root_shape.clone(),
        }
    }

    /// Create and [`ObjectTemplate`] with a prototype.
    pub(crate) fn with_prototype(root_shape: &SharedShape, prototype: JsObject) -> Self {
        let shape = root_shape.change_prototype_transition(Some(prototype));
        Self { shape }
    }

    /// Check if the shape has a specific, prototype.
    pub(crate) fn has_prototype(&self, prototype: &JsObject) -> bool {
        self.shape.has_prototype(prototype)
    }

    /// Set the prototype of the [`ObjectTemplate`].
    ///
    /// This assumes that the prototype has not been set yet.
    pub(crate) fn set_prototype(&mut self, prototype: JsObject) -> &mut Self {
        self.shape = self.shape.change_prototype_transition(Some(prototype));
        self
    }

    /// Add a data property to the [`ObjectTemplate`].
    ///
    /// This assumes that the property with the given key was not previously set
    /// and that it's a string or symbol.
    pub(crate) fn property(&mut self, key: PropertyKey, attributes: Attribute) -> &mut Self {
        debug_assert!(!matches!(&key, PropertyKey::Index(_)));

        let attributes = SlotAttributes::from_bits_truncate(attributes.bits());
        self.shape = self.shape.insert_property_transition(TransitionKey {
            property_key: key,
            attributes,
        });

        self
    }

    /// Add a accessor property to the [`ObjectTemplate`].
    ///
    /// This assumes that the property with the given key was not previously set
    /// and that it's a string or symbol.
    pub(crate) fn accessor(
        &mut self,
        key: PropertyKey,
        get: bool,
        set: bool,
        attributes: Attribute,
    ) -> &mut Self {
        // TOOD: We don't support indexed keys.
        debug_assert!(!matches!(&key, PropertyKey::Index(_)));

        let attributes = {
            let mut result = SlotAttributes::empty();
            result.set(
                SlotAttributes::CONFIGURABLE,
                attributes.contains(Attribute::CONFIGURABLE),
            );
            result.set(
                SlotAttributes::ENUMERABLE,
                attributes.contains(Attribute::ENUMERABLE),
            );

            result.set(SlotAttributes::GET, get);
            result.set(SlotAttributes::SET, set);

            result
        };

        self.shape = self.shape.insert_property_transition(TransitionKey {
            property_key: key,
            attributes,
        });

        self
    }

    /// Create an object from the [`ObjectTemplate`]
    ///
    /// The storage must match the properties provided.
    pub(crate) fn create(&self, data: ObjectData, storage: Vec<JsValue>) -> JsObject {
        let mut object = Object {
            kind: data.kind,
            extensible: true,
            properties: PropertyMap::new(Shape::shared(self.shape.clone()), ThinVec::default()),
            private_elements: ThinVec::new(),
        };

        object.properties.storage = storage;

        JsObject::from_object_and_vtable(object, data.internal_methods)
    }

    /// Create an object from the [`ObjectTemplate`]
    ///
    /// The storage must match the properties provided. It does not apply to
    /// the indexed propeties.
    pub(crate) fn create_with_indexed_properties(
        &self,
        data: ObjectData,
        storage: Vec<JsValue>,
        elements: ThinVec<JsValue>,
    ) -> JsObject {
        let mut object = Object {
            kind: data.kind,
            extensible: true,
            properties: PropertyMap::new(Shape::shared(self.shape.clone()), elements),
            private_elements: ThinVec::new(),
        };

        object.properties.storage = storage;

        JsObject::from_object_and_vtable(object, data.internal_methods)
    }
}
