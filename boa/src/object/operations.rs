use crate::{
    builtins::Array,
    property::{PropertyDescriptor, PropertyKey, PropertyNameKind},
    symbol::WellKnownSymbols,
    value::Type,
    Context, JsResult, JsValue,
};

use super::JsObject;

impl JsObject {
    /// Get property from object or throw.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-o-p
    #[inline]
    pub(crate) fn get<K>(&self, key: K, context: &mut Context) -> JsResult<JsValue>
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
    #[inline]
    pub(crate) fn set<K, V>(
        &self,
        key: K,
        value: V,
        throw: bool,
        context: &mut Context,
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
            return Err(
                context.construct_type_error(format!("cannot set non-writable property: {}", key))
            );
        }
        // 6. Return success.
        Ok(success)
    }

    /// Create data property
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-deletepropertyorthrow
    pub(crate) fn create_data_property<K, V>(
        &self,
        key: K,
        value: V,
        context: &mut Context,
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
    /// [spec]: https://tc39.es/ecma262/#sec-deletepropertyorthrow
    pub(crate) fn create_data_property_or_throw<K, V>(
        &self,
        key: K,
        value: V,
        context: &mut Context,
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
            return Err(context.construct_type_error(format!("cannot redefine property: {}", key)));
        }
        // 5. Return success.
        Ok(success)
    }

    /// Define property or throw.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-definepropertyorthrow
    #[inline]
    pub(crate) fn define_property_or_throw<K, P>(
        &self,
        key: K,
        desc: P,
        context: &mut Context,
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
            return Err(context.construct_type_error(format!("cannot redefine property: {}", key)));
        }
        // 5. Return success.
        Ok(success)
    }

    /// Defines the property or throws a `TypeError` if the operation fails.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-definepropertyorthrow
    #[inline]
    pub(crate) fn delete_property_or_throw<K>(
        &self,
        key: K,
        context: &mut Context,
    ) -> JsResult<bool>
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
            return Err(context.construct_type_error(format!("cannot delete property: {}", key)));
        }
        // 5. Return success.
        Ok(success)
    }

    /// Retrieves value of specific property, when the value of the property is expected to be a function.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getmethod
    #[inline]
    pub(crate) fn get_method<K>(&self, context: &mut Context, key: K) -> JsResult<Option<JsObject>>
    where
        K: Into<PropertyKey>,
    {
        // 1. Assert: IsPropertyKey(P) is true.
        // 2. Let func be ? GetV(V, P).
        let value = self.get(key, context)?;

        // 3. If func is either undefined or null, return undefined.
        if value.is_null_or_undefined() {
            return Ok(None);
        }

        // 4. If IsCallable(func) is false, throw a TypeError exception.
        // 5. Return func.
        match value.as_object() {
            Some(object) if object.is_callable() => Ok(Some(object)),
            _ => Err(context
                .construct_type_error("value returned for property of object is not a function")),
        }
    }

    /// Check if object has property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasproperty
    // NOTE: for now context is not used but it will in the future.
    #[inline]
    pub(crate) fn has_property<K>(&self, key: K, context: &mut Context) -> JsResult<bool>
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
    #[inline]
    pub(crate) fn has_own_property<K>(&self, key: K, context: &mut Context) -> JsResult<bool>
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
    pub(crate) fn call(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        self.call_construct(this, args, context, false)
    }

    /// Construct an instance of this object with the specified arguments.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    // <https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget>
    #[track_caller]
    #[inline]
    pub(crate) fn construct(
        &self,
        args: &[JsValue],
        new_target: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        self.call_construct(new_target, args, context, true)
    }

    // todo: SetIntegrityLevel

    // todo: TestIntegrityLevel

    pub(crate) fn length_of_array_like(&self, context: &mut Context) -> JsResult<usize> {
        // 1. Assert: Type(obj) is Object.
        // 2. Return ℝ(? ToLength(? Get(obj, "length"))).
        self.get("length", context)?.to_length(context)
    }

    /// `7.3.22 SpeciesConstructor ( O, defaultConstructor )`
    ///
    /// The abstract operation SpeciesConstructor takes arguments O (an Object) and defaultConstructor (a constructor).
    /// It is used to retrieve the constructor that should be used to create new objects that are derived from O.
    /// defaultConstructor is the constructor to use if a constructor @@species property cannot be found starting from O.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-speciesconstructor
    pub(crate) fn species_constructor(
        &self,
        default_constructor: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Assert: Type(O) is Object.

        // 2. Let C be ? Get(O, "constructor").
        let c = self.clone().get("constructor", context)?;

        // 3. If C is undefined, return defaultConstructor.
        if c.is_undefined() {
            return Ok(default_constructor);
        }

        // 4. If Type(C) is not Object, throw a TypeError exception.
        if !c.is_object() {
            return context.throw_type_error("property 'constructor' is not an object");
        }

        // 5. Let S be ? Get(C, @@species).
        let s = c.get_field(WellKnownSymbols::species(), context)?;

        // 6. If S is either undefined or null, return defaultConstructor.
        if s.is_null_or_undefined() {
            return Ok(default_constructor);
        }

        // 7. If IsConstructor(S) is true, return S.
        // 8. Throw a TypeError exception.
        if let Some(obj) = s.as_object() {
            if obj.is_constructable() {
                Ok(s)
            } else {
                context.throw_type_error("property 'constructor' is not a constructor")
            }
        } else {
            context.throw_type_error("property 'constructor' is not an object")
        }
    }

    /// It is used to iterate over names of object's keys.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-enumerableownpropertynames
    pub(crate) fn enumerable_own_property_names(
        &self,
        kind: PropertyNameKind,
        context: &mut Context,
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
                _ => None,
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
                                properties.push(self.get(key.clone(), context)?)
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

    // todo: GetFunctionRealm

    // todo: CopyDataProperties

    // todo: PrivateElementFind

    // todo: PrivateFieldAdd

    // todo: PrivateMethodOrAccessorAdd

    // todo: PrivateGet

    // todo: PrivateSet

    // todo: DefineField

    // todo: InitializeInstanceElements
}

impl JsValue {
    // todo: GetV

    // todo: GetMethod

    /// It is used to create List value whose elements are provided by the indexed properties of
    /// self.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createlistfromarraylike
    pub(crate) fn create_list_from_array_like(
        &self,
        element_types: &[Type],
        context: &mut Context,
    ) -> JsResult<Vec<JsValue>> {
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
        let obj = self
            .as_object()
            .ok_or_else(|| context.construct_type_error("cannot create list from a primitive"))?;

        // 3. Let len be ? LengthOfArrayLike(obj).
        let len = obj.length_of_array_like(context)?;

        // 4. Let list be a new empty List.
        let mut list = Vec::with_capacity(len);

        // 5. Let index be 0.
        // 6. Repeat, while index < len,
        for index in 0..len {
            // a. Let indexName be ! ToString(𝔽(index)).
            // b. Let next be ? Get(obj, indexName).
            let next = obj.get(index, context)?;
            // c. If Type(next) is not an element of elementTypes, throw a TypeError exception.
            if !types.contains(&next.get_type()) {
                return Err(context.construct_type_error("bad type"));
            }
            // d. Append next as the last element of list.
            list.push(next.clone());
            // e. Set index to index + 1.
        }

        // 7. Return list.
        Ok(list)
    }
}
