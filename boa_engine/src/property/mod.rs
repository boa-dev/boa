//! This module implements the Property Descriptor.
//!
//! The Property Descriptor type is used to explain the manipulation and reification of `Object`
//! property attributes. Values of the Property Descriptor type are Records. Each field's name is
//! an attribute name and its value is a corresponding attribute value as specified in
//! [6.1.7.1][section]. In addition, any field may be present or absent. The schema name used
//! within this specification to tag literal descriptions of Property Descriptor records is
//! `PropertyDescriptor`.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-property-descriptor-specification-type
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/defineProperty
//! [section]: https://tc39.es/ecma262/#sec-property-attributes

use crate::{js_string, JsString, JsSymbol, JsValue};
use boa_gc::{Finalize, Trace};
use std::fmt;

mod attribute;
pub use attribute::Attribute;

/// This represents a JavaScript Property AKA The Property Descriptor.
///
/// Property descriptors present in objects come in three main flavors:
///  - data descriptors
///  - accessor descriptors
///  - generic descriptor
///
/// A data Property Descriptor is one that includes any fields named either
/// \[\[Value\]\] or \[\[Writable\]\].
///
/// An accessor Property Descriptor is one that includes any fields named either
/// \[\[Get\]\] or \[\[Set\]\].
///
/// A generic Property Descriptor is a Property Descriptor value that is neither
/// a data Property Descriptor nor an accessor Property Descriptor.
///
/// More information:
/// - [MDN documentation][mdn]
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-property-descriptor-specification-type
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/defineProperty
#[derive(Default, Debug, Clone, Trace, Finalize)]
pub struct PropertyDescriptor {
    enumerable: Option<bool>,
    configurable: Option<bool>,
    kind: DescriptorKind,
}

#[derive(Debug, Clone, Trace, Finalize)]
pub enum DescriptorKind {
    Data {
        value: Option<JsValue>,
        writable: Option<bool>,
    },
    Accessor {
        get: Option<JsValue>,
        set: Option<JsValue>,
    },
    Generic,
}

impl Default for DescriptorKind {
    fn default() -> Self {
        Self::Generic
    }
}

impl PropertyDescriptor {
    /// An accessor Property Descriptor is one that includes any fields named either `[[Get]]` or `[[Set]]`.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isaccessordescriptor
    #[inline]
    pub fn is_accessor_descriptor(&self) -> bool {
        matches!(self.kind, DescriptorKind::Accessor { .. })
    }

    /// A data Property Descriptor is one that includes any fields named either `[[Value]]` or `[[Writable]]`.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isdatadescriptor
    #[inline]
    pub fn is_data_descriptor(&self) -> bool {
        matches!(self.kind, DescriptorKind::Data { .. })
    }

    /// A generic Property Descriptor is one that is neither a data descriptor nor an accessor descriptor.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isgenericdescriptor
    #[inline]
    pub fn is_generic_descriptor(&self) -> bool {
        matches!(self.kind, DescriptorKind::Generic)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.is_generic_descriptor() && self.enumerable.is_none() && self.configurable.is_none()
    }

    #[inline]
    pub fn enumerable(&self) -> Option<bool> {
        self.enumerable
    }

    #[inline]
    pub fn configurable(&self) -> Option<bool> {
        self.configurable
    }

    #[inline]
    pub fn writable(&self) -> Option<bool> {
        match self.kind {
            DescriptorKind::Data { writable, .. } => writable,
            _ => None,
        }
    }

    #[inline]
    pub fn value(&self) -> Option<&JsValue> {
        match &self.kind {
            DescriptorKind::Data { value, .. } => value.as_ref(),
            _ => None,
        }
    }

    #[inline]
    pub fn get(&self) -> Option<&JsValue> {
        match &self.kind {
            DescriptorKind::Accessor { get, .. } => get.as_ref(),
            _ => None,
        }
    }

    #[inline]
    pub fn set(&self) -> Option<&JsValue> {
        match &self.kind {
            DescriptorKind::Accessor { set, .. } => set.as_ref(),
            _ => None,
        }
    }

    #[inline]
    pub fn expect_enumerable(&self) -> bool {
        if let Some(enumerable) = self.enumerable {
            enumerable
        } else {
            panic!("[[enumerable]] field not in property descriptor")
        }
    }

