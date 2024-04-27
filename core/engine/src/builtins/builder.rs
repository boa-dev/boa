use boa_macros::js_str;

use crate::{
    js_string,
    native_function::{NativeFunctionObject, NativeFunctionPointer},
    object::{
        shape::{property_table::PropertyTableInner, slot::SlotAttributes},
        FunctionBinding, JsFunction, JsPrototype, CONSTRUCTOR, PROTOTYPE,
    },
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    string::StaticJsStrings,
    JsObject, JsString, JsValue, NativeFunction,
};

use super::{function::ConstructorKind, BuiltInConstructor, IntrinsicObject};

/// Marker for a constructor function.
pub(crate) struct Constructor {
    prototype: JsObject,
    inherits: JsPrototype,
    attributes: Attribute,
}

/// Marker for a constructor function without a custom prototype for its instances.
pub(crate) struct ConstructorNoProto;

/// Marker for an ordinary function.
pub(crate) struct OrdinaryFunction;

/// Indicates if the marker is a constructor.
pub(crate) trait IsConstructor {
    const IS_CONSTRUCTOR: bool;
}

impl IsConstructor for Constructor {
    const IS_CONSTRUCTOR: bool = true;
}

impl IsConstructor for ConstructorNoProto {
    const IS_CONSTRUCTOR: bool = true;
}

impl IsConstructor for OrdinaryFunction {
    const IS_CONSTRUCTOR: bool = false;
}

/// Marker for a callable object.
pub(crate) struct Callable<Kind> {
    function: NativeFunctionPointer,
    name: JsString,
    length: usize,
    kind: Kind,
    realm: Realm,
}

/// Marker for an ordinary object.
pub(crate) struct OrdinaryObject;

/// Applies the pending builder data to the object.
pub(crate) trait ApplyToObject {
    fn apply_to(self, object: &JsObject);
}

impl ApplyToObject for Constructor {
    fn apply_to(self, object: &JsObject) {
        object.insert(
            PROTOTYPE,
            PropertyDescriptor::builder()
                .value(self.prototype.clone())
                .writable(false)
                .enumerable(false)
                .configurable(false),
        );

        {
            let mut prototype = self.prototype.borrow_mut();
            prototype.set_prototype(self.inherits);
            prototype.insert(
                CONSTRUCTOR,
                PropertyDescriptor::builder()
                    .value(object.clone())
                    .writable(self.attributes.writable())
                    .enumerable(self.attributes.enumerable())
                    .configurable(self.attributes.configurable()),
            );
        }
    }
}

impl ApplyToObject for ConstructorNoProto {
    fn apply_to(self, _: &JsObject) {}
}

impl ApplyToObject for OrdinaryFunction {
    fn apply_to(self, _: &JsObject) {}
}

impl<S: ApplyToObject + IsConstructor> ApplyToObject for Callable<S> {
    fn apply_to(self, object: &JsObject) {
        {
            let mut function = object
                .downcast_mut::<NativeFunctionObject>()
                .expect("Builtin must be a function object");
            function.f = NativeFunction::from_fn_ptr(self.function);
            function.constructor = S::IS_CONSTRUCTOR.then_some(ConstructorKind::Base);
            function.realm = Some(self.realm);
        }
        object.insert(
            StaticJsStrings::LENGTH,
            PropertyDescriptor::builder()
                .value(self.length)
                .writable(false)
                .enumerable(false)
                .configurable(true),
        );
        object.insert(
            js_str!("name"),
            PropertyDescriptor::builder()
                .value(self.name)
                .writable(false)
                .enumerable(false)
                .configurable(true),
        );

        self.kind.apply_to(object);
    }
}

impl ApplyToObject for OrdinaryObject {
    fn apply_to(self, _: &JsObject) {}
}

/// Builder for creating built-in objects, like `Array`.
///
/// The marker `ObjectType` restricts the methods that can be called depending on the
/// type of object that is being constructed.
#[derive(Debug)]
#[must_use = "You need to call the `build` method in order for this to correctly assign the inner data"]
pub(crate) struct BuiltInBuilder<'ctx, Kind> {
    realm: &'ctx Realm,
    object: JsObject,
    kind: Kind,
    prototype: JsObject,
}

impl<'ctx> BuiltInBuilder<'ctx, OrdinaryObject> {
    pub(crate) fn with_intrinsic<I: IntrinsicObject>(
        realm: &'ctx Realm,
    ) -> BuiltInBuilder<'ctx, OrdinaryObject> {
        BuiltInBuilder {
            realm,
            object: I::get(realm.intrinsics()),
            kind: OrdinaryObject,
            prototype: realm.intrinsics().constructors().object().prototype(),
        }
    }
}

