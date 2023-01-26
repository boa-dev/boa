use crate::{
    builtins::{function::ClassFieldDefinition, Array},
    context::intrinsics::{StandardConstructor, StandardConstructors},
    error::JsNativeError,
    object::{JsObject, PrivateElement},
    property::{PropertyDescriptor, PropertyDescriptorBuilder, PropertyKey, PropertyNameKind},
    value::Type,
    Context, JsResult, JsSymbol, JsValue,
};
use boa_ast::function::PrivateName;

/// Object integrity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityLevel {
    /// Sealed object integrity level.
    ///
    /// Preventing new properties from being added to it and marking all existing
    /// properties as non-configurable. Values of present properties can still be
    /// changed as long as they are writable.
    Sealed,

    /// Frozen object integrity level
    ///
    /// A frozen object can no longer be changed; freezing an object prevents new
    /// properties from being added to it, existing properties from being removed,
    /// prevents changing the enumerability, configurability, or writability of
    /// existing properties, and prevents the values of existing properties from
    /// being changed. In addition, freezing an object also prevents its prototype
    /// from being changed.
    Frozen,
}

impl IntegrityLevel {
    /// Returns `true` if the integrity level is sealed.
    #[must_use]
    pub const fn is_sealed(&self) -> bool {
        matches!(self, Self::Sealed)
    }

    /// Returns `true` if the integrity level is frozen.
    #[must_use]
    pub const fn is_frozen(&self) -> bool {
        matches!(self, Self::Frozen)
    }
}

impl JsObject {
    /// Check if object is extensible.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isextensible-o
    #[inline]
    pub fn is_extensible(&self, context: &mut Context<'_>) -> JsResult<bool> {
        // 1. Return ? O.[[IsExtensible]]().
        self.__is_extensible__(context)
    }

    /// Get property from object or throw.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-o-p
    pub fn get<K>(&self, key: K, context: &mut Context<'_>) -> JsResult<JsValue>
    where
        K: Into<PropertyKey>,
    {
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Return ? O.[[Get]](P, O).
        self.__get__(&key.into(), self.clone().into(), context)
    }

    /// set property of object or throw if bool flag is passed.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set-o-p-v-throw
    pub fn set<K, V>(
        &self,
        key: K,
        value: V,
        throw: bool,
        context: &mut Context<'_>,
    ) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Assert: Type(Throw) is Boolean.
        // 4. Let success be ? O.[[Set]](P, V, O).
        let success = self.__set__(key.clone(), value.into(), self.clone().into(), context)?;
        // 5. If success is false and Throw is true, throw a TypeError exception.
        if !success && throw {
            return Err(JsNativeError::typ()
                .with_message(format!("cannot set non-writable property: {key}"))
                .into());
        }
        // 6. Return success.
        Ok(success)
    }

