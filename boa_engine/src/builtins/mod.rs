//! Boa's ECMAScript built-in object implementations, e.g. Object, String, Math, Array, etc.
//!
//! This module also contains a JavaScript Console implementation.

pub mod array;
pub mod array_buffer;
pub mod async_function;
pub mod async_generator;
pub mod async_generator_function;
pub mod bigint;
pub mod boolean;
pub mod dataview;
pub mod date;
pub mod error;
pub mod eval;
pub mod function;
pub mod generator;
pub mod generator_function;
pub mod iterable;
pub mod json;
pub mod map;
pub mod math;
pub mod number;
pub mod object;
pub mod promise;
pub mod proxy;
pub mod reflect;
pub mod regexp;
pub mod set;
pub mod string;
pub mod symbol;
pub mod typed_array;
pub mod uri;
pub mod weak;
pub mod weak_map;
pub mod weak_set;

#[cfg(feature = "annex-b")]
pub mod escape;

#[cfg(feature = "intl")]
pub mod intl;

pub(crate) use self::{
    array::Array,
    async_function::AsyncFunction,
    bigint::BigInt,
    boolean::Boolean,
    dataview::DataView,
    date::Date,
    error::{
        AggregateError, Error, EvalError, RangeError, ReferenceError, SyntaxError, TypeError,
        UriError,
    },
    eval::Eval,
    function::BuiltInFunctionObject,
    json::Json,
    map::Map,
    math::Math,
    number::{IsFinite, IsNaN, Number, ParseFloat, ParseInt},
    object::Object as BuiltInObjectObject,
    promise::Promise,
    proxy::Proxy,
    reflect::Reflect,
    regexp::RegExp,
    set::Set,
    string::String,
    symbol::Symbol,
    typed_array::{
        BigInt64Array, BigUint64Array, Float32Array, Float64Array, Int16Array, Int32Array,
        Int8Array, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
    },
};

use crate::{
    builtins::{
        array::ArrayIterator,
        array_buffer::ArrayBuffer,
        async_generator::AsyncGenerator,
        async_generator_function::AsyncGeneratorFunction,
        error::r#type::ThrowTypeError,
        generator::Generator,
        generator_function::GeneratorFunction,
        iterable::{AsyncFromSyncIterator, AsyncIterator, Iterator},
        map::MapIterator,
        object::for_in_iterator::ForInIterator,
        regexp::RegExpStringIterator,
        set::SetIterator,
        string::StringIterator,
        typed_array::TypedArray,
        uri::{DecodeUri, DecodeUriComponent, EncodeUri, EncodeUriComponent},
        weak::WeakRef,
        weak_map::WeakMap,
        weak_set::WeakSet,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    native_function::{NativeFunction, NativeFunctionPointer},
    object::{
        FunctionBinding, JsFunction, JsObject, JsPrototype, Object, ObjectData, CONSTRUCTOR,
        PROTOTYPE,
    },
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    string::utf16,
    Context, JsResult, JsString, JsValue,
};

#[cfg(feature = "console")]
use crate::console::Console;

/// A [Well-Known Intrinsic Object].
///
/// Well-known intrinsics are built-in objects that are explicitly referenced by the algorithms of
/// the specification and which usually have realm-specific identities.
///
/// [Well-Known Intrinsic Object]: https://tc39.es/ecma262/#sec-well-known-intrinsic-objects
pub(crate) trait IntrinsicObject {
    /// Initializes the intrinsic object.
    ///
    /// This is where the methods, properties, static methods and the constructor of a built-in must
    /// be initialized to be accessible from ECMAScript.
    fn init(realm: &Realm);

    /// Gets the intrinsic object.
    fn get(intrinsics: &Intrinsics) -> JsObject;
}

/// A [built-in object].
///
/// This trait must be implemented for any global built-in that lives in the global context of a script.
///
/// [built-in object]: https://tc39.es/ecma262/#sec-built-in-object
pub(crate) trait BuiltInObject: IntrinsicObject {
    /// Binding name of the builtin inside the global object.
    ///
    /// E.g. If you want access the properties of a `Complex` built-in with the name `Cplx` you must
    /// assign `"Cplx"` to this constant, making any property inside it accessible from ECMAScript
    /// as `Cplx.prop`
    const NAME: &'static str;

    /// Property attribute flags of the built-in. Check [`Attribute`] for more information.
    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);
}