pub(crate) struct BuiltInConstructorWithPrototype<'ctx> {
    realm: &'ctx Realm,
    function: NativeFunctionPointer,
    name: JsString,
    length: usize,

    object_property_table: PropertyTableInner,
    object_storage: Vec<JsValue>,
    object: JsObject,

    prototype_property_table: PropertyTableInner,
    prototype_storage: Vec<JsValue>,
    prototype: JsObject,
    __proto__: JsPrototype,
    inherits: Option<JsObject>,
    attributes: Attribute,
}

#[allow(dead_code)]
impl BuiltInConstructorWithPrototype<'_> {
    /// Specify how many arguments the constructor function takes.
    ///
    /// Default is `0`.
    #[inline]
    pub(crate) const fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    /// Specify the name of the constructor function.
    ///
    /// Default is `""`
    pub(crate) fn name(mut self, name: JsString) -> Self {
        self.name = name;
        self
    }

    /// Adds a new static method to the builtin object.
    pub(crate) fn static_method<B>(
        mut self,
        function: NativeFunctionPointer,
        binding: B,
        length: usize,
    ) -> Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = BuiltInBuilder::callable(self.realm, function)
            .name(binding.name)
            .length(length)
            .build();

        debug_assert!(self
            .object_property_table
            .map
            .get(&binding.binding)
            .is_none());
        self.object_property_table.insert(
            binding.binding,
            SlotAttributes::WRITABLE | SlotAttributes::CONFIGURABLE,
        );
        self.object_storage.push(function.into());
        self
    }

    /// Adds a new static data property to the builtin object.
    pub(crate) fn static_property<K, V>(mut self, key: K, value: V, attribute: Attribute) -> Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let key = key.into();

        debug_assert!(self.object_property_table.map.get(&key).is_none());
        self.object_property_table
            .insert(key, SlotAttributes::from_bits_truncate(attribute.bits()));
        self.object_storage.push(value.into());
        self
    }

    /// Adds a new static accessor property to the builtin object.
    pub(crate) fn static_accessor<K>(
        mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> Self
    where
        K: Into<PropertyKey>,
    {
        let mut attributes = SlotAttributes::from_bits_truncate(attribute.bits());
        debug_assert!(!attributes.contains(SlotAttributes::WRITABLE));
        attributes.set(SlotAttributes::GET, get.is_some());
        attributes.set(SlotAttributes::SET, set.is_some());

        let key = key.into();

        debug_assert!(self.object_property_table.map.get(&key).is_none());
        self.object_property_table.insert(key, attributes);
        self.object_storage.extend([
            get.map(JsValue::new).unwrap_or_default(),
            set.map(JsValue::new).unwrap_or_default(),
        ]);
        self
    }

    /// Specify the `[[Prototype]]` internal field of the builtin object.
    ///
    /// Default is `Function.prototype` for constructors and `Object.prototype` for statics.
    pub(crate) fn prototype(mut self, prototype: JsObject) -> Self {
        self.__proto__ = Some(prototype);
        self
    }

    /// Adds a new method to the constructor's prototype.
    pub(crate) fn method<B>(
        mut self,
        function: NativeFunctionPointer,
        binding: B,
        length: usize,
    ) -> Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = BuiltInBuilder::callable(self.realm, function)
            .name(binding.name)
            .length(length)
            .build();

        debug_assert!(self
            .prototype_property_table
            .map
            .get(&binding.binding)
            .is_none());
        self.prototype_property_table.insert(
            binding.binding,
            SlotAttributes::WRITABLE | SlotAttributes::CONFIGURABLE,
        );
        self.prototype_storage.push(function.into());
        self
    }

    /// Adds a new data property to the constructor's prototype.
    pub(crate) fn property<K, V>(mut self, key: K, value: V, attribute: Attribute) -> Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let key = key.into();

        debug_assert!(self.prototype_property_table.map.get(&key).is_none());
        self.prototype_property_table
            .insert(key, SlotAttributes::from_bits_truncate(attribute.bits()));
        self.prototype_storage.push(value.into());
        self
    }

    /// Adds new accessor property to the constructor's prototype.
    pub(crate) fn accessor<K>(
        mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> Self
    where
        K: Into<PropertyKey>,
    {
        let mut attributes = SlotAttributes::from_bits_truncate(attribute.bits());
        debug_assert!(!attributes.contains(SlotAttributes::WRITABLE));
        attributes.set(SlotAttributes::GET, get.is_some());
        attributes.set(SlotAttributes::SET, set.is_some());

        let key = key.into();

        debug_assert!(self.prototype_property_table.map.get(&key).is_none());
        self.prototype_property_table.insert(key, attributes);
        self.prototype_storage.extend([
            get.map(JsValue::new).unwrap_or_default(),
            set.map(JsValue::new).unwrap_or_default(),
        ]);
        self
    }

    /// Specifies the parent prototype which objects created by this constructor inherit from.
    ///
    /// Default is `Object.prototype`.
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn inherits(mut self, prototype: JsPrototype) -> Self {
        self.inherits = prototype;
        self
    }

    /// Specifies the property attributes of the prototype's "constructor" property.
    pub(crate) const fn constructor_attributes(mut self, attributes: Attribute) -> Self {
        self.attributes = attributes;
        self
    }

    pub(crate) fn build(mut self) {
        let length = self.length;
        let name = self.name.clone();
        let prototype = self.prototype.clone();
        self = self.static_property(js_str!("length"), length, Attribute::CONFIGURABLE);
        self = self.static_property(js_str!("name"), name, Attribute::CONFIGURABLE);
        self = self.static_property(PROTOTYPE, prototype, Attribute::empty());

        let attributes = self.attributes;
        let object = self.object.clone();
        self = self.property(CONSTRUCTOR, object, attributes);

        {
            let mut prototype = self.prototype.borrow_mut();
            prototype
                .properties_mut()
                .shape
                .as_unique()
                .expect("The object should have a unique shape")
                .override_internal(self.prototype_property_table, self.inherits);

            let prototype_old_storage = std::mem::replace(
                &mut prototype.properties_mut().storage,
                self.prototype_storage,
            );

            debug_assert_eq!(prototype_old_storage.len(), 0);
        }

        let mut object = self.object.borrow_mut();
        let function = object
            .downcast_mut::<NativeFunctionObject>()
            .expect("Builtin must be a function object");
        function.f = NativeFunction::from_fn_ptr(self.function);
        function.constructor = Some(ConstructorKind::Base);
        function.realm = Some(self.realm.clone());
        object
            .properties_mut()
            .shape
            .as_unique()
            .expect("The object should have a unique shape")
            .override_internal(self.object_property_table, self.__proto__);

        let object_old_storage =
            std::mem::replace(&mut object.properties_mut().storage, self.object_storage);

        debug_assert_eq!(object_old_storage.len(), 0);
    }

    pub(crate) fn build_without_prototype(mut self) {
        let length = self.length;
        let name = self.name.clone();
        self = self.static_property(js_str!("length"), length, Attribute::CONFIGURABLE);
        self = self.static_property(js_str!("name"), name, Attribute::CONFIGURABLE);

        let mut object = self.object.borrow_mut();
        let function = object
            .downcast_mut::<NativeFunctionObject>()
            .expect("Builtin must be a function object");
        function.f = NativeFunction::from_fn_ptr(self.function);
        function.constructor = Some(ConstructorKind::Base);
        function.realm = Some(self.realm.clone());
        object
            .properties_mut()
            .shape
            .as_unique()
            .expect("The object should have a unique shape")
            .override_internal(self.object_property_table, self.__proto__);

        let object_old_storage =
            std::mem::replace(&mut object.properties_mut().storage, self.object_storage);

        debug_assert_eq!(object_old_storage.len(), 0);
    }
}

