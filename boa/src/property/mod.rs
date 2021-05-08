//! This module implements the Property Descriptor.
//!
//! The Property Descriptor type is used to explain the manipulation and reification of Object property attributes.
//! Values of the Property Descriptor type are Records. Each field's name is an attribute name
//! and its value is a corresponding attribute value as specified in [6.1.7.1][section].
//! In addition, any field may be present or absent.
//! The schema name used within this specification to tag literal descriptions of Property Descriptor records is “PropertyDescriptor”.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-property-descriptor-specification-type
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/defineProperty
//! [section]: https://tc39.es/ecma262/#sec-property-attributes

use crate::{
    gc::{empty_trace, Finalize, Trace},
    object::GcObject,
    symbol::RcSymbol,
    value::{RcString, Value},
};
use std::{cell::RefCell, convert::TryFrom, fmt, marker::PhantomData};

mod attribute;
pub use attribute::Attribute;
use rustc_hash::FxHashMap;

/// A data descriptor is a property that has a value, which may or may not be writable.
///
/// More information:
/// - [MDN documentation][mdn]
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-property-descriptor-specification-type
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/defineProperty
#[derive(Debug, Clone, Trace, Finalize)]
pub struct DataDescriptor {
    value: Value,
    attributes: Attribute,
}

impl DataDescriptor {
    /// Create a new `DataDescriptor`.
    #[inline]
    pub fn new<V>(value: V, attributes: Attribute) -> Self
    where
        V: Into<Value>,
    {
        Self {
            value: value.into(),
            attributes,
        }
    }

    /// Return the `value` of the data descriptor.
    #[inline]
    pub fn value(&self) -> Value {
        self.value.clone()
    }

    /// Return the attributes of the descriptor.
    #[inline]
    pub fn attributes(&self) -> Attribute {
        self.attributes
    }

    /// Check whether the descriptor is configurable.
    #[inline]
    pub fn configurable(&self) -> bool {
        self.attributes.configurable()
    }

    /// Set whether the descriptor is configurable.
    #[inline]
    pub fn set_configurable(&mut self, configurable: bool) {
        self.attributes.set_configurable(configurable)
    }

    /// Check whether the descriptor is enumerable.
    #[inline]
    pub fn enumerable(&self) -> bool {
        self.attributes.enumerable()
    }

    /// Set whether the descriptor is enumerable.
    #[inline]
    pub fn set_enumerable(&mut self, enumerable: bool) {
        self.attributes.set_enumerable(enumerable)
    }

    /// Check whether the descriptor is writable.
    #[inline]
    pub fn writable(&self) -> bool {
        self.attributes.writable()
    }

    /// Set whether the descriptor is writable.
    #[inline]
    pub fn set_writable(&mut self, writable: bool) {
        self.attributes.set_writable(writable)
    }
}

impl From<DataDescriptor> for PropertyDescriptor {
    #[inline]
    fn from(value: DataDescriptor) -> Self {
        Self::Data(value)
    }
}

/// An accessor descriptor is a property described by a getter-setter pair of functions.
///
/// More information:
/// - [MDN documentation][mdn]
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-property-descriptor-specification-type
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/defineProperty
#[derive(Debug, Clone, Trace, Finalize)]
pub struct AccessorDescriptor {
    /// The function serving as getter.
    pub(crate) get: Option<GcObject>,
    /// The function serving as setter.
    pub(crate) set: Option<GcObject>,
    /// The attributes of the accessor descriptor.
    pub(crate) attributes: Attribute,
}

impl AccessorDescriptor {
    /// Create a new `AccessorDescriptor`.
    ///
    /// If the `attributes` argument contains a `writable` flag, it will be removed so only `enumerable`
    /// and `configurable` remains.
    #[inline]
    pub fn new(get: Option<GcObject>, set: Option<GcObject>, mut attributes: Attribute) -> Self {
        // Accessors can not have writable attribute.
        attributes.remove(Attribute::WRITABLE);
        Self {
            get,
            set,
            attributes,
        }
    }

    /// Return the getter if it exists.
    #[inline]
    pub fn getter(&self) -> Option<&GcObject> {
        self.get.as_ref()
    }

    /// Return the setter if it exists.
    #[inline]
    pub fn setter(&self) -> Option<&GcObject> {
        self.set.as_ref()
    }

    /// Set the getter of the accessor descriptor.
    #[inline]
    pub fn set_getter(&mut self, get: Option<GcObject>) {
        self.get = get;
    }