/// A [built-in object] that is also a constructor.
///
/// This trait must be implemented for any global built-in that can also be called with `new` to
/// construct an object instance e.g. `Array`, `Map` or `Object`.
///
/// [built-in object]: https://tc39.es/ecma262/#sec-built-in-object
pub(crate) trait BuiltInConstructor: BuiltInObject {
    /// The amount of arguments this function object takes.
    const LENGTH: usize;

    /// The corresponding standard constructor of this constructor.
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor;

    /// The native constructor function.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
}

fn global_binding<B: BuiltInObject>(context: &mut Context<'_>) -> JsResult<()> {
    let name = B::NAME;
    let attr = B::ATTRIBUTE;
    let intrinsic = B::get(context.intrinsics());
    let global_object = context.global_object();

    global_object.define_property_or_throw(
        name,
        PropertyDescriptor::builder()
            .value(intrinsic)
            .writable(attr.writable())
            .enumerable(attr.enumerable())
            .configurable(attr.configurable())
            .build(),
        context,
    )?;
    Ok(())
}

impl Realm {
    /// Abstract operation [`CreateIntrinsics ( realmRec )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createintrinsics
    pub(crate) fn initialize(&self) {
        BuiltInFunctionObject::init(self);
        BuiltInObjectObject::init(self);
        Iterator::init(self);
        AsyncIterator::init(self);
        AsyncFromSyncIterator::init(self);
        ForInIterator::init(self);
        Math::init(self);
        Json::init(self);
        Array::init(self);
        ArrayIterator::init(self);
        Proxy::init(self);
        ArrayBuffer::init(self);
        BigInt::init(self);
        Boolean::init(self);
        Date::init(self);
        DataView::init(self);
        Map::init(self);
        MapIterator::init(self);
        IsFinite::init(self);
        IsNaN::init(self);
        ParseInt::init(self);
        ParseFloat::init(self);
        Number::init(self);
        Eval::init(self);
        Set::init(self);
        SetIterator::init(self);
        String::init(self);
        StringIterator::init(self);
        RegExp::init(self);
        RegExpStringIterator::init(self);
        TypedArray::init(self);
        Int8Array::init(self);
        Uint8Array::init(self);
        Uint8ClampedArray::init(self);
        Int16Array::init(self);
        Uint16Array::init(self);
        Int32Array::init(self);
        Uint32Array::init(self);
        BigInt64Array::init(self);
        BigUint64Array::init(self);
        Float32Array::init(self);
        Float64Array::init(self);
        Symbol::init(self);
        Error::init(self);
        RangeError::init(self);
        ReferenceError::init(self);
        TypeError::init(self);
        ThrowTypeError::init(self);
        SyntaxError::init(self);
        EvalError::init(self);
        UriError::init(self);
        AggregateError::init(self);
        Reflect::init(self);
        Generator::init(self);
        GeneratorFunction::init(self);
        Promise::init(self);
        AsyncFunction::init(self);
        AsyncGenerator::init(self);
        AsyncGeneratorFunction::init(self);
        EncodeUri::init(self);
        EncodeUriComponent::init(self);
        DecodeUri::init(self);
        DecodeUriComponent::init(self);
        WeakRef::init(self);
        WeakMap::init(self);
        WeakSet::init(self);

        #[cfg(feature = "annex-b")]
        {
            escape::Escape::init(self);
            escape::Unescape::init(self);
        }

        #[cfg(feature = "intl")]
        {
            intl::Intl::init(self);
            intl::Collator::init(self);
            intl::ListFormat::init(self);
            intl::Locale::init(self);
            intl::DateTimeFormat::init(self);
            intl::Segmenter::init(self);
            intl::segmenter::Segments::init(self);
            intl::segmenter::SegmentIterator::init(self);
        }
    }
}

