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
        FunctionBinding, JsFunction, JsObject, JsPrototype, ObjectData, CONSTRUCTOR, PROTOTYPE,
    },
    property::{Attribute, PropertyDescriptor, PropertyKey},
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
    fn init(intrinsics: &Intrinsics);

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
    let global_object = context.global_object().clone();

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

impl Intrinsics {
    /// Abstract operation [`CreateIntrinsics ( realmRec )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createintrinsics
    pub(crate) fn new() -> Self {
        let intrinsics = Self::default();

        BuiltInFunctionObject::init(&intrinsics);
        BuiltInObjectObject::init(&intrinsics);
        Iterator::init(&intrinsics);
        AsyncIterator::init(&intrinsics);
        AsyncFromSyncIterator::init(&intrinsics);
        ForInIterator::init(&intrinsics);
        Math::init(&intrinsics);
        Json::init(&intrinsics);
        Array::init(&intrinsics);
        ArrayIterator::init(&intrinsics);
        Proxy::init(&intrinsics);
        ArrayBuffer::init(&intrinsics);
        BigInt::init(&intrinsics);
        Boolean::init(&intrinsics);
        Date::init(&intrinsics);
        DataView::init(&intrinsics);
        Map::init(&intrinsics);
        MapIterator::init(&intrinsics);
        IsFinite::init(&intrinsics);
        IsNaN::init(&intrinsics);
        ParseInt::init(&intrinsics);
        ParseFloat::init(&intrinsics);
        Number::init(&intrinsics);
        Eval::init(&intrinsics);
        Set::init(&intrinsics);
        SetIterator::init(&intrinsics);
        String::init(&intrinsics);
        StringIterator::init(&intrinsics);
        RegExp::init(&intrinsics);
        RegExpStringIterator::init(&intrinsics);
        TypedArray::init(&intrinsics);
        Int8Array::init(&intrinsics);
        Uint8Array::init(&intrinsics);
        Uint8ClampedArray::init(&intrinsics);
        Int16Array::init(&intrinsics);
        Uint16Array::init(&intrinsics);
        Int32Array::init(&intrinsics);
        Uint32Array::init(&intrinsics);
        BigInt64Array::init(&intrinsics);
        BigUint64Array::init(&intrinsics);
        Float32Array::init(&intrinsics);
        Float64Array::init(&intrinsics);
        Symbol::init(&intrinsics);
        Error::init(&intrinsics);
        RangeError::init(&intrinsics);
        ReferenceError::init(&intrinsics);
        TypeError::init(&intrinsics);
        ThrowTypeError::init(&intrinsics);
        SyntaxError::init(&intrinsics);
        EvalError::init(&intrinsics);
        UriError::init(&intrinsics);
        AggregateError::init(&intrinsics);
        Reflect::init(&intrinsics);
        Generator::init(&intrinsics);
        GeneratorFunction::init(&intrinsics);
        Promise::init(&intrinsics);
        AsyncFunction::init(&intrinsics);
        AsyncGenerator::init(&intrinsics);
        AsyncGeneratorFunction::init(&intrinsics);
        EncodeUri::init(&intrinsics);
        EncodeUriComponent::init(&intrinsics);
        DecodeUri::init(&intrinsics);
        DecodeUriComponent::init(&intrinsics);
        WeakRef::init(&intrinsics);
        WeakMap::init(&intrinsics);
        WeakSet::init(&intrinsics);

        #[cfg(feature = "annex-b")]
        {
            escape::Escape::init(&intrinsics);
            escape::Unescape::init(&intrinsics);
        }

        #[cfg(feature = "intl")]
        {
            intl::Intl::init(&intrinsics);
            intl::Collator::init(&intrinsics);
            intl::ListFormat::init(&intrinsics);
            intl::Locale::init(&intrinsics);
            intl::DateTimeFormat::init(&intrinsics);
            intl::Segmenter::init(&intrinsics);
        }

        intrinsics
    }
}