    #[inline]
    pub fn expect_configurable(&self) -> bool {
        if let Some(configurable) = self.configurable {
            configurable
        } else {
            panic!("[[configurable]] field not in property descriptor")
        }
    }

    #[inline]
    pub fn expect_writable(&self) -> bool {
        if let Some(writable) = self.writable() {
            writable
        } else {
            panic!("[[writable]] field not in property descriptor")
        }
    }

    #[inline]
    pub fn expect_value(&self) -> &JsValue {
        if let Some(value) = self.value() {
            value
        } else {
            panic!("[[value]] field not in property descriptor")
        }
    }

    #[inline]
    pub fn expect_get(&self) -> &JsValue {
        if let Some(get) = self.get() {
            get
        } else {
            panic!("[[get]] field not in property descriptor")
        }
    }

    #[inline]
    pub fn expect_set(&self) -> &JsValue {
        if let Some(set) = self.set() {
            set
        } else {
            panic!("[[set]] field not in property descriptor")
        }
    }

    #[inline]
    pub fn kind(&self) -> &DescriptorKind {
        &self.kind
    }

    #[inline]
    pub fn builder() -> PropertyDescriptorBuilder {
        PropertyDescriptorBuilder::new()
    }

    #[inline]
    #[must_use]
    pub fn into_accessor_defaulted(mut self) -> Self {
        self.kind = DescriptorKind::Accessor {
            get: self.get().cloned(),
            set: self.set().cloned(),
        };
        PropertyDescriptorBuilder { inner: self }
            .complete_with_defaults()
            .build()
    }

    #[must_use]
    pub fn into_data_defaulted(mut self) -> Self {
        self.kind = DescriptorKind::Data {
            value: self.value().cloned(),
            writable: self.writable(),
        };
        PropertyDescriptorBuilder { inner: self }
            .complete_with_defaults()
            .build()
    }

    #[inline]
    #[must_use]
    pub fn complete_property_descriptor(self) -> Self {
        PropertyDescriptorBuilder { inner: self }
            .complete_with_defaults()
            .build()
    }