/// Abstract operation [`SetDefaultGlobalBindings ( realmRec )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-setdefaultglobalbindings
pub(crate) fn set_default_global_bindings(context: &mut Context<'_>) -> JsResult<()> {
    let global_object = context.global_object();

    global_object.define_property_or_throw(
        utf16!("globalThis"),
        PropertyDescriptor::builder()
            .value(context.realm().global_this().clone())
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;
    let restricted = PropertyDescriptor::builder()
        .writable(false)
        .enumerable(false)
        .configurable(false);
    global_object.define_property_or_throw(
        utf16!("Infinity"),
        restricted.clone().value(f64::INFINITY),
        context,
    )?;
    global_object.define_property_or_throw(
        utf16!("NaN"),
        restricted.clone().value(f64::NAN),
        context,
    )?;
    global_object.define_property_or_throw(
        utf16!("undefined"),
        restricted.value(JsValue::undefined()),
        context,
    )?;

    global_binding::<BuiltInFunctionObject>(context)?;
    global_binding::<BuiltInObjectObject>(context)?;
    global_binding::<Math>(context)?;
    global_binding::<Json>(context)?;
    global_binding::<Array>(context)?;
    global_binding::<Proxy>(context)?;
    global_binding::<ArrayBuffer>(context)?;
    global_binding::<BigInt>(context)?;
    global_binding::<Boolean>(context)?;
    global_binding::<Date>(context)?;
    global_binding::<DataView>(context)?;
    global_binding::<Map>(context)?;
    global_binding::<IsFinite>(context)?;
    global_binding::<IsNaN>(context)?;
    global_binding::<ParseInt>(context)?;
    global_binding::<ParseFloat>(context)?;
    global_binding::<Number>(context)?;
    global_binding::<Eval>(context)?;
    global_binding::<Set>(context)?;
    global_binding::<String>(context)?;
    global_binding::<RegExp>(context)?;
    global_binding::<TypedArray>(context)?;
    global_binding::<Int8Array>(context)?;
    global_binding::<Uint8Array>(context)?;
    global_binding::<Uint8ClampedArray>(context)?;
    global_binding::<Int16Array>(context)?;
    global_binding::<Uint16Array>(context)?;
    global_binding::<Int32Array>(context)?;
    global_binding::<Uint32Array>(context)?;
    global_binding::<BigInt64Array>(context)?;
    global_binding::<BigUint64Array>(context)?;
    global_binding::<Float32Array>(context)?;
    global_binding::<Float64Array>(context)?;
    global_binding::<Symbol>(context)?;
    global_binding::<Error>(context)?;
    global_binding::<RangeError>(context)?;
    global_binding::<ReferenceError>(context)?;
    global_binding::<TypeError>(context)?;
    global_binding::<SyntaxError>(context)?;
    global_binding::<EvalError>(context)?;
    global_binding::<UriError>(context)?;
    global_binding::<AggregateError>(context)?;
    global_binding::<Reflect>(context)?;
    global_binding::<Promise>(context)?;
    global_binding::<EncodeUri>(context)?;
    global_binding::<EncodeUriComponent>(context)?;
    global_binding::<DecodeUri>(context)?;
    global_binding::<DecodeUriComponent>(context)?;
    global_binding::<WeakRef>(context)?;
    global_binding::<WeakMap>(context)?;
    global_binding::<WeakSet>(context)?;

    #[cfg(feature = "annex-b")]
    {
        global_binding::<escape::Escape>(context)?;
        global_binding::<escape::Unescape>(context)?;
    }

    #[cfg(feature = "intl")]
    global_binding::<intl::Intl>(context)?;

    #[cfg(feature = "console")]
    {
        let object = Console::init(context);
        let global_object = context.global_object();
        global_object.define_property_or_throw(
            utf16!("console"),
            PropertyDescriptor::builder()
                .value(object)
                .writable(true)
                .enumerable(true)
                .configurable(true),
            context,
        )?;
    }

    Ok(())
}

// === Builder typestate ===

#[derive(Debug)]
enum BuiltInObjectInitializer {
    Shared(JsObject),
    Unique { object: Object, data: ObjectData },
}

impl BuiltInObjectInitializer {
    /// Inserts a new property descriptor into the builtin.
    fn insert<K, P>(&mut self, key: K, property: P)
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        match self {
            BuiltInObjectInitializer::Shared(obj) => obj.borrow_mut().insert(key, property),
            BuiltInObjectInitializer::Unique { object, .. } => object.insert(key, property),
        };
    }

    /// Sets the prototype of the builtin
    fn set_prototype(&mut self, prototype: JsObject) {
        match self {
            BuiltInObjectInitializer::Shared(obj) => {
                let mut obj = obj.borrow_mut();
                obj.set_prototype(prototype);
            }
            BuiltInObjectInitializer::Unique { object, .. } => {
                object.set_prototype(prototype);
            }
        }
    }

    /// Sets the `ObjectData` of the builtin.
    ///
    /// # Panics
    ///
    /// Panics if the builtin is a shared builtin and the data's vtable is not the same as the
    /// builtin's vtable.
    fn set_data(&mut self, new_data: ObjectData) {
        match self {
            BuiltInObjectInitializer::Shared(obj) => {
                assert!(
                    std::ptr::eq(obj.vtable(), new_data.internal_methods),
                    "intrinsic object's vtable didn't match with new data"
                );
                *obj.borrow_mut().kind_mut() = new_data.kind;
            }
            BuiltInObjectInitializer::Unique { ref mut data, .. } => *data = new_data,
        }
    }

    /// Gets a shared object from the builtin, transitioning its state if it's necessary.
    fn as_shared(&mut self) -> JsObject {
        match std::mem::replace(
            self,
            BuiltInObjectInitializer::Unique {
                object: Object::default(),
                data: ObjectData::ordinary(),
            },
        ) {
            BuiltInObjectInitializer::Shared(obj) => {
                *self = BuiltInObjectInitializer::Shared(obj.clone());
                obj
            }
            BuiltInObjectInitializer::Unique { mut object, data } => {
                *object.kind_mut() = data.kind;
                let obj = JsObject::from_object_and_vtable(object, data.internal_methods);
                *self = BuiltInObjectInitializer::Shared(obj.clone());
                obj
            }
        }
    }

    /// Converts the builtin into a shared object.
    fn into_shared(mut self) -> JsObject {
        self.as_shared()
    }
}