/// Abstract operation [`SetDefaultGlobalBindings ( realmRec )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-setdefaultglobalbindings
pub(crate) fn set_default_global_bindings(context: &mut Context<'_>) -> JsResult<()> {
    let global_object = context.global_object().clone();

    global_object.define_property_or_throw(
        utf16!("globalThis"),
        PropertyDescriptor::builder()
            .value(context.realm.global_this().clone())
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
        let global_object = context.global_object().clone();
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
}

/// Marker for an ordinary object.
struct OrdinaryObject;

/// Applies the pending builder data to the object.
trait ApplyToObject {
    fn apply_to(self, object: &JsObject);
}

impl ApplyToObject for Constructor {
    fn apply_to(self, object: &JsObject) {
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
        let mut object = object.borrow_mut();
        object.insert(
            PROTOTYPE,
            PropertyDescriptor::builder()
                .value(self.prototype)
                .writable(false)
                .enumerable(false)
                .configurable(false),
        );
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
        self.kind.apply_to(object);

        let function = function::Function::Native {
            function: NativeFunction::from_fn_ptr(self.function),
            constructor: S::IS_CONSTRUCTOR.then_some(function::ConstructorKind::Base),
        };

        let length = PropertyDescriptor::builder()
            .value(self.length)
            .writable(false)
            .enumerable(false)
            .configurable(true);
        let name = PropertyDescriptor::builder()
            .value(self.name)
            .writable(false)
            .enumerable(false)
            .configurable(true);

        {
            let mut constructor = object.borrow_mut();
            constructor.data = ObjectData::function(function);
            constructor.insert(utf16!("length"), length);
            constructor.insert(utf16!("name"), name);
        }
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
struct BuiltInBuilder<'ctx, Kind> {
    intrinsics: &'ctx Intrinsics,
    object: JsObject,
    kind: Kind,
    prototype: JsObject,
}

impl<'ctx> BuiltInBuilder<'ctx, OrdinaryObject> {
    fn new(intrinsics: &'ctx Intrinsics) -> BuiltInBuilder<'ctx, OrdinaryObject> {
        BuiltInBuilder {
            intrinsics,
            object: JsObject::with_null_proto(),
            kind: OrdinaryObject,
            prototype: intrinsics.constructors().object().prototype(),
        }
    }

    fn with_intrinsic<I: IntrinsicObject>(
        intrinsics: &'ctx Intrinsics,
    ) -> BuiltInBuilder<'ctx, OrdinaryObject> {
        BuiltInBuilder {
            intrinsics,
            object: I::get(intrinsics),
            kind: OrdinaryObject,
            prototype: intrinsics.constructors().object().prototype(),
        }
    }

    fn with_object(
        intrinsics: &'ctx Intrinsics,
        object: JsObject,
    ) -> BuiltInBuilder<'ctx, OrdinaryObject> {
        BuiltInBuilder {
            intrinsics,
            object,
            kind: OrdinaryObject,
            prototype: intrinsics.constructors().object().prototype(),
        }
    }
}

impl<'ctx> BuiltInBuilder<'ctx, OrdinaryObject> {
    fn callable(
        self,
        function: NativeFunctionPointer,
    ) -> BuiltInBuilder<'ctx, Callable<OrdinaryFunction>> {
        BuiltInBuilder {
            intrinsics: self.intrinsics,
            object: self.object,
            kind: Callable {
                function,
                name: js_string!(""),
                length: 0,
                kind: OrdinaryFunction,
            },
            prototype: self.intrinsics.constructors().function().prototype(),
        }
    }
}

impl<'ctx> BuiltInBuilder<'ctx, Callable<Constructor>> {
    fn from_standard_constructor<SC: BuiltInConstructor>(
        intrinsics: &'ctx Intrinsics,
    ) -> BuiltInBuilder<'ctx, Callable<Constructor>> {
        let constructor = SC::STANDARD_CONSTRUCTOR(intrinsics.constructors());
        BuiltInBuilder {
            intrinsics,
            object: constructor.constructor(),
            kind: Callable {
                function: SC::constructor,
                name: js_string!(SC::NAME),
                length: SC::LENGTH,
                kind: Constructor {
                    prototype: constructor.prototype(),
                    inherits: Some(intrinsics.constructors().object().prototype()),
                    attributes: Attribute::WRITABLE | Attribute::CONFIGURABLE,
                },
            },
            prototype: intrinsics.constructors().function().prototype(),
        }
    }

    fn no_proto(self) -> BuiltInBuilder<'ctx, Callable<ConstructorNoProto>> {
        BuiltInBuilder {
            intrinsics: self.intrinsics,
            object: self.object,
            kind: Callable {
                function: self.kind.function,
                name: self.kind.name,
                length: self.kind.length,
                kind: ConstructorNoProto,
            },
            prototype: self.prototype,
        }
    }
}

impl<T> BuiltInBuilder<'_, T> {
    /// Adds a new static method to the builtin object.
    fn static_method<B>(self, function: NativeFunctionPointer, binding: B, length: usize) -> Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = BuiltInBuilder::new(self.intrinsics)
            .callable(function)
            .name(binding.name)
            .length(length)
            .build();

        self.object.borrow_mut().insert(
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
    fn static_property<K, V>(self, key: K, value: V, attribute: Attribute) -> Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.object.borrow_mut().insert(key, property);
        self
    }

    /// Adds a new static accessor property to the builtin object.
    fn static_accessor<K>(
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
        self.object.borrow_mut().insert(key, property);
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
        let function = BuiltInBuilder::new(self.intrinsics)
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
    fn build(self) -> JsObject {
        self.kind.apply_to(&self.object);

        {
            let mut object = self.object.borrow_mut();
            object.set_prototype(self.prototype);
        }

        self.object
    }
}

impl<FnTyp: ApplyToObject + IsConstructor> BuiltInBuilder<'_, Callable<FnTyp>> {
    /// Build the builtin callable.
    fn build(self) -> JsFunction {
        self.kind.apply_to(&self.object);

        {
            let mut object = self.object.borrow_mut();
            object.set_prototype(self.prototype);
        }

        JsFunction::from_object_unchecked(self.object)
    }
}