pub(crate) struct BuiltInCallable<'ctx> {
    realm: &'ctx Realm,
    function: NativeFunctionPointer,
    name: JsString,
    length: usize,
}

impl BuiltInCallable<'_> {
    /// Specify how many arguments the constructor function takes.
    ///
    /// Default is `0`.
    #[inline]
    pub(crate) const fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    /// Specify the name of the constructor function.
    ///
    /// Default is `""`
    pub(crate) fn name(mut self, name: JsString) -> Self {
        self.name = name;
        self
    }

    pub(crate) fn build(self) -> JsFunction {
        let object = self.realm.intrinsics().templates().function().create(
            NativeFunctionObject {
                f: NativeFunction::from_fn_ptr(self.function),
                constructor: None,
                realm: Some(self.realm.clone()),
            },
            vec![JsValue::new(self.length), JsValue::new(self.name)],
        );

        JsFunction::from_object_unchecked(object)
    }
}

impl<'ctx> BuiltInBuilder<'ctx, OrdinaryObject> {
    pub(crate) fn callable(
        realm: &'ctx Realm,
        function: NativeFunctionPointer,
    ) -> BuiltInCallable<'ctx> {
        BuiltInCallable {
            realm,
            function,
            length: 0,
            name: js_string!(),
        }
    }

    pub(crate) fn callable_with_intrinsic<I: IntrinsicObject>(
        realm: &'ctx Realm,
        function: NativeFunctionPointer,
    ) -> BuiltInBuilder<'ctx, Callable<OrdinaryFunction>> {
        BuiltInBuilder {
            realm,
            object: I::get(realm.intrinsics()),
            kind: Callable {
                function,
                name: js_string!(),
                length: 0,
                kind: OrdinaryFunction,
                realm: realm.clone(),
            },
            prototype: realm.intrinsics().constructors().function().prototype(),
        }
    }

    pub(crate) fn callable_with_object(
        realm: &'ctx Realm,
        object: JsObject,
        function: NativeFunctionPointer,
    ) -> BuiltInBuilder<'ctx, Callable<OrdinaryFunction>> {
        BuiltInBuilder {
            realm,
            object,
            kind: Callable {
                function,
                name: js_string!(),
                length: 0,
                kind: OrdinaryFunction,
                realm: realm.clone(),
            },
            prototype: realm.intrinsics().constructors().function().prototype(),
        }
    }
}