/// Marker for a constructor function.
struct Constructor {
    prototype: JsObject,
    inherits: JsPrototype,
    attributes: Attribute,
}

/// Marker for a constructor function without a custom prototype for its instances.
struct ConstructorNoProto;

/// Marker for an ordinary function.
struct OrdinaryFunction;

/// Indicates if the marker is a constructor.
trait IsConstructor {
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
struct Callable<Kind> {
    function: NativeFunctionPointer,
    name: JsString,
    length: usize,
    kind: Kind,
    realm: Realm,
}

/// Marker for an ordinary object.
struct OrdinaryObject;

/// Applies the pending builder data to the object.
trait ApplyToObject {
    fn apply_to(self, object: &mut BuiltInObjectInitializer);
}

impl ApplyToObject for Constructor {
    fn apply_to(self, object: &mut BuiltInObjectInitializer) {
        object.insert(
            PROTOTYPE,
            PropertyDescriptor::builder()
                .value(self.prototype.clone())
                .writable(false)
                .enumerable(false)
                .configurable(false),
        );

        let object = object.as_shared();

        {
            let mut prototype = self.prototype.borrow_mut();
            prototype.set_prototype(self.inherits);
            prototype.insert(
                CONSTRUCTOR,
                PropertyDescriptor::builder()
                    .value(object)
                    .writable(self.attributes.writable())
                    .enumerable(self.attributes.enumerable())
                    .configurable(self.attributes.configurable()),
            );
        }
    }
}

impl ApplyToObject for ConstructorNoProto {
    fn apply_to(self, _: &mut BuiltInObjectInitializer) {}
}

impl ApplyToObject for OrdinaryFunction {
    fn apply_to(self, _: &mut BuiltInObjectInitializer) {}
}

impl<S: ApplyToObject + IsConstructor> ApplyToObject for Callable<S> {
    fn apply_to(self, object: &mut BuiltInObjectInitializer) {
        let function = ObjectData::function(function::Function::new(
            function::FunctionKind::Native {
                function: NativeFunction::from_fn_ptr(self.function),
                constructor: S::IS_CONSTRUCTOR.then_some(function::ConstructorKind::Base),
            },
            self.realm,
        ));
        object.set_data(function);
        object.insert(
            utf16!("length"),
            PropertyDescriptor::builder()
                .value(self.length)
                .writable(false)
                .enumerable(false)
                .configurable(true),
        );
        object.insert(
            utf16!("name"),
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
    fn apply_to(self, _: &mut BuiltInObjectInitializer) {}
}

/// Builder for creating built-in objects, like `Array`.
///
/// The marker `ObjectType` restricts the methods that can be called depending on the
/// type of object that is being constructed.
#[derive(Debug)]
#[must_use = "You need to call the `build` method in order for this to correctly assign the inner data"]
struct BuiltInBuilder<'ctx, Kind> {
    realm: &'ctx Realm,
    object: BuiltInObjectInitializer,
    kind: Kind,
    prototype: JsObject,
}

impl<'ctx> BuiltInBuilder<'ctx, OrdinaryObject> {
    fn new(realm: &'ctx Realm) -> BuiltInBuilder<'ctx, OrdinaryObject> {
        BuiltInBuilder {
            realm,
            object: BuiltInObjectInitializer::Unique {
                object: Object::default(),
                data: ObjectData::ordinary(),
            },
            kind: OrdinaryObject,
            prototype: realm.intrinsics().constructors().object().prototype(),
        }
    }

    fn with_intrinsic<I: IntrinsicObject>(
        realm: &'ctx Realm,
    ) -> BuiltInBuilder<'ctx, OrdinaryObject> {
        BuiltInBuilder {
            realm,
            object: BuiltInObjectInitializer::Shared(I::get(realm.intrinsics())),
            kind: OrdinaryObject,
            prototype: realm.intrinsics().constructors().object().prototype(),
        }
    }