    /// Create data property
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createdataproperty
    pub fn create_data_property<K, V>(
        &self,
        key: K,
        value: V,
        context: &mut Context<'_>,
    ) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let newDesc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true }.
        let new_desc = PropertyDescriptor::builder()
            .value(value)
            .writable(true)
            .enumerable(true)
            .configurable(true);
        // 4. Return ? O.[[DefineOwnProperty]](P, newDesc).
        self.__define_own_property__(key.into(), new_desc.into(), context)
    }

    // todo: CreateMethodProperty

    /// Create data property or throw
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createdatapropertyorthrow
    pub fn create_data_property_or_throw<K, V>(
        &self,
        key: K,
        value: V,
        context: &mut Context<'_>,
    ) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let success be ? CreateDataProperty(O, P, V).
        let success = self.create_data_property(key.clone(), value, context)?;
        // 4. If success is false, throw a TypeError exception.
        if !success {
            return Err(JsNativeError::typ()
                .with_message(format!("cannot redefine property: {key}"))
                .into());
        }
        // 5. Return success.
        Ok(success)
    }

    /// Create non-enumerable data property or throw
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createnonenumerabledatapropertyinfallibly
    pub(crate) fn create_non_enumerable_data_property_or_throw<K, V>(
        &self,
        key: K,
        value: V,
        context: &mut Context<'_>,
    ) where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        // 1. Assert: O is an ordinary, extensible object with no non-configurable properties.

        // 2. Let newDesc be the PropertyDescriptor {
        //    [[Value]]: V,
        //    [[Writable]]: true,
        //    [[Enumerable]]: false,
        //    [[Configurable]]: true
        //  }.
        let new_desc = PropertyDescriptorBuilder::new()
            .value(value)
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build();

        // 3. Perform ! DefinePropertyOrThrow(O, P, newDesc).
        self.define_property_or_throw(key, new_desc, context)
            .expect("should not fail according to spec");

        // 4. Return unused.
    }

    /// Define property or throw.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-definepropertyorthrow
    pub fn define_property_or_throw<K, P>(
        &self,
        key: K,
        desc: P,
        context: &mut Context<'_>,
    ) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let success be ? O.[[DefineOwnProperty]](P, desc).
        let success = self.__define_own_property__(key.clone(), desc.into(), context)?;
        // 4. If success is false, throw a TypeError exception.
        if !success {
            return Err(JsNativeError::typ()
                .with_message(format!("cannot redefine property: {key}"))
                .into());
        }
        // 5. Return success.
        Ok(success)
    }

    /// Defines the property or throws a `TypeError` if the operation fails.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-deletepropertyorthrow
    pub fn delete_property_or_throw<K>(&self, key: K, context: &mut Context<'_>) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let success be ? O.[[Delete]](P).
        let success = self.__delete__(&key, context)?;
        // 4. If success is false, throw a TypeError exception.
        if !success {
            return Err(JsNativeError::typ()
                .with_message(format!("cannot delete property: {key}"))
                .into());
        }
        // 5. Return success.
        Ok(success)
    }

    /// Check if object has property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasproperty
    pub fn has_property<K>(&self, key: K, context: &mut Context<'_>) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
    {
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Return ? O.[[HasProperty]](P).
        self.__has_property__(&key.into(), context)
    }

    /// Check if object has an own property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasownproperty
    pub fn has_own_property<K>(&self, key: K, context: &mut Context<'_>) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let desc be ? O.[[GetOwnProperty]](P).
        let desc = self.__get_own_property__(&key, context)?;
        // 4. If desc is undefined, return false.
        // 5. Return true.
        Ok(desc.is_some())
    }

    /// Call this object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
    // <https://tc39.es/ecma262/#sec-ecmascript-function-objects-call-thisargument-argumentslist>
    #[track_caller]
    #[inline]
    pub fn call(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If argumentsList is not present, set argumentsList to a new empty List.
        // 2. If IsCallable(F) is false, throw a TypeError exception.
        if !self.is_callable() {
            return Err(JsNativeError::typ().with_message("not a function").into());
        }
        // 3. Return ? F.[[Call]](V, argumentsList).
        self.__call__(this, args, context)
    }

    /// `Construct ( F [ , argumentsList [ , newTarget ] ] )`
    ///
    /// Construct an instance of this object with the specified arguments.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-construct
    #[track_caller]
    #[inline]
    pub fn construct(
        &self,
        args: &[JsValue],
        new_target: Option<&Self>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        // 1. If newTarget is not present, set newTarget to F.
        let new_target = new_target.unwrap_or(self);
        // 2. If argumentsList is not present, set argumentsList to a new empty List.
        // 3. Return ? F.[[Construct]](argumentsList, newTarget).
        self.__construct__(args, new_target, context)
    }

    /// Make the object [`sealed`][IntegrityLevel::Sealed] or [`frozen`][IntegrityLevel::Frozen].
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-setintegritylevel
    pub fn set_integrity_level(
        &self,
        level: IntegrityLevel,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        // 1. Assert: Type(O) is Object.
        // 2. Assert: level is either sealed or frozen.

        // 3. Let status be ? O.[[PreventExtensions]]().
        let status = self.__prevent_extensions__(context)?;
        // 4. If status is false, return false.
        if !status {
            return Ok(false);
        }

        // 5. Let keys be ? O.[[OwnPropertyKeys]]().
        let keys = self.__own_property_keys__(context)?;

        match level {
            // 6. If level is sealed, then
            IntegrityLevel::Sealed => {
                // a. For each element k of keys, do
                for k in keys {
                    // i. Perform ? DefinePropertyOrThrow(O, k, PropertyDescriptor { [[Configurable]]: false }).
                    self.define_property_or_throw(
                        k,
                        PropertyDescriptor::builder().configurable(false).build(),
                        context,
                    )?;
                }
            }
            // 7. Else,
            //     a. Assert: level is frozen.
            IntegrityLevel::Frozen => {
                // b. For each element k of keys, do
                for k in keys {
                    // i. Let currentDesc be ? O.[[GetOwnProperty]](k).
                    let current_desc = self.__get_own_property__(&k, context)?;
                    // ii. If currentDesc is not undefined, then
                    if let Some(current_desc) = current_desc {
                        // 1. If IsAccessorDescriptor(currentDesc) is true, then
                        let desc = if current_desc.is_accessor_descriptor() {
                            // a. Let desc be the PropertyDescriptor { [[Configurable]]: false }.
                            PropertyDescriptor::builder().configurable(false).build()
                        // 2. Else,
                        } else {
                            // a. Let desc be the PropertyDescriptor { [[Configurable]]: false, [[Writable]]: false }.
                            PropertyDescriptor::builder()
                                .configurable(false)
                                .writable(false)
                                .build()
                        };
                        // 3. Perform ? DefinePropertyOrThrow(O, k, desc).
                        self.define_property_or_throw(k, desc, context)?;
                    }
                }
            }
        }

        // 8. Return true.
        Ok(true)
    }

    /// Check if the object is [`sealed`][IntegrityLevel::Sealed] or [`frozen`][IntegrityLevel::Frozen].
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-testintegritylevel
    pub fn test_integrity_level(
        &self,
        level: IntegrityLevel,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        // 1. Assert: Type(O) is Object.
        // 2. Assert: level is either sealed or frozen.

        // 3. Let extensible be ? IsExtensible(O).
        let extensible = self.is_extensible(context)?;

        // 4. If extensible is true, return false.
        if extensible {
            return Ok(false);
        }

        // 5. NOTE: If the object is extensible, none of its properties are examined.
        // 6. Let keys be ? O.[[OwnPropertyKeys]]().
        let keys = self.__own_property_keys__(context)?;

        // 7. For each element k of keys, do
        for k in keys {
            // a. Let currentDesc be ? O.[[GetOwnProperty]](k).
            let current_desc = self.__get_own_property__(&k, context)?;
            // b. If currentDesc is not undefined, then
            if let Some(current_desc) = current_desc {
                // i. If currentDesc.[[Configurable]] is true, return false.
                if current_desc.expect_configurable() {
                    return Ok(false);
                }
                // ii. If level is frozen and IsDataDescriptor(currentDesc) is true, then
                if level.is_frozen() && current_desc.is_data_descriptor() {
                    // 1. If currentDesc.[[Writable]] is true, return false.
                    if current_desc.expect_writable() {
                        return Ok(false);
                    }
                }
            }
        }
        // 8. Return true.
        Ok(true)
    }

    /// Abstract operation [`LengthOfArrayLike ( obj )`][spec].
    ///
    /// Returns the value of the "length" property of an array-like object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-lengthofarraylike
    pub(crate) fn length_of_array_like(&self, context: &mut Context<'_>) -> JsResult<u64> {
        // 1. Assert: Type(obj) is Object.
        // 2. Return ℝ(? ToLength(? Get(obj, "length"))).
        self.get("length", context)?.to_length(context)
    }

    /// `7.3.22 SpeciesConstructor ( O, defaultConstructor )`
    ///
    /// The abstract operation `SpeciesConstructor` takes arguments `O` (an Object) and
    /// `defaultConstructor` (a constructor). It is used to retrieve the constructor that should be
    /// used to create new objects that are derived from `O`. `defaultConstructor` is the
    /// constructor to use if a constructor `@@species` property cannot be found starting from `O`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-speciesconstructor
    pub(crate) fn species_constructor<F>(
        &self,
        default_constructor: F,
        context: &mut Context<'_>,
    ) -> JsResult<Self>
    where
        F: FnOnce(&StandardConstructors) -> &StandardConstructor,
    {
        // 1. Assert: Type(O) is Object.

        // 2. Let C be ? Get(O, "constructor").
        let c = self.get("constructor", context)?;

        // 3. If C is undefined, return defaultConstructor.
        if c.is_undefined() {
            return Ok(default_constructor(context.intrinsics().constructors()).constructor());
        }

        // 4. If Type(C) is not Object, throw a TypeError exception.
        let c = c.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("property 'constructor' is not an object")
        })?;

        // 5. Let S be ? Get(C, @@species).
        let s = c.get(JsSymbol::species(), context)?;

        // 6. If S is either undefined or null, return defaultConstructor.
        if s.is_null_or_undefined() {
            return Ok(default_constructor(context.intrinsics().constructors()).constructor());
        }

        // 7. If IsConstructor(S) is true, return S.
        if let Some(s) = s.as_constructor() {
            return Ok(s.clone());
        }

        // 8. Throw a TypeError exception.
        Err(JsNativeError::typ()
            .with_message("property 'constructor' is not a constructor")
            .into())
    }

    /// It is used to iterate over names of object's keys.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-enumerableownpropertynames
    pub(crate) fn enumerable_own_property_names(
        &self,
        kind: PropertyNameKind,
        context: &mut Context<'_>,
    ) -> JsResult<Vec<JsValue>> {
        // 1. Assert: Type(O) is Object.
        // 2. Let ownKeys be ? O.[[OwnPropertyKeys]]().
        let own_keys = self.__own_property_keys__(context)?;
        // 3. Let properties be a new empty List.
        let mut properties = vec![];

        // 4. For each element key of ownKeys, do
        for key in own_keys {
            // a. If Type(key) is String, then
            let key_str = match &key {
                PropertyKey::String(s) => Some(s.clone()),
                PropertyKey::Index(i) => Some(i.to_string().into()),
                PropertyKey::Symbol(_) => None,
            };

            if let Some(key_str) = key_str {
                // i. Let desc be ? O.[[GetOwnProperty]](key).
                let desc = self.__get_own_property__(&key, context)?;
                // ii. If desc is not undefined and desc.[[Enumerable]] is true, then
                if let Some(desc) = desc {
                    if desc.expect_enumerable() {
                        match kind {
                            // 1. If kind is key, append key to properties.
                            PropertyNameKind::Key => properties.push(key_str.into()),
                            // 2. Else,
                            // a. Let value be ? Get(O, key).
                            // b. If kind is value, append value to properties.
                            PropertyNameKind::Value => {
                                properties.push(self.get(key.clone(), context)?);
                            }
                            // c. Else,
                            // i. Assert: kind is key+value.
                            // ii. Let entry be ! CreateArrayFromList(« key, value »).
                            // iii. Append entry to properties.
                            PropertyNameKind::KeyAndValue => properties.push(
                                Array::create_array_from_list(
                                    [key_str.into(), self.get(key.clone(), context)?],
                                    context,
                                )
                                .into(),
                            ),
                        }
                    }
                }
            }
        }

        // 5. Return properties.
        Ok(properties)
    }

    /// Abstract operation `GetMethod ( V, P )`
    ///
    /// Retrieves the value of a specific property, when the value of the property is expected to be a function.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getmethod
    pub(crate) fn get_method<K>(&self, key: K, context: &mut Context<'_>) -> JsResult<Option<Self>>
    where
        K: Into<PropertyKey>,
    {
        // Note: The spec specifies this function for JsValue.
        // It is implemented for JsObject for convenience.

        // 1. Assert: IsPropertyKey(P) is true.
        // 2. Let func be ? GetV(V, P).
        match &self.__get__(&key.into(), self.clone().into(), context)? {
            // 3. If func is either undefined or null, return undefined.
            JsValue::Undefined | JsValue::Null => Ok(None),
            // 5. Return func.
            JsValue::Object(obj) if obj.is_callable() => Ok(Some(obj.clone())),
            // 4. If IsCallable(func) is false, throw a TypeError exception.
            _ => Err(JsNativeError::typ()
                .with_message("value returned for property of object is not a function")
                .into()),
        }
    }

    /// Abstract operation `IsArray ( argument )`
    ///
    /// Check if a value is an array.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isarray
    pub(crate) fn is_array_abstract(&self) -> JsResult<bool> {
        // Note: The spec specifies this function for JsValue.
        // It is implemented for JsObject for convenience.

        // 2. If argument is an Array exotic object, return true.
        if self.is_array() {
            return Ok(true);
        }

        // 3. If argument is a Proxy exotic object, then
        let object = self.borrow();
        if let Some(proxy) = object.as_proxy() {
            // a. If argument.[[ProxyHandler]] is null, throw a TypeError exception.
            // b. Let target be argument.[[ProxyTarget]].
            let (target, _) = proxy.try_data()?;

            // c. Return ? IsArray(target).
            return target.is_array_abstract();
        }

        // 4. Return false.
        Ok(false)
    }

    // todo: GetFunctionRealm

    // todo: CopyDataProperties

    /// Abstract operation `PrivateElementFind ( O, P )`
    ///
    /// Get the private element from an object.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-privateelementfind
    #[allow(clippy::similar_names)]
    pub(crate) fn private_element_find(
        &self,
        name: &PrivateName,
        is_getter: bool,
        is_setter: bool,
    ) -> Option<PrivateElement> {
        // 1. If O.[[PrivateElements]] contains a PrivateElement whose [[Key]] is P, then
        for (key, value) in &self.borrow().private_elements {
            if key == name {
                // a. Let entry be that PrivateElement.
                // b. Return entry.
                if let PrivateElement::Accessor { getter, setter } = value {
                    if getter.is_some() && is_getter || setter.is_some() && is_setter {
                        return Some(value.clone());
                    }
                } else {
                    return Some(value.clone());
                }
            }
        }

        // 2. Return empty.
        None
    }

    /// Abstract operation `PrivateFieldAdd ( O, P, value )`
    ///
    /// Add private field to an object.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-privatefieldadd
    pub(crate) fn private_field_add(
        &self,
        name: &PrivateName,
        value: JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        // 1. If the host is a web browser, then
        // a. Perform ? HostEnsureCanAddPrivateElement(O).
        context
            .host_hooks()
            .ensure_can_add_private_element(self, context)?;

        // 2. Let entry be PrivateElementFind(O, P).
        let entry = self.private_element_find(name, false, false);

        // 3. If entry is not empty, throw a TypeError exception.
        if entry.is_some() {
            return Err(JsNativeError::typ()
                .with_message("Private field already exists on prototype")
                .into());
        }

        // 4. Append PrivateElement { [[Key]]: P, [[Kind]]: field, [[Value]]: value } to O.[[PrivateElements]].
        self.borrow_mut()
            .private_elements
            .push((*name, PrivateElement::Field(value)));

        // 5. Return unused.
        Ok(())
    }

    /// Abstract operation `PrivateMethodOrAccessorAdd ( O, method )`
    ///
    /// Add private method or accessor to an object.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-privatemethodoraccessoradd
    pub(crate) fn private_method_or_accessor_add(
        &self,
        name: &PrivateName,
        method: &PrivateElement,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        // 1. Assert: method.[[Kind]] is either method or accessor.
        assert!(matches!(
            method,
            PrivateElement::Method(_) | PrivateElement::Accessor { .. }
        ));
        let (getter, setter) = if let PrivateElement::Accessor { getter, setter } = method {
            (getter.is_some(), setter.is_some())
        } else {
            (false, false)
        };

        // 2. If the host is a web browser, then
        // a. Perform ? HostEnsureCanAddPrivateElement(O).
        context
            .host_hooks()
            .ensure_can_add_private_element(self, context)?;

        // 3. Let entry be PrivateElementFind(O, method.[[Key]]).
        let entry = self.private_element_find(name, getter, setter);

        // 4. If entry is not empty, throw a TypeError exception.
        if entry.is_some() {
            return Err(JsNativeError::typ()
                .with_message("Private method already exists on prototype")
                .into());
        }

        // 5. Append method to O.[[PrivateElements]].
        self.borrow_mut()
            .append_private_element(*name, method.clone());

        // 6. Return unused.
        Ok(())
    }

    /// Abstract operation `PrivateGet ( O, P )`
    ///
    /// Get the value of a private element.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-privateget
    pub(crate) fn private_get(
        &self,
        name: &PrivateName,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let entry be PrivateElementFind(O, P).
        let entry = self.private_element_find(name, true, true);

        match &entry {
            // 2. If entry is empty, throw a TypeError exception.
            None => Err(JsNativeError::typ()
                .with_message("Private element does not exist on object")
                .into()),

            // 3. If entry.[[Kind]] is field or method, then
            // a. Return entry.[[Value]].
            Some(PrivateElement::Field(value)) => Ok(value.clone()),
            Some(PrivateElement::Method(value)) => Ok(value.clone().into()),

            // 4. Assert: entry.[[Kind]] is accessor.
            Some(PrivateElement::Accessor { getter, .. }) => {
                // 5. If entry.[[Get]] is undefined, throw a TypeError exception.
                // 6. Let getter be entry.[[Get]].
                let getter = getter.as_ref().ok_or_else(|| {
                    JsNativeError::typ()
                        .with_message("private property was defined without a getter")
                })?;

                // 7. Return ? Call(getter, O).
                getter.call(&self.clone().into(), &[], context)
            }
        }
    }

    /// Abstract operation `PrivateSet ( O, P, value )`
    ///
    /// Set the value of a private element.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-privateset
    pub(crate) fn private_set(
        &self,
        name: &PrivateName,
        value: JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        // 1. Let entry be PrivateElementFind(O, P).
        // Note: This function is inlined here for mutable access.
        let mut object_mut = self.borrow_mut();
        let entry = object_mut
            .private_elements
            .iter_mut()
            .find_map(|(key, value)| if key == name { Some(value) } else { None });

        match entry {
            // 2. If entry is empty, throw a TypeError exception.
            None => {
                return Err(JsNativeError::typ()
                    .with_message("Private element does not exist on object")
                    .into())
            }

            // 3. If entry.[[Kind]] is field, then
            // a. Set entry.[[Value]] to value.
            Some(PrivateElement::Field(field)) => {
                *field = value;
            }

            // 4. Else if entry.[[Kind]] is method, then
            // a. Throw a TypeError exception.
            Some(PrivateElement::Method(_)) => {
                return Err(JsNativeError::typ()
                    .with_message("private method is not writable")
                    .into())
            }

            // 5. Else,
            Some(PrivateElement::Accessor { setter, .. }) => {
                // a. Assert: entry.[[Kind]] is accessor.
                // b. If entry.[[Set]] is undefined, throw a TypeError exception.
                // c. Let setter be entry.[[Set]].
                let setter = setter.clone().ok_or_else(|| {
                    JsNativeError::typ()
                        .with_message("private property was defined without a setter")
                })?;

                // d. Perform ? Call(setter, O, « value »).
                drop(object_mut);
                setter.call(&self.clone().into(), &[value], context)?;
            }
        }

        // 6. Return unused.
        Ok(())
    }

    /// Abstract operation `DefineField ( receiver, fieldRecord )`
    ///
    /// Define a field on an object.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-definefield
    pub(crate) fn define_field(
        &self,
        field_record: &ClassFieldDefinition,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        // 2. Let initializer be fieldRecord.[[Initializer]].
        let initializer = match field_record {
            ClassFieldDefinition::Public(_, function)
            | ClassFieldDefinition::Private(_, function) => function,
        };

        // 3. If initializer is not empty, then
        // a. Let initValue be ? Call(initializer, receiver).
        // 4. Else, let initValue be undefined.
        let init_value = initializer.call(&self.clone().into(), &[], context)?;

        match field_record {
            // 1. Let fieldName be fieldRecord.[[Name]].
            // 5. If fieldName is a Private Name, then
            ClassFieldDefinition::Private(field_name, _) => {
                // a. Perform ? PrivateFieldAdd(receiver, fieldName, initValue).
                self.private_field_add(field_name, init_value, context)?;
            }
            // 1. Let fieldName be fieldRecord.[[Name]].
            // 6. Else,
            ClassFieldDefinition::Public(field_name, _) => {
                // a. Assert: IsPropertyKey(fieldName) is true.
                // b. Perform ? CreateDataPropertyOrThrow(receiver, fieldName, initValue).
                self.create_data_property_or_throw(field_name.clone(), init_value, context)?;
            }
        }

        // 7. Return unused.
        Ok(())
    }

    /// Abstract operation `InitializeInstanceElements ( O, constructor )`
    ///
    /// Add private methods and fields from a class constructor to an object.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-initializeinstanceelements
    pub(crate) fn initialize_instance_elements(
        &self,
        constructor: &Self,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        let constructor_borrow = constructor.borrow();
        let constructor_function = constructor_borrow
            .as_function()
            .expect("class constructor must be function object");

        // 1. Let methods be the value of constructor.[[PrivateMethods]].
        // 2. For each PrivateElement method of methods, do
        for (name, method) in constructor_function.get_private_methods() {
            // a. Perform ? PrivateMethodOrAccessorAdd(O, method).
            self.private_method_or_accessor_add(name, method, context)?;
        }

        // 3. Let fields be the value of constructor.[[Fields]].
        // 4. For each element fieldRecord of fields, do
        for field_record in constructor_function.get_fields() {
            // a. Perform ? DefineField(O, fieldRecord).
            self.define_field(field_record, context)?;
        }

        // 5. Return unused.
        Ok(())
    }

    /// Abstract operation `Invoke ( V, P [ , argumentsList ] )`
    ///
    /// Calls a method property of an ECMAScript object.
    ///
    /// Equivalent to the [`JsValue::invoke`] method, but specialized for objects.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-invoke
    pub(crate) fn invoke<K>(
        &self,
        key: K,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>
    where
        K: Into<PropertyKey>,
    {
        let this_value: JsValue = self.clone().into();

        // 1. If argumentsList is not present, set argumentsList to a new empty List.
        // 2. Let func be ? GetV(V, P).
        let func = self.__get__(&key.into(), this_value.clone(), context)?;

        // 3. Return ? Call(func, V, argumentsList)
        func.call(&this_value, args, context)
    }
}

impl JsValue {
    /// Abstract operation `GetV ( V, P )`.
    ///
    /// Retrieves the value of a specific property of an ECMAScript language value. If the value is
    /// not an object, the property lookup is performed using a wrapper object appropriate for the
    /// type of the value.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getv
    pub(crate) fn get_v<K>(&self, key: K, context: &mut Context<'_>) -> JsResult<Self>
    where
        K: Into<PropertyKey>,
    {
        // 1. Let O be ? ToObject(V).
        let o = self.to_object(context)?;

        // 2. Return ? O.[[Get]](P, V).
        o.__get__(&key.into(), self.clone(), context)
    }

    /// Abstract operation `GetMethod ( V, P )`
    ///
    /// Retrieves the value of a specific property, when the value of the property is expected to be a function.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getmethod
    pub(crate) fn get_method<K>(
        &self,
        key: K,
        context: &mut Context<'_>,
    ) -> JsResult<Option<JsObject>>
    where
        K: Into<PropertyKey>,
    {
        // Note: The spec specifies this function for JsValue.
        // The main part of the function is implemented for JsObject.
        self.to_object(context)?.get_method(key, context)
    }

    /// It is used to create List value whose elements are provided by the indexed properties of
    /// self.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createlistfromarraylike
    pub(crate) fn create_list_from_array_like(
        &self,
        element_types: &[Type],
        context: &mut Context<'_>,
    ) -> JsResult<Vec<Self>> {
        // 1. If elementTypes is not present, set elementTypes to « Undefined, Null, Boolean, String, Symbol, Number, BigInt, Object ».
        let types = if element_types.is_empty() {
            &[
                Type::Undefined,
                Type::Null,
                Type::Boolean,
                Type::String,
                Type::Symbol,
                Type::Number,
                Type::BigInt,
                Type::Object,
            ]
        } else {
            element_types
        };

        // 2. If Type(obj) is not Object, throw a TypeError exception.
        let obj = self.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("cannot create list from a primitive")
        })?;

        // 3. Let len be ? LengthOfArrayLike(obj).
        let len = obj.length_of_array_like(context)?;

        // 4. Let list be a new empty List.
        let mut list = Vec::with_capacity(len as usize);

        // 5. Let index be 0.
        // 6. Repeat, while index < len,
        for index in 0..len {
            // a. Let indexName be ! ToString(𝔽(index)).
            // b. Let next be ? Get(obj, indexName).
            let next = obj.get(index, context)?;
            // c. If Type(next) is not an element of elementTypes, throw a TypeError exception.
            if !types.contains(&next.get_type()) {
                return Err(JsNativeError::typ().with_message("bad type").into());
            }
            // d. Append next as the last element of list.
            list.push(next.clone());
            // e. Set index to index + 1.
        }

        // 7. Return list.
        Ok(list)
    }

    /// Abstract operation [`Call ( F, V [ , argumentsList ] )`][call].
    ///
    /// Calls this value if the value is a callable object.
    ///
    /// # Note
    ///
    /// It is almost always better to try to obtain a callable object first with [`JsValue::as_callable`],
    /// then calling [`JsObject::call`], since that allows reusing the unwrapped function for other
    /// operations. This method is only an utility method for when the spec directly uses `Call`
    /// without using the value as a proper object.
    ///
    /// [call]: https://tc39.es/ecma262/#sec-call
    #[inline]
    pub(crate) fn call(
        &self,
        this: &Self,
        args: &[Self],
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        self.as_callable()
            .ok_or_else(|| {
                JsNativeError::typ().with_message(format!(
                    "value with type `{}` is not callable",
                    self.type_of()
                ))
            })?
            .__call__(this, args, context)
    }

    /// Abstract operation `( V, P [ , argumentsList ] )`
    ///
    /// Calls a method property of an ECMAScript language value.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-invoke
    pub(crate) fn invoke<K>(
        &self,
        key: K,
        args: &[Self],
        context: &mut Context<'_>,
    ) -> JsResult<Self>
    where
        K: Into<PropertyKey>,
    {
        // 1. If argumentsList is not present, set argumentsList to a new empty List.
        // 2. Let func be ? GetV(V, P).
        let func = self.get_v(key, context)?;

        // 3. Return ? Call(func, V, argumentsList)
        func.call(self, args, context)
    }

    /// Abstract operation `OrdinaryHasInstance ( C, O )`
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinaryhasinstance
    pub fn ordinary_has_instance(
        function: &Self,
        object: &Self,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        // 1. If IsCallable(C) is false, return false.
        let Some(function) = function.as_callable() else {
            return Ok(false);
        };

        // 2. If C has a [[BoundTargetFunction]] internal slot, then
        if let Some(bound_function) = function.borrow().as_bound_function() {
            // a. Let BC be C.[[BoundTargetFunction]].
            // b. Return ? InstanceofOperator(O, BC).
            return Self::instance_of(
                object,
                &bound_function.target_function().clone().into(),
                context,
            );
        }

        let Some(mut object) = object.as_object().cloned() else {
            // 3. If Type(O) is not Object, return false.
            return Ok(false);
        };

        // 4. Let P be ? Get(C, "prototype").
        let prototype = function.get("prototype", context)?;

        // 5. If Type(P) is not Object, throw a TypeError exception.
        let prototype = prototype.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("function has non-object prototype in instanceof check")
        })?;

        // 6. Repeat,
        loop {
            // a. Set O to ? O.[[GetPrototypeOf]]().
            object = match object.__get_prototype_of__(context)? {
                Some(obj) => obj,
                // b. If O is null, return false.
                None => return Ok(false),
            };

            // c. If SameValue(P, O) is true, return true.
            if JsObject::equals(&object, prototype) {
                return Ok(true);
            }
        }
    }
}