impl<'ctx> BuiltInBuilder<'ctx, Callable<Constructor>> {
    pub(crate) fn from_standard_constructor<SC: BuiltInConstructor>(
        realm: &'ctx Realm,
    ) -> BuiltInConstructorWithPrototype<'ctx> {
        let constructor = SC::STANDARD_CONSTRUCTOR(realm.intrinsics().constructors());
        BuiltInConstructorWithPrototype {
            realm,
            function: SC::constructor,
            name: js_string!(SC::NAME),
            length: SC::LENGTH,
            object_property_table: PropertyTableInner::default(),
            object_storage: Vec::default(),
            object: constructor.constructor(),
            prototype_property_table: PropertyTableInner::default(),
            prototype_storage: Vec::default(),
            prototype: constructor.prototype(),
            __proto__: Some(realm.intrinsics().constructors().function().prototype()),
            inherits: Some(realm.intrinsics().constructors().object().prototype()),
            attributes: Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        }
    }
}

impl<T> BuiltInBuilder<'_, T> {
    /// Adds a new static method to the builtin object.
    pub(crate) fn static_method<B>(
        self,
        function: NativeFunctionPointer,
        binding: B,
        length: usize,
    ) -> Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = BuiltInBuilder::callable(self.realm, function)
            .name(binding.name)
            .length(length)
            .build();

        self.object.insert(
            binding.binding,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        self
    }

    /// Adds a new static data property to the builtin object.
    pub(crate) fn static_property<K, V>(self, key: K, value: V, attribute: Attribute) -> Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.object.insert(key, property);
        self
    }

    /// Specify the `[[Prototype]]` internal field of the builtin object.
    ///
    /// Default is `Function.prototype` for constructors and `Object.prototype` for statics.
    pub(crate) fn prototype(mut self, prototype: JsObject) -> Self {
        self.prototype = prototype;
        self
    }
}

impl<FnTyp> BuiltInBuilder<'_, Callable<FnTyp>> {
    /// Specify how many arguments the constructor function takes.
    ///
    /// Default is `0`.
    #[inline]
    pub(crate) const fn length(mut self, length: usize) -> Self {
        self.kind.length = length;
        self
    }

    /// Specify the name of the constructor function.
    ///
    /// Default is `""`
    pub(crate) fn name(mut self, name: JsString) -> Self {
        self.kind.name = name;
        self
    }
}

impl BuiltInBuilder<'_, OrdinaryObject> {
    /// Build the builtin object.
    pub(crate) fn build(self) -> JsObject {
        self.kind.apply_to(&self.object);

        self.object.set_prototype(Some(self.prototype));

        self.object
    }
}

impl<FnTyp: ApplyToObject + IsConstructor> BuiltInBuilder<'_, Callable<FnTyp>> {
    /// Build the builtin callable.
    pub(crate) fn build(self) -> JsFunction {
        self.kind.apply_to(&self.object);

        self.object.set_prototype(Some(self.prototype));

        JsFunction::from_object_unchecked(self.object)
    }
}