    fn with_object(realm: &'ctx Realm, object: JsObject) -> BuiltInBuilder<'ctx, OrdinaryObject> {
        BuiltInBuilder {
            realm,
            object: BuiltInObjectInitializer::Shared(object),
            kind: OrdinaryObject,
            prototype: realm.intrinsics().constructors().object().prototype(),
        }
    }
}

impl<'ctx> BuiltInBuilder<'ctx, OrdinaryObject> {
    fn callable(
        self,
        function: NativeFunctionPointer,
    ) -> BuiltInBuilder<'ctx, Callable<OrdinaryFunction>> {
        BuiltInBuilder {
            realm: self.realm,
            object: self.object,
            kind: Callable {
                function,
                name: js_string!(""),
                length: 0,
                kind: OrdinaryFunction,
                realm: self.realm.clone(),
            },
            prototype: self
                .realm
                .intrinsics()
                .constructors()
                .function()
                .prototype(),
        }
    }
}

impl<'ctx> BuiltInBuilder<'ctx, Callable<Constructor>> {
    fn from_standard_constructor<SC: BuiltInConstructor>(
        realm: &'ctx Realm,
    ) -> BuiltInBuilder<'ctx, Callable<Constructor>> {
        let constructor = SC::STANDARD_CONSTRUCTOR(realm.intrinsics().constructors());
        BuiltInBuilder {
            realm,
            object: BuiltInObjectInitializer::Shared(constructor.constructor()),
            kind: Callable {
                function: SC::constructor,
                name: js_string!(SC::NAME),
                length: SC::LENGTH,
                kind: Constructor {
                    prototype: constructor.prototype(),
                    inherits: Some(realm.intrinsics().constructors().object().prototype()),
                    attributes: Attribute::WRITABLE | Attribute::CONFIGURABLE,
                },
                realm: realm.clone(),
            },
            prototype: realm.intrinsics().constructors().function().prototype(),
        }
    }