    /// Set the setter of the accessor descriptor.
    #[inline]
    pub fn set_setter(&mut self, set: Option<GcObject>) {
        self.set = set;
    }

    /// Return the attributes of the accessor descriptor.
    ///
    /// It is guaranteed to not contain a `writable` flag
    #[inline]
    pub fn attributes(&self) -> Attribute {
        self.attributes
    }

    /// Check whether the descriptor is configurable.
    #[inline]
    pub fn configurable(&self) -> bool {
        self.attributes.configurable()
    }

    /// Set whether the descriptor is configurable.
    #[inline]
    pub fn set_configurable(&mut self, configurable: bool) {
        self.attributes.set_configurable(configurable)
    }

    /// Check whether the descriptor is enumerable.
    #[inline]
    pub fn enumerable(&self) -> bool {
        self.attributes.enumerable()
    }

    /// Set whether the descriptor is enumerable.
    #[inline]
    pub fn set_enumerable(&mut self, enumerable: bool) {
        self.attributes.set_enumerable(enumerable)
    }
}

impl From<AccessorDescriptor> for PropertyDescriptor {
    #[inline]
    fn from(value: AccessorDescriptor) -> Self {
        Self::Accessor(value)
    }
}

/// This represents a JavaScript Property AKA The Property Descriptor.
///
/// Property descriptors present in objects come in two main flavors:
///  - data descriptors
///  - accessor descriptors
///
/// A data descriptor is a property that has a value, which may or may not be writable.
/// An accessor descriptor is a property described by a getter-setter pair of functions.
/// A descriptor must be one of these two flavors; it cannot be both.
///
/// More information:
/// - [MDN documentation][mdn]
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-property-descriptor-specification-type
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/defineProperty
#[derive(Debug, Clone, Trace, Finalize)]
pub enum PropertyDescriptor {
    Accessor(AccessorDescriptor),
    Data(DataDescriptor),
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
        matches!(self, Self::Accessor(_))
    }

    /// Return `Some()` if it is a accessor descriptor, `None` otherwise.
    #[inline]
    pub fn as_accessor_descriptor(&self) -> Option<&AccessorDescriptor> {
        match self {
            Self::Accessor(ref accessor) => Some(accessor),
            _ => None,
        }
    }

    /// A data Property Descriptor is one that includes any fields named either `[[Value]]` or `[[Writable]]`.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isdatadescriptor
    #[inline]
    pub fn is_data_descriptor(&self) -> bool {
        matches!(self, Self::Data(_))
    }

    /// Return `Some()` if it is a data descriptor, `None` otherwise.
    #[inline]
    pub fn as_data_descriptor(&self) -> Option<&DataDescriptor> {
        match self {
            Self::Data(ref data) => Some(data),
            _ => None,
        }
    }

    /// Check whether the descriptor is enumerable.
    #[inline]
    pub fn enumerable(&self) -> bool {
        match self {
            Self::Accessor(ref accessor) => accessor.enumerable(),
            Self::Data(ref data) => data.enumerable(),
        }
    }

    /// Check whether the descriptor is configurable.
    #[inline]
    pub fn configurable(&self) -> bool {
        match self {
            Self::Accessor(ref accessor) => accessor.configurable(),
            Self::Data(ref data) => data.configurable(),
        }
    }

    /// Return the attributes of the descriptor.
    #[inline]
    pub fn attributes(&self) -> Attribute {
        match self {
            Self::Accessor(ref accessor) => accessor.attributes(),
            Self::Data(ref data) => data.attributes(),
        }
    }
}

#[derive(Debug)]
struct Table {
    names: FxHashMap<&'static str, u32>,
    indexes: Vec<&'static str>,
}

impl Table {
    fn new() -> Self {
        Self {
            names: Default::default(),
            indexes: Default::default(),
        }
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        for s in self.indexes.iter_mut() {
            unsafe {
                let _ = Box::from_raw((*s) as *const _ as *mut str);
            }
        }
    }
}