    #[inline]
    pub fn fill_with(&mut self, desc: &Self) {
        match (&mut self.kind, &desc.kind) {
            (
                DescriptorKind::Data { value, writable },
                DescriptorKind::Data {
                    value: desc_value,
                    writable: desc_writable,
                },
            ) => {
                if let Some(desc_value) = desc_value {
                    *value = Some(desc_value.clone());
                }
                if let Some(desc_writable) = desc_writable {
                    *writable = Some(*desc_writable);
                }
            }
            (
                DescriptorKind::Accessor { get, set },
                DescriptorKind::Accessor {
                    get: desc_get,
                    set: desc_set,
                },
            ) => {
                if let Some(desc_get) = desc_get {
                    *get = Some(desc_get.clone());
                }
                if let Some(desc_set) = desc_set {
                    *set = Some(desc_set.clone());
                }
            }
            (_, DescriptorKind::Generic) => {}
            _ => panic!("Tried to fill a descriptor with an incompatible descriptor"),
        }

        if let Some(enumerable) = desc.enumerable {
            self.enumerable = Some(enumerable);
        }
        if let Some(configurable) = desc.configurable {
            self.configurable = Some(configurable);
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct PropertyDescriptorBuilder {
    inner: PropertyDescriptor,
}

impl PropertyDescriptorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn value<V: Into<JsValue>>(mut self, value: V) -> Self {
        match self.inner.kind {
            DescriptorKind::Data {
                value: ref mut v, ..
            } => *v = Some(value.into()),
            // TODO: maybe panic when trying to convert accessor to data?
            _ => {
                self.inner.kind = DescriptorKind::Data {
                    value: Some(value.into()),
                    writable: None,
                }
            }
        }
        self
    }

    #[must_use]
    pub fn writable(mut self, writable: bool) -> Self {
        match self.inner.kind {
            DescriptorKind::Data {
                writable: ref mut w,
                ..
            } => *w = Some(writable),
            // TODO: maybe panic when trying to convert accessor to data?
            _ => {
                self.inner.kind = DescriptorKind::Data {
                    value: None,
                    writable: Some(writable),
                }
            }
        }
        self
    }

    #[must_use]
    pub fn get<V: Into<JsValue>>(mut self, get: V) -> Self {
        match self.inner.kind {
            DescriptorKind::Accessor { get: ref mut g, .. } => *g = Some(get.into()),
            // TODO: maybe panic when trying to convert data to accessor?
            _ => {
                self.inner.kind = DescriptorKind::Accessor {
                    get: Some(get.into()),
                    set: None,
                }
            }
        }
        self
    }

    #[must_use]
    pub fn set<V: Into<JsValue>>(mut self, set: V) -> Self {
        match self.inner.kind {
            DescriptorKind::Accessor { set: ref mut s, .. } => *s = Some(set.into()),
            // TODO: maybe panic when trying to convert data to accessor?
            _ => {
                self.inner.kind = DescriptorKind::Accessor {
                    set: Some(set.into()),
                    get: None,
                }
            }
        }
        self
    }

    #[must_use]
    pub fn maybe_enumerable(mut self, enumerable: Option<bool>) -> Self {
        if let Some(enumerable) = enumerable {
            self = self.enumerable(enumerable);
        }
        self
    }

    #[must_use]
    pub fn maybe_configurable(mut self, configurable: Option<bool>) -> Self {
        if let Some(configurable) = configurable {
            self = self.configurable(configurable);
        }
        self
    }

    #[must_use]
    pub fn maybe_value<V: Into<JsValue>>(mut self, value: Option<V>) -> Self {
        if let Some(value) = value {
            self = self.value(value);
        }
        self
    }

    #[must_use]
    pub fn maybe_writable(mut self, writable: Option<bool>) -> Self {
        if let Some(writable) = writable {
            self = self.writable(writable);
        }
        self
    }

    #[must_use]
    pub fn maybe_get<V: Into<JsValue>>(mut self, get: Option<V>) -> Self {
        if let Some(get) = get {
            self = self.get(get);
        }
        self
    }

    #[must_use]
    pub fn maybe_set<V: Into<JsValue>>(mut self, set: Option<V>) -> Self {
        if let Some(set) = set {
            self = self.set(set);
        }
        self
    }

    #[must_use]
    pub fn enumerable(mut self, enumerable: bool) -> Self {
        self.inner.enumerable = Some(enumerable);
        self
    }

    #[must_use]
    pub fn configurable(mut self, configurable: bool) -> Self {
        self.inner.configurable = Some(configurable);
        self
    }

    #[must_use]
    pub fn complete_with_defaults(mut self) -> Self {
        match self.inner.kind {
            DescriptorKind::Generic => {
                self.inner.kind = DescriptorKind::Data {
                    value: Some(JsValue::undefined()),
                    writable: Some(false),
                }
            }
            DescriptorKind::Data {
                ref mut value,
                ref mut writable,
            } => {
                if value.is_none() {
                    *value = Some(JsValue::undefined());
                }
                if writable.is_none() {
                    *writable = Some(false);
                }
            }
            DescriptorKind::Accessor {
                ref mut set,
                ref mut get,
            } => {
                if set.is_none() {
                    *set = Some(JsValue::undefined());
                }
                if get.is_none() {
                    *get = Some(JsValue::undefined());
                }
            }
        }
        if self.inner.configurable.is_none() {
            self.inner.configurable = Some(false);
        }
        if self.inner.enumerable.is_none() {
            self.inner.enumerable = Some(false);
        }
        self
    }

    pub fn inner(&self) -> &PropertyDescriptor {
        &self.inner
    }

    pub fn build(self) -> PropertyDescriptor {
        self.inner
    }
}

impl From<PropertyDescriptorBuilder> for PropertyDescriptor {
    fn from(builder: PropertyDescriptorBuilder) -> Self {
        builder.build()
    }
}

/// This abstracts away the need for `IsPropertyKey` by transforming the `PropertyKey`
/// values into an enum with both valid types: String and Symbol
///
/// More information:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ispropertykey
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum PropertyKey {
    String(JsString),
    Symbol(JsSymbol),
    Index(u32),
}

impl From<JsString> for PropertyKey {
    #[inline]
    fn from(string: JsString) -> Self {
        if let Some(index) = string.to_std_string().ok().and_then(|s| s.parse().ok()) {
            Self::Index(index)
        } else {
            Self::String(string)
        }
    }
}

impl From<&str> for PropertyKey {
    #[inline]
    fn from(string: &str) -> Self {
        if let Ok(index) = string.parse() {
            Self::Index(index)
        } else {
            Self::String(string.into())
        }
    }
}

impl From<String> for PropertyKey {
    #[inline]
    fn from(string: String) -> Self {
        if let Ok(index) = string.parse() {
            Self::Index(index)
        } else {
            Self::String(string.into())
        }
    }
}

impl From<Box<str>> for PropertyKey {
    #[inline]
    fn from(string: Box<str>) -> Self {
        if let Ok(index) = string.parse() {
            Self::Index(index)
        } else {
            Self::String(string.as_ref().into())
        }
    }
}

impl From<JsSymbol> for PropertyKey {
    #[inline]
    fn from(symbol: JsSymbol) -> Self {
        Self::Symbol(symbol)
    }
}

impl fmt::Display for PropertyKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(ref string) => string.to_std_string_escaped().fmt(f),
            Self::Symbol(ref symbol) => symbol.descriptive_string().to_std_string_escaped().fmt(f),
            Self::Index(index) => index.fmt(f),
        }
    }
}