    fn no_proto(self) -> BuiltInBuilder<'ctx, Callable<ConstructorNoProto>> {
        BuiltInBuilder {
            realm: self.realm,
            object: self.object,
            kind: Callable {
                function: self.kind.function,
                name: self.kind.name,
                length: self.kind.length,
                kind: ConstructorNoProto,
                realm: self.realm.clone(),
            },
            prototype: self.prototype,
        }
    }
}

impl<T> BuiltInBuilder<'_, T> {
    /// Adds a new static method to the builtin object.
    fn static_method<B>(
        mut self,
        function: NativeFunctionPointer,
        binding: B,
        length: usize,
    ) -> Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = BuiltInBuilder::new(self.realm)
            .callable(function)
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
    fn static_property<K, V>(mut self, key: K, value: V, attribute: Attribute) -> Self
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

    /// Adds a new static accessor property to the builtin object.
    fn static_accessor<K>(
        mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> Self
    where
        K: Into<PropertyKey>,
    {
        let property = PropertyDescriptor::builder()
            .maybe_get(get)
            .maybe_set(set)
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.object.insert(key, property);
        self
    }

    /// Specify the `[[Prototype]]` internal field of the builtin object.
    ///
    /// Default is `Function.prototype` for constructors and `Object.prototype` for statics.
    fn prototype(mut self, prototype: JsObject) -> Self {
        self.prototype = prototype;
        self
    }
}

impl BuiltInBuilder<'_, Callable<Constructor>> {
    /// Adds a new method to the constructor's prototype.
    fn method<B>(self, function: NativeFunctionPointer, binding: B, length: usize) -> Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = BuiltInBuilder::new(self.realm)
            .callable(function)
            .name(binding.name)
            .length(length)
            .build();

        self.kind.kind.prototype.borrow_mut().insert(
            binding.binding,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        self
    }

    /// Adds a new data property to the constructor's prototype.
    fn property<K, V>(self, key: K, value: V, attribute: Attribute) -> Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.kind.kind.prototype.borrow_mut().insert(key, property);
        self
    }

    /// Adds new accessor property to the constructor's prototype.
    fn accessor<K>(
        self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> Self
    where
        K: Into<PropertyKey>,
    {
        let property = PropertyDescriptor::builder()
            .maybe_get(get)
            .maybe_set(set)
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.kind.kind.prototype.borrow_mut().insert(key, property);
        self
    }

    /// Specifies the parent prototype which objects created by this constructor inherit from.
    ///
    /// Default is `Object.prototype`.
    #[allow(clippy::missing_const_for_fn)]
    fn inherits(mut self, prototype: JsPrototype) -> Self {
        self.kind.kind.inherits = prototype;
        self
    }

    /// Specifies the property attributes of the prototype's "constructor" property.
    const fn constructor_attributes(mut self, attributes: Attribute) -> Self {
        self.kind.kind.attributes = attributes;
        self
    }
}

impl<FnTyp> BuiltInBuilder<'_, Callable<FnTyp>> {
    /// Specify how many arguments the constructor function takes.
    ///
    /// Default is `0`.
    #[inline]
    const fn length(mut self, length: usize) -> Self {
        self.kind.length = length;
        self
    }

    /// Specify the name of the constructor function.
    ///
    /// Default is `""`
    fn name<N: Into<JsString>>(mut self, name: N) -> Self {
        self.kind.name = name.into();
        self
    }
}

impl BuiltInBuilder<'_, OrdinaryObject> {
    /// Build the builtin object.
    fn build(mut self) -> JsObject {
        self.kind.apply_to(&mut self.object);

        self.object.set_prototype(self.prototype);

        self.object.into_shared()
    }
}

impl<FnTyp: ApplyToObject + IsConstructor> BuiltInBuilder<'_, Callable<FnTyp>> {
    /// Build the builtin callable.
    fn build(mut self) -> JsFunction {
        self.kind.apply_to(&mut self.object);

        self.object.set_prototype(self.prototype);

        JsFunction::from_object_unchecked(self.object.into_shared())
    }
}