thread_local! {
    static TABLE: RefCell<Table> = RefCell::new(Table::new());
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Name {
    value: u32,
    _marker: PhantomData<*mut str>,
}

impl Name {
    #[inline]
    pub fn new<S>(string: S) -> Name
    where
        S: AsRef<str>,
    {
        let string = string.as_ref();
        TABLE.with(|table| {
            let mut table = table.borrow_mut();

            if let Some(value) = table.names.get(string).cloned() {
                return Name {
                    value,
                    _marker: PhantomData,
                };
            }

            let string: &'static str = Box::leak(Box::<str>::from(string));
            let id = table.indexes.len() as u32;
            table.indexes.push(string);
            table.names.insert(string, id);

            Name {
                value: id,
                _marker: PhantomData,
            }
        })
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        TABLE.with(|table| {
            let table = table.borrow();

            table.indexes[self.value as usize]
        })
    }
}

impl AsRef<str> for Name {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Finalize for Name {}

unsafe impl Trace for Name {
    empty_trace!();
}

impl fmt::Display for Name {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// This abstracts away the need for IsPropertyKey by transforming the PropertyKey
/// values into an enum with both valid types: String and Symbol
///
/// More information:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ispropertykey
#[derive(Trace, Finalize, Debug, Clone)]
pub enum PropertyKey {
    String(Name),
    Symbol(RcSymbol),
    Index(u32),
}

impl From<Name> for PropertyKey {
    #[inline]
    fn from(name: Name) -> Self {
        PropertyKey::String(name)
    }
}

impl From<RcString> for PropertyKey {
    #[inline]
    fn from(string: RcString) -> PropertyKey {
        if let Ok(index) = string.parse() {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(Name::new(string))
        }
    }
}

impl From<&str> for PropertyKey {
    #[inline]
    fn from(string: &str) -> PropertyKey {
        if let Ok(index) = string.parse() {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(Name::new(string))
        }
    }
}

impl From<String> for PropertyKey {
    #[inline]
    fn from(string: String) -> PropertyKey {
        if let Ok(index) = string.parse() {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(Name::new(string))
        }
    }
}

impl From<Box<str>> for PropertyKey {
    #[inline]
    fn from(string: Box<str>) -> PropertyKey {
        if let Ok(index) = string.parse() {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(Name::new(string))
        }
    }
}

impl From<RcSymbol> for PropertyKey {
    #[inline]
    fn from(symbol: RcSymbol) -> PropertyKey {
        PropertyKey::Symbol(symbol)
    }
}

impl fmt::Display for PropertyKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyKey::String(ref string) => string.fmt(f),
            PropertyKey::Symbol(ref symbol) => symbol.fmt(f),
            PropertyKey::Index(index) => index.fmt(f),
        }
    }
}

impl From<&PropertyKey> for Value {
    #[inline]
    fn from(property_key: &PropertyKey) -> Value {
        match property_key {
            PropertyKey::String(ref string) => string.as_str().into(),
            PropertyKey::Symbol(ref symbol) => symbol.clone().into(),
            PropertyKey::Index(index) => {
                if let Ok(integer) = i32::try_from(*index) {
                    Value::integer(integer)
                } else {
                    Value::number(*index)
                }
            }
        }
    }
}

impl From<PropertyKey> for Value {
    #[inline]
    fn from(property_key: PropertyKey) -> Value {
        match property_key {
            PropertyKey::String(ref string) => string.as_str().into(),
            PropertyKey::Symbol(ref symbol) => symbol.clone().into(),
            PropertyKey::Index(index) => {
                if let Ok(integer) = i32::try_from(index) {
                    Value::integer(integer)
                } else {
                    Value::number(index)
                }
            }
        }
    }
}

impl From<u8> for PropertyKey {
    fn from(value: u8) -> Self {
        PropertyKey::Index(value.into())
    }
}

impl From<u16> for PropertyKey {
    fn from(value: u16) -> Self {
        PropertyKey::Index(value.into())
    }
}

impl From<u32> for PropertyKey {
    fn from(value: u32) -> Self {
        PropertyKey::Index(value)
    }
}

impl From<usize> for PropertyKey {
    fn from(value: usize) -> Self {
        if let Ok(index) = u32::try_from(value) {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(Name::new(value.to_string()))
        }
    }
}

impl From<isize> for PropertyKey {
    fn from(value: isize) -> Self {
        if let Ok(index) = u32::try_from(value) {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(Name::new(value.to_string()))
        }
    }
}

impl From<i32> for PropertyKey {
    fn from(value: i32) -> Self {
        if let Ok(index) = u32::try_from(value) {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(Name::new(value.to_string()))
        }
    }
}

impl From<f64> for PropertyKey {
    fn from(value: f64) -> Self {
        use num_traits::cast::FromPrimitive;
        if let Some(index) = u32::from_f64(value) {
            return PropertyKey::Index(index);
        }

        PropertyKey::String(Name::new(ryu_js::Buffer::new().format(value)))
    }
}

impl PartialEq<&str> for PropertyKey {
    fn eq(&self, other: &&str) -> bool {
        match self {
            PropertyKey::String(ref string) => string.as_str() == *other,
            _ => false,
        }
    }
}