impl From<&PropertyKey> for JsValue {
    #[inline]
    fn from(property_key: &PropertyKey) -> Self {
        match property_key {
            PropertyKey::String(ref string) => string.clone().into(),
            PropertyKey::Symbol(ref symbol) => symbol.clone().into(),
            PropertyKey::Index(index) => {
                if let Ok(integer) = i32::try_from(*index) {
                    Self::new(integer)
                } else {
                    Self::new(*index)
                }
            }
        }
    }
}

impl From<PropertyKey> for JsValue {
    #[inline]
    fn from(property_key: PropertyKey) -> Self {
        match property_key {
            PropertyKey::String(ref string) => string.clone().into(),
            PropertyKey::Symbol(ref symbol) => symbol.clone().into(),
            PropertyKey::Index(index) => index.to_string().into(),
        }
    }
}

impl From<u8> for PropertyKey {
    fn from(value: u8) -> Self {
        Self::Index(value.into())
    }
}

impl From<u16> for PropertyKey {
    fn from(value: u16) -> Self {
        Self::Index(value.into())
    }
}

impl From<u32> for PropertyKey {
    fn from(value: u32) -> Self {
        Self::Index(value)
    }
}

impl From<usize> for PropertyKey {
    fn from(value: usize) -> Self {
        if let Ok(index) = u32::try_from(value) {
            Self::Index(index)
        } else {
            Self::String(js_string!(value.to_string()))
        }
    }
}

impl From<i64> for PropertyKey {
    fn from(value: i64) -> Self {
        if let Ok(index) = u32::try_from(value) {
            Self::Index(index)
        } else {
            Self::String(js_string!(value.to_string()))
        }
    }
}

impl From<u64> for PropertyKey {
    fn from(value: u64) -> Self {
        if let Ok(index) = u32::try_from(value) {
            Self::Index(index)
        } else {
            Self::String(js_string!(value.to_string()))
        }
    }
}

impl From<isize> for PropertyKey {
    fn from(value: isize) -> Self {
        if let Ok(index) = u32::try_from(value) {
            Self::Index(index)
        } else {
            Self::String(js_string!(value.to_string()))
        }
    }
}

impl From<i32> for PropertyKey {
    fn from(value: i32) -> Self {
        if let Ok(index) = u32::try_from(value) {
            Self::Index(index)
        } else {
            Self::String(js_string!(value.to_string()))
        }
    }
}

impl From<f64> for PropertyKey {
    fn from(value: f64) -> Self {
        use num_traits::cast::FromPrimitive;
        if let Some(index) = u32::from_f64(value) {
            return Self::Index(index);
        }

        Self::String(ryu_js::Buffer::new().format(value).into())
    }
}

impl PartialEq<[u16]> for PropertyKey {
    fn eq(&self, other: &[u16]) -> bool {
        match self {
            Self::String(ref string) => string == other,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PropertyNameKind {
    Key,
    Value,
    KeyAndValue,
}
