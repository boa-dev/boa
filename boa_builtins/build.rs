use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::{env, fmt};

use bitflags::bitflags;
use phf_shared::{FmtConst, PhfBorrow, PhfHash};

use boa_macros::utf16;

bitflags! {
    /// This struct constains the property flags as described in the ECMAScript specification.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Attribute: u8 {
        /// The `Writable` attribute decides whether the value associated with the property can be changed or not, from its initial value.
        const WRITABLE = 0b0000_0001;

        /// If the property can be enumerated by a `for-in` loop.
        const ENUMERABLE = 0b0000_0010;

        /// If the property descriptor can be changed later.
        const CONFIGURABLE = 0b0000_0100;

        const GET = 0b0000_1000;
        const SET = 0b0001_0000;
    }
}

/// List of well known symbols.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
enum WellKnown {
    AsyncIterator,
    HasInstance,
    IsConcatSpreadable,
    Iterator,
    Match,
    MatchAll,
    Replace,
    Search,
    Species,
    Split,
    ToPrimitive,
    ToStringTag,
    Unscopables,
}

pub struct EncodedStaticPropertyKey(u16);

impl EncodedStaticPropertyKey {
    #[inline]
    pub fn decode(&self) -> StaticPropertyKey {
        let value = self.0 >> 1;
        if self.0 & 1 == 0 {
            StaticPropertyKey::String(value)
        } else {
            StaticPropertyKey::Symbol(value as u8)
        }
    }
}

const fn string(index: u16) -> EncodedStaticPropertyKey {
    debug_assert!(index < 2u16.pow(15));

    EncodedStaticPropertyKey(index << 1)
}

const fn symbol(index: u8) -> EncodedStaticPropertyKey {
    EncodedStaticPropertyKey(((index as u16) << 1) | 1)
}

impl Debug for EncodedStaticPropertyKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.decode().fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StaticPropertyKey {
    String(u16),
    Symbol(u8),
}

impl StaticPropertyKey {
    #[inline]
    pub fn encode(self) -> EncodedStaticPropertyKey {
        match self {
            StaticPropertyKey::String(x) => string(x),
            StaticPropertyKey::Symbol(x) => symbol(x),
        }
    }
}

impl Debug for StaticPropertyKey {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            StaticPropertyKey::String(index) => {
                let string = RAW_STATICS[index as usize];
                let string = String::from_utf16_lossy(string);
                write!(f, "String(\"{string}\")")
            }
            StaticPropertyKey::Symbol(symbol) => {
                write!(f, "Symbol({symbol})")
            }
        }
    }
}

impl Eq for EncodedStaticPropertyKey {}

impl PartialEq for EncodedStaticPropertyKey {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for EncodedStaticPropertyKey {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PhfHash for EncodedStaticPropertyKey {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

impl PhfBorrow<EncodedStaticPropertyKey> for EncodedStaticPropertyKey {
    #[inline]
    fn borrow(&self) -> &EncodedStaticPropertyKey {
        self
    }
}

impl FmtConst for EncodedStaticPropertyKey {
    #[inline]
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = self.decode();
        if matches!(key, StaticPropertyKey::String { .. }) {
            f.write_str("string(")?;
        } else {
            f.write_str("symbol(")?;
        }

        match key {
            StaticPropertyKey::String(index) => {
                write!(
                    f,
                    "/* */ {index})",
                    // String::from_utf16_lossy(value),
                )
            }
            StaticPropertyKey::Symbol(s) => write!(f, "{})", s),
        }
    }
}

trait ToPropertyKey {
    fn to_property_key(self, _context: &Context) -> StaticPropertyKey;
}

impl ToPropertyKey for &'static [u16] {
    fn to_property_key(self, context: &Context) -> StaticPropertyKey {
        let index = context.insert_or_get(self);
        StaticPropertyKey::String(index)
    }
}

impl ToPropertyKey for WellKnown {
    fn to_property_key(self, _context: &Context) -> StaticPropertyKey {
        StaticPropertyKey::Symbol(self as u8)
    }
}

#[allow(clippy::type_complexity)]
struct Context {
    strings: RefCell<(HashMap<&'static [u16], u16>, Vec<&'static [u16]>)>,
}

impl Context {
    fn new() -> Self {
        Self {
            strings: RefCell::default(),
        }
    }

    fn insert_or_get(&self, value: &'static [u16]) -> u16 {
        let mut strings = self.strings.borrow_mut();
        if let Some(index) = strings.0.get(value) {
            return *index;
        }
        let index = strings.1.len();
        debug_assert!(index < u16::MAX as usize);
        let index = index as u16;

        strings.0.insert(value, index);
        strings.1.push(value);

        index
    }

    fn build(&self, file: &mut BufWriter<File>) -> io::Result<()> {
        let strings = self.strings.borrow();

        let len = strings.1.len();

        writeln!(file, "\npub const RAW_STATICS: &[&[u16]; {len}] = &[")?;
        for string in &strings.1 {
            writeln!(
                file,
                "    /* {} */ &{:?},",
                String::from_utf16_lossy(string),
                string
            )?;
        }
        writeln!(file, "];")?;
        Ok(())
    }
}

struct BuiltInBuilder<'a> {
    context: &'a Context,
    name: &'static str,
    map: phf_codegen::OrderedMap<EncodedStaticPropertyKey>,
    prototype: Option<&'static str>,

    slot_index: usize,
}

impl<'a> BuiltInBuilder<'a> {
    fn new(context: &'a Context, name: &'static str) -> Self {
        Self {
            context,
            name,
            map: phf_codegen::OrderedMap::new(),
            prototype: None,
            slot_index: 0,
        }
    }

    fn inherits(&mut self, prototype: &'static str) -> &mut Self {
        self.prototype = Some(prototype);
        self
    }

    fn method<K>(&mut self, key: K) -> &mut Self
    where
        K: ToPropertyKey,
    {
        let key = key.to_property_key(self.context).encode();
        let attributes = Attribute::WRITABLE | Attribute::CONFIGURABLE;
        self.map.entry(
            key,
            &format!(
                "({}, Attribute::from_bits_retain({}))",
                self.slot_index,
                attributes.bits()
            ),
        );
        self.slot_index += 1;
        self
    }

    fn accessor<K>(&mut self, key: K, mut attributes: Attribute) -> &mut Self
    where
        K: ToPropertyKey,
    {
        // TODO: should they always be set?
        attributes |= Attribute::GET;
        attributes |= Attribute::SET;

        let key = key.to_property_key(self.context).encode();
        self.map.entry(
            key,
            &format!(
                "({}, Attribute::from_bits_retain({}))",
                self.slot_index,
                attributes.bits()
            ),
        );
        self.slot_index += 2;
        self
    }

    fn property<K>(&mut self, key: K, attributes: Attribute) -> &mut Self
    where
        K: ToPropertyKey,
    {
        assert!(!attributes.contains(Attribute::GET) && !attributes.contains(Attribute::SET));

        let key = key.to_property_key(self.context).encode();
        self.map.entry(
            key,
            &format!(
                "({}, Attribute::from_bits_retain({}))",
                self.slot_index,
                attributes.bits()
            ),
        );
        self.slot_index += 1;
        self
    }

    fn build(&mut self, file: &mut BufWriter<File>) -> io::Result<&'static str> {
        let prototype = if let Some(prototype) = self.prototype {
            format!("Some(&'static {})", prototype)
        } else {
            "None".into()
        };
        writeln!(
            file,
            "pub static {}_STATIC_SHAPE: StaticShape = StaticShape {{\n    storage_len: {},\n    prototype: {},\n    property_table: {} }};",
            self.name,
            self.slot_index + 1,
            prototype,
            self.map.build(),
        )?;

        Ok(self.name)
    }
}

struct BuiltInBuilderConstructor<'a> {
    object: BuiltInBuilder<'a>,
    prototype: BuiltInBuilder<'a>,
}

impl<'a> BuiltInBuilderConstructor<'a> {
    fn new(context: &'a Context, name: &'static str) -> Self {
        Self::with_constructor_attributes(
            context,
            name,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
    }

    fn with_constructor_attributes(
        context: &'a Context,
        name: &'static str,
        constructor_attributes: Attribute,
    ) -> Self {
        let object_name = Box::leak(format!("{name}_CONSTRUCTOR").into_boxed_str());
        let prototype_name = Box::leak(format!("{name}_PROTOTYPE").into_boxed_str());
        let mut this = Self {
            object: BuiltInBuilder::new(context, object_name),
            prototype: BuiltInBuilder::new(context, prototype_name),
        };

        this.object
            .property(utf16!("length"), Attribute::CONFIGURABLE);
        this.object
            .property(utf16!("name"), Attribute::CONFIGURABLE);
        this.object
            .property(utf16!("prototype"), Attribute::empty());

        this.prototype
            .property(utf16!("constructor"), constructor_attributes);

        this
    }

    fn inherits(&mut self, prototype: &'static str) -> &mut Self {
        self.object.inherits(prototype);
        self
    }

    fn method<K>(&mut self, key: K) -> &mut Self
    where
        K: ToPropertyKey,
    {
        self.prototype.method(key);
        self
    }

    fn static_method<K>(&mut self, key: K) -> &mut Self
    where
        K: ToPropertyKey,
    {
        self.object.method(key);
        self
    }

    fn accessor<K>(&mut self, key: K, attributes: Attribute) -> &mut Self
    where
        K: ToPropertyKey,
    {
        self.prototype.accessor(key, attributes);
        self
    }

    fn static_accessor<K>(&mut self, key: K, attributes: Attribute) -> &mut Self
    where
        K: ToPropertyKey,
    {
        self.object.accessor(key, attributes);
        self
    }

    fn static_property<K>(&mut self, key: K, attributes: Attribute) -> &mut Self
    where
        K: ToPropertyKey,
    {
        self.object.property(key, attributes);
        self
    }

    fn property<K>(&mut self, key: K, attributes: Attribute) -> &mut Self
    where
        K: ToPropertyKey,
    {
        self.prototype.property(key, attributes);
        self
    }

    fn build(&mut self, file: &mut BufWriter<File>) -> io::Result<()> {
        self.object.build(file)?;
        self.prototype.build(file)?;

        Ok(())
    }
}

fn main() -> io::Result<()> {
    // TODO: split into separate files
    // TODO: Move common parts between build and lib.rs into common.rs file.

    // TODO: because the generated static shapes for builtin file does not change that often and it's not that big,
    //       it's kept as rust source code which changes only in certain places when a property is added or removed.
    //       We could properly cache this into the git history, to avoid generating it on a fresh build.
    //       (even though it's fast to build).

    let file = Path::new(&env::var("OUT_DIR").unwrap()).join("static_shapes_codegen.rs");
    let file = &mut BufWriter::new(File::create(file)?);

    let context = Context::new();

    for string in RAW_STATICS {
        context.insert_or_get(string);
    }

    BuiltInBuilder::new(&context, "EMPTY_OBJECT").build(file)?;

    BuiltInBuilder::new(&context, "JSON_OBJECT")
        .method(utf16!("parse"))
        .method(utf16!("stringify"))
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilder::new(&context, "MATH_OBJECT")
        .property(utf16!("E"), Attribute::empty())
        .property(utf16!("LN10"), Attribute::empty())
        .property(utf16!("LN2"), Attribute::empty())
        .property(utf16!("LOG10E"), Attribute::empty())
        .property(utf16!("LOG2E"), Attribute::empty())
        .property(utf16!("PI"), Attribute::empty())
        .property(utf16!("SQRT1_2"), Attribute::empty())
        .property(utf16!("SQRT2"), Attribute::empty())
        .method(utf16!("abs"))
        .method(utf16!("acos"))
        .method(utf16!("acosh"))
        .method(utf16!("asin"))
        .method(utf16!("asinh"))
        .method(utf16!("atan"))
        .method(utf16!("atanh"))
        .method(utf16!("atan2"))
        .method(utf16!("cbrt"))
        .method(utf16!("ceil"))
        .method(utf16!("clz32"))
        .method(utf16!("cos"))
        .method(utf16!("cosh"))
        .method(utf16!("exp"))
        .method(utf16!("expm1"))
        .method(utf16!("floor"))
        .method(utf16!("fround"))
        .method(utf16!("hypot"))
        .method(utf16!("imul"))
        .method(utf16!("log"))
        .method(utf16!("log1p"))
        .method(utf16!("log10"))
        .method(utf16!("log2"))
        .method(utf16!("max"))
        .method(utf16!("min"))
        .method(utf16!("pow"))
        .method(utf16!("random"))
        .method(utf16!("round"))
        .method(utf16!("sign"))
        .method(utf16!("sin"))
        .method(utf16!("sinh"))
        .method(utf16!("sqrt"))
        .method(utf16!("tan"))
        .method(utf16!("tanh"))
        .method(utf16!("trunc"))
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilder::new(&context, "REFLECT_OBJECT")
        .method(utf16!("apply"))
        .method(utf16!("construct"))
        .method(utf16!("defineProperty"))
        .method(utf16!("deleteProperty"))
        .method(utf16!("get"))
        .method(utf16!("getOwnPropertyDescriptor"))
        .method(utf16!("getPrototypeOf"))
        .method(utf16!("has"))
        .method(utf16!("isExtensible"))
        .method(utf16!("ownKeys"))
        .method(utf16!("preventExtensions"))
        .method(utf16!("set"))
        .method(utf16!("setPrototypeOf"))
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "OBJECT")
        .accessor(utf16!("__proto__"), Attribute::CONFIGURABLE)
        .method(utf16!("hasOwnProperty"))
        .method(utf16!("propertyIsEnumerable"))
        .method(utf16!("toString"))
        .method(utf16!("toLocaleString"))
        .method(utf16!("valueOf"))
        .method(utf16!("isPrototypeOf"))
        .method(utf16!("__defineGetter__"))
        .method(utf16!("__defineSetter__"))
        .method(utf16!("__lookupGetter__"))
        .method(utf16!("__lookupSetter__"))
        .static_method(utf16!("create"))
        .static_method(utf16!("setPrototypeOf"))
        .static_method(utf16!("getPrototypeOf"))
        .static_method(utf16!("defineProperty"))
        .static_method(utf16!("defineProperties"))
        .static_method(utf16!("assign"))
        .static_method(utf16!("is"))
        .static_method(utf16!("keys"))
        .static_method(utf16!("values"))
        .static_method(utf16!("entries"))
        .static_method(utf16!("seal"))
        .static_method(utf16!("isSealed"))
        .static_method(utf16!("freeze"))
        .static_method(utf16!("isFrozen"))
        .static_method(utf16!("preventExtensions"))
        .static_method(utf16!("isExtensible"))
        .static_method(utf16!("getOwnPropertyDescriptor"))
        .static_method(utf16!("getOwnPropertyDescriptors"))
        .static_method(utf16!("getOwnPropertyNames"))
        .static_method(utf16!("getOwnPropertySymbols"))
        .static_method(utf16!("hasOwn"))
        .static_method(utf16!("fromEntries"))
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "FUNCTION")
        .property(utf16!("length"), Attribute::CONFIGURABLE)
        .property(utf16!("name"), Attribute::CONFIGURABLE)
        .method(utf16!("apply"))
        .method(utf16!("bind"))
        .method(utf16!("call"))
        .method(utf16!("toString"))
        .property(WellKnown::HasInstance, Attribute::empty())
        .accessor(utf16!("caller"), Attribute::CONFIGURABLE)
        .accessor(utf16!("arguments"), Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "ARRAY")
        .property(utf16!("length"), Attribute::WRITABLE)
        .property(
            utf16!("values"),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .property(
            WellKnown::Iterator,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .property(WellKnown::Unscopables, Attribute::CONFIGURABLE)
        .method(utf16!("at"))
        .method(utf16!("concat"))
        .method(utf16!("push"))
        .method(utf16!("indexOf"))
        .method(utf16!("lastIndexOf"))
        .method(utf16!("includes"))
        .method(utf16!("map"))
        .method(utf16!("fill"))
        .method(utf16!("forEach"))
        .method(utf16!("filter"))
        .method(utf16!("pop"))
        .method(utf16!("join"))
        .method(utf16!("toString"))
        .method(utf16!("reverse"))
        .method(utf16!("shift"))
        .method(utf16!("unshift"))
        .method(utf16!("every"))
        .method(utf16!("find"))
        .method(utf16!("findIndex"))
        .method(utf16!("findLast"))
        .method(utf16!("findLastIndex"))
        .method(utf16!("flat"))
        .method(utf16!("flatMap"))
        .method(utf16!("slice"))
        .method(utf16!("some"))
        .method(utf16!("sort"))
        .method(utf16!("splice"))
        .method(utf16!("toLocaleString"))
        .method(utf16!("reduce"))
        .method(utf16!("reduceRight"))
        .method(utf16!("keys"))
        .method(utf16!("entries"))
        .method(utf16!("copyWithin"))
        // Static properties
        .static_accessor(WellKnown::Species, Attribute::CONFIGURABLE)
        .static_method(utf16!("from"))
        .static_method(utf16!("isArray"))
        .static_method(utf16!("of"))
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "DATE")
        .static_method(utf16!("now"))
        .static_method(utf16!("parse"))
        .static_method(utf16!("UTC"))
        .method(utf16!("getDate"))
        .method(utf16!("getDay"))
        .method(utf16!("getFullYear"))
        .method(utf16!("getHours"))
        .method(utf16!("getMilliseconds"))
        .method(utf16!("getMinutes"))
        .method(utf16!("getMonth"))
        .method(utf16!("getSeconds"))
        .method(utf16!("getTime"))
        .method(utf16!("getTimezoneOffset"))
        .method(utf16!("getUTCDate"))
        .method(utf16!("getUTCDay"))
        .method(utf16!("getUTCFullYear"))
        .method(utf16!("getUTCHours"))
        .method(utf16!("getUTCMilliseconds"))
        .method(utf16!("getUTCMinutes"))
        .method(utf16!("getUTCMonth"))
        .method(utf16!("getUTCSeconds"))
        .method(utf16!("getYear"))
        .method(utf16!("setDate"))
        .method(utf16!("setFullYear"))
        .method(utf16!("setHours"))
        .method(utf16!("setMilliseconds"))
        .method(utf16!("setMinutes"))
        .method(utf16!("setMonth"))
        .method(utf16!("setSeconds"))
        .method(utf16!("setTime"))
        .method(utf16!("setUTCDate"))
        .method(utf16!("setUTCFullYear"))
        .method(utf16!("setUTCHours"))
        .method(utf16!("setUTCMilliseconds"))
        .method(utf16!("setUTCMinutes"))
        .method(utf16!("setUTCMonth"))
        .method(utf16!("setUTCSeconds"))
        .method(utf16!("setYear"))
        .method(utf16!("toDateString"))
        .method(utf16!("toISOString"))
        .method(utf16!("toJSON"))
        .method(utf16!("toLocaleDateString"))
        .method(utf16!("toLocaleString"))
        .method(utf16!("toLocaleTimeString"))
        .method(utf16!("toString"))
        .method(utf16!("toTimeString"))
        .method(utf16!("valueOf"))
        .property(
            utf16!("toGMTString"),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .property(
            utf16!("toUTCString"),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .property(WellKnown::ToPrimitive, Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "NUMBER")
        .static_property(utf16!("EPSILON"), Attribute::empty())
        .static_property(utf16!("MAX_SAFE_INTEGER"), Attribute::empty())
        .static_property(utf16!("MIN_SAFE_INTEGER"), Attribute::empty())
        .static_property(utf16!("MAX_VALUE"), Attribute::empty())
        .static_property(utf16!("MIN_VALUE"), Attribute::empty())
        .static_property(utf16!("NEGATIVE_INFINITY"), Attribute::empty())
        .static_property(utf16!("POSITIVE_INFINITY"), Attribute::empty())
        .static_property(utf16!("NaN"), Attribute::empty())
        .static_property(
            utf16!("parseInt"),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .static_property(
            utf16!("parseFloat"),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .static_method(utf16!("isFinite"))
        .static_method(utf16!("isNaN"))
        .static_method(utf16!("isSafeInteger"))
        .static_method(utf16!("isInteger"))
        .method(utf16!("toExponential"))
        .method(utf16!("toFixed"))
        .method(utf16!("toLocaleString"))
        .method(utf16!("toPrecision"))
        .method(utf16!("toString"))
        .method(utf16!("valueOf"))
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "BOOLEAN")
        .method(utf16!("toString"))
        .method(utf16!("valueOf"))
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "BIGINT")
        .method(utf16!("toString"))
        .method(utf16!("valueOf"))
        .static_method(utf16!("asIntN"))
        .static_method(utf16!("asUintN"))
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "SYMBOL")
        .static_method(utf16!("for"))
        .static_method(utf16!("keyFor"))
        .static_property(utf16!("asyncIterator"), Attribute::empty())
        .static_property(utf16!("hasInstance"), Attribute::empty())
        .static_property(utf16!("isConcatSpreadable"), Attribute::empty())
        .static_property(utf16!("iterator"), Attribute::empty())
        .static_property(utf16!("match"), Attribute::empty())
        .static_property(utf16!("matchAll"), Attribute::empty())
        .static_property(utf16!("replace"), Attribute::empty())
        .static_property(utf16!("search"), Attribute::empty())
        .static_property(utf16!("species"), Attribute::empty())
        .static_property(utf16!("split"), Attribute::empty())
        .static_property(utf16!("toPrimitive"), Attribute::empty())
        .static_property(utf16!("toStringTag"), Attribute::empty())
        .static_property(utf16!("unscopables"), Attribute::empty())
        .method(utf16!("toString"))
        .method(utf16!("valueOf"))
        .accessor(utf16!("description"), Attribute::CONFIGURABLE)
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .property(WellKnown::ToPrimitive, Attribute::CONFIGURABLE)
        .build(file)?;

    let mut builder = BuiltInBuilderConstructor::new(&context, "STRING");
    builder
        .property(utf16!("length"), Attribute::empty())
        .property(
            utf16!("trimStart"),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .property(
            utf16!("trimEnd"),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .static_method(utf16!("raw"))
        .static_method(utf16!("fromCharCode"))
        .static_method(utf16!("fromCodePoint"))
        .method(utf16!("charAt"))
        .method(utf16!("charCodeAt"))
        .method(utf16!("codePointAt"))
        .method(utf16!("toString"))
        .method(utf16!("concat"))
        .method(utf16!("repeat"))
        .method(utf16!("slice"))
        .method(utf16!("startsWith"))
        .method(utf16!("endsWith"))
        .method(utf16!("includes"))
        .method(utf16!("indexOf"))
        .method(utf16!("lastIndexOf"))
        .method(utf16!("localeCompare"))
        .method(utf16!("match"))
        .method(utf16!("normalize"))
        .method(utf16!("padEnd"))
        .method(utf16!("padStart"))
        .method(utf16!("trim"))
        .method(utf16!("toLowerCase"))
        .method(utf16!("toUpperCase"))
        .method(utf16!("toLocaleLowerCase"))
        .method(utf16!("toLocaleUpperCase"))
        .method(utf16!("substring"))
        .method(utf16!("split"))
        .method(utf16!("valueOf"))
        .method(utf16!("matchAll"))
        .method(utf16!("replace"))
        .method(utf16!("replaceAll"))
        .method(WellKnown::Iterator)
        .method(utf16!("search"))
        .method(utf16!("at"));

    #[cfg(feature = "annex-b")]
    {
        builder
            .property(
                utf16!("trimLeft"),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                utf16!("trimRight"),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .method(utf16!("substr"))
            .method(utf16!("anchor"))
            .method(utf16!("big"))
            .method(utf16!("blink"))
            .method(utf16!("bold"))
            .method(utf16!("fixed"))
            .method(utf16!("fontcolor"))
            .method(utf16!("fontsize"))
            .method(utf16!("italics"))
            .method(utf16!("link"))
            .method(utf16!("small"))
            .method(utf16!("strike"))
            .method(utf16!("sub"))
            .method(utf16!("sup"));
    }
    builder.build(file)?;

    let mut regexp = BuiltInBuilderConstructor::new(&context, "REGEXP");
    regexp
        .static_accessor(WellKnown::Species, Attribute::CONFIGURABLE)
        .property(
            utf16!("lastIndex"),
            Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .method(utf16!("test"))
        .method(utf16!("exec"))
        .method(utf16!("toString"))
        .method(WellKnown::Match)
        .method(WellKnown::MatchAll)
        .method(WellKnown::Replace)
        .method(WellKnown::Search)
        .method(WellKnown::Split)
        .accessor(utf16!("hasIndices"), Attribute::CONFIGURABLE)
        .accessor(utf16!("global"), Attribute::CONFIGURABLE)
        .accessor(utf16!("ignoreCase"), Attribute::CONFIGURABLE)
        .accessor(utf16!("multiline"), Attribute::CONFIGURABLE)
        .accessor(utf16!("dotAll"), Attribute::CONFIGURABLE)
        .accessor(utf16!("unicode"), Attribute::CONFIGURABLE)
        .accessor(utf16!("sticky"), Attribute::CONFIGURABLE)
        .accessor(utf16!("flags"), Attribute::CONFIGURABLE)
        .accessor(utf16!("source"), Attribute::CONFIGURABLE);

    #[cfg(feature = "annex-b")]
    regexp.method(utf16!("compile"));

    regexp.build(file)?;

    let attribute = Attribute::WRITABLE | Attribute::CONFIGURABLE;
    BuiltInBuilderConstructor::new(&context, "ERROR")
        .property(utf16!("name"), attribute)
        .property(utf16!("message"), attribute)
        .method(utf16!("toString"))
        .build(file)?;

    let attribute = Attribute::WRITABLE | Attribute::CONFIGURABLE;
    BuiltInBuilderConstructor::new(&context, "NATIVE_ERROR")
        .property(utf16!("name"), attribute)
        .property(utf16!("message"), attribute)
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "MAP")
        .static_accessor(WellKnown::Species, Attribute::CONFIGURABLE)
        .property(
            utf16!("entries"),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .property(
            WellKnown::Iterator,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .method(utf16!("clear"))
        .method(utf16!("delete"))
        .method(utf16!("forEach"))
        .method(utf16!("get"))
        .method(utf16!("has"))
        .method(utf16!("keys"))
        .method(utf16!("set"))
        .method(utf16!("values"))
        .accessor(utf16!("size"), Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "SET")
        .static_accessor(WellKnown::Species, Attribute::CONFIGURABLE)
        .method(utf16!("add"))
        .method(utf16!("clear"))
        .method(utf16!("delete"))
        .method(utf16!("entries"))
        .method(utf16!("forEach"))
        .method(utf16!("has"))
        .property(
            utf16!("keys"),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .accessor(utf16!("size"), Attribute::CONFIGURABLE)
        .property(
            utf16!("values"),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .property(
            WellKnown::Iterator,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "TYPED_ARRAY")
        .static_accessor(WellKnown::Species, Attribute::CONFIGURABLE)
        .property(
            WellKnown::Iterator,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .accessor(utf16!("buffer"), Attribute::CONFIGURABLE)
        .accessor(utf16!("byteLength"), Attribute::CONFIGURABLE)
        .accessor(utf16!("byteOffset"), Attribute::CONFIGURABLE)
        .accessor(utf16!("length"), Attribute::CONFIGURABLE)
        .accessor(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .static_method(utf16!("from"))
        .static_method(utf16!("of"))
        .method(utf16!("at"))
        .method(utf16!("copyWithin"))
        .method(utf16!("entries"))
        .method(utf16!("every"))
        .method(utf16!("fill"))
        .method(utf16!("filter"))
        .method(utf16!("find"))
        .method(utf16!("findIndex"))
        .method(utf16!("forEach"))
        .method(utf16!("includes"))
        .method(utf16!("indexOf"))
        .method(utf16!("join"))
        .method(utf16!("keys"))
        .method(utf16!("lastIndexOf"))
        .method(utf16!("map"))
        .method(utf16!("reduce"))
        .method(utf16!("reduceRight"))
        .method(utf16!("reverse"))
        .method(utf16!("set"))
        .method(utf16!("slice"))
        .method(utf16!("some"))
        .method(utf16!("sort"))
        .method(utf16!("subarray"))
        .method(utf16!("values"))
        // 23.2.3.29 %TypedArray%.prototype.toString ( )
        // The initial value of the %TypedArray%.prototype.toString data property is the same
        // built-in function object as the Array.prototype.toString method defined in 23.1.3.30.
        .method(utf16!("toString"))
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "TYPED_ARRAY_INSTANCE")
        .static_accessor(WellKnown::Species, Attribute::CONFIGURABLE)
        .property(utf16!("BYTES_PER_ELEMENT"), Attribute::empty())
        .static_property(utf16!("BYTES_PER_ELEMENT"), Attribute::empty())
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "ARRAY_BUFFER")
        .static_accessor(WellKnown::Species, Attribute::CONFIGURABLE)
        .static_method(utf16!("isView"))
        .accessor(utf16!("byteLength"), Attribute::CONFIGURABLE)
        .method(utf16!("slice"))
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "DATA_VIEW")
        .accessor(utf16!("buffer"), Attribute::CONFIGURABLE)
        .accessor(utf16!("byteLength"), Attribute::CONFIGURABLE)
        .accessor(utf16!("byteOffset"), Attribute::CONFIGURABLE)
        .method(utf16!("getBigInt64"))
        .method(utf16!("getBigUint64"))
        .method(utf16!("getFloat32"))
        .method(utf16!("getFloat64"))
        .method(utf16!("getInt8"))
        .method(utf16!("getInt16"))
        .method(utf16!("getInt32"))
        .method(utf16!("getUint8"))
        .method(utf16!("getUint16"))
        .method(utf16!("getUint32"))
        .method(utf16!("setBigInt64"))
        .method(utf16!("setBigUint64"))
        .method(utf16!("setFloat32"))
        .method(utf16!("setFloat64"))
        .method(utf16!("setInt8"))
        .method(utf16!("setInt16"))
        .method(utf16!("setInt32"))
        .method(utf16!("setUint8"))
        .method(utf16!("setUint16"))
        .method(utf16!("setUint32"))
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "WEAK_REF")
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .method(utf16!("deref"))
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "PROMISE")
        .static_method(utf16!("all"))
        .static_method(utf16!("allSettled"))
        .static_method(utf16!("any"))
        .static_method(utf16!("race"))
        .static_method(utf16!("reject"))
        .static_method(utf16!("resolve"))
        .static_accessor(WellKnown::Species, Attribute::CONFIGURABLE)
        .method(utf16!("then"))
        .method(utf16!("catch"))
        .method(utf16!("finally"))
        // <https://tc39.es/ecma262/#sec-promise.prototype-@@tostringtag>
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "WEAK_MAP")
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .method(utf16!("delete"))
        .method(utf16!("get"))
        .method(utf16!("has"))
        .method(utf16!("set"))
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "WEAK_SET")
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .method(utf16!("add"))
        .method(utf16!("delete"))
        .method(utf16!("has"))
        .build(file)?;

    BuiltInBuilderConstructor::with_constructor_attributes(
        &context,
        "GENERATOR_FUNCTION",
        Attribute::CONFIGURABLE,
    )
    // .inherits(Some(
    //     realm.intrinsics().constructors().function().prototype(),
    // ))
    .property(utf16!("prototype"), Attribute::CONFIGURABLE)
    .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
    .build(file)?;

    BuiltInBuilder::new(&context, "ITERATOR_PROTOTYPE")
        .method(WellKnown::Iterator)
        .build(file)?;

    BuiltInBuilder::new(&context, "COMMON_ITERATOR_PROTOTYPE")
        .method(utf16!("next"))
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilder::new(&context, "FOR_IN_ITERATOR_PROTOTYPE")
        .method(utf16!("next"))
        .build(file)?;

    BuiltInBuilder::new(&context, "ASYNC_ITERATOR_PROTOTYPE")
        .method(WellKnown::AsyncIterator)
        .build(file)?;

    BuiltInBuilder::new(&context, "ASYNC_FROM_SYNC_ITERATOR_PROTOTYPE")
        .method(utf16!("next"))
        .method(utf16!("return"))
        .method(utf16!("throw"))
        .build(file)?;

    BuiltInBuilder::new(&context, "THROW_TYPE_ERROR_OBJECT")
        .property(utf16!("length"), Attribute::empty())
        .property(utf16!("name"), Attribute::empty())
        .build(file)?;

    BuiltInBuilder::new(&context, "GENERATOR_OBJECT")
        .method(utf16!("next"))
        .method(utf16!("return"))
        .method(utf16!("throw"))
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .property(utf16!("constructor"), Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilderConstructor::new(&context, "ASYNC_FUNCTION")
        .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
        .build(file)?;

    BuiltInBuilderConstructor::with_constructor_attributes(
        &context,
        "ASYNC_GENERATOR_FUNCTION",
        Attribute::CONFIGURABLE,
    )
    .property(utf16!("prototype"), Attribute::CONFIGURABLE)
    .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
    .build(file)?;

    #[cfg(feature = "intl")]
    {
        BuiltInBuilder::new(&context, "INTL_OBJECT")
            .property(
                WellKnown::ToStringTag,
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                utf16!("Collator"),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                utf16!("ListFormat"),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                utf16!("Locale"),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                utf16!("Segmenter"),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                utf16!("DateTimeFormat"),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .method(utf16!("getCanonicalLocales"))
            .build(file)?;

        BuiltInBuilderConstructor::new(&context, "DATE_TIME_FORMAT").build(file)?;

        BuiltInBuilderConstructor::new(&context, "COLLATOR")
            .static_method(utf16!("supportedLocalesOf"))
            .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
            .accessor(utf16!("compare"), Attribute::CONFIGURABLE)
            .method(utf16!("resolvedOptions"))
            .build(file)?;

        BuiltInBuilderConstructor::new(&context, "INTL_SEGMENTER")
            .static_method(utf16!("supportedLocalesOf"))
            .property(WellKnown::ToStringTag, Attribute::CONFIGURABLE)
            .method(utf16!("resolvedOptions"))
            .method(utf16!("segment"))
            .build(file)?;

        BuiltInBuilder::new(&context, "SEGMENTS_PROTOTYPE")
            .method(utf16!("containing"))
            .method(WellKnown::Iterator)
            .build(file)?;
    }

    context.build(file)?;

    Ok(())
}

/// Array of raw static strings that aren't reference counted.
///
/// The macro `static_strings` automatically sorts the array of strings, making it faster
/// for searches by using `binary_search`.
const RAW_STATICS: &[&[u16]] = &[
    utf16!(""),
    // Misc
    utf16!(","),
    utf16!(":"),
    // Generic use
    utf16!("name"),
    utf16!("length"),
    utf16!("arguments"),
    utf16!("prototype"),
    utf16!("constructor"),
    utf16!("return"),
    utf16!("throw"),
    utf16!("global"),
    utf16!("globalThis"),
    // typeof
    utf16!("null"),
    utf16!("undefined"),
    utf16!("number"),
    utf16!("string"),
    utf16!("symbol"),
    utf16!("bigint"),
    utf16!("object"),
    utf16!("function"),
    // Property descriptor
    utf16!("value"),
    utf16!("get"),
    utf16!("set"),
    utf16!("writable"),
    utf16!("enumerable"),
    utf16!("configurable"),
    // Object object
    utf16!("Object"),
    utf16!("assign"),
    utf16!("create"),
    utf16!("toString"),
    utf16!("valueOf"),
    utf16!("is"),
    utf16!("seal"),
    utf16!("isSealed"),
    utf16!("freeze"),
    utf16!("isFrozen"),
    utf16!("isExtensible"),
    utf16!("hasOwnProperty"),
    utf16!("isPrototypeOf"),
    utf16!("setPrototypeOf"),
    utf16!("getPrototypeOf"),
    utf16!("defineProperty"),
    utf16!("defineProperties"),
    utf16!("deleteProperty"),
    utf16!("construct"),
    utf16!("hasOwn"),
    utf16!("ownKeys"),
    utf16!("keys"),
    utf16!("values"),
    utf16!("entries"),
    utf16!("fromEntries"),
    // Function object
    utf16!("Function"),
    utf16!("apply"),
    utf16!("bind"),
    utf16!("call"),
    // Generator object
    utf16!("Generator"),
    // Array object
    utf16!("Array"),
    utf16!("at"),
    utf16!("from"),
    utf16!("isArray"),
    utf16!("of"),
    utf16!("copyWithin"),
    utf16!("every"),
    utf16!("fill"),
    utf16!("filter"),
    utf16!("find"),
    utf16!("findIndex"),
    utf16!("findLast"),
    utf16!("findLastIndex"),
    utf16!("flat"),
    utf16!("flatMap"),
    utf16!("forEach"),
    utf16!("includes"),
    utf16!("indexOf"),
    utf16!("join"),
    utf16!("map"),
    utf16!("next"),
    utf16!("reduce"),
    utf16!("reduceRight"),
    utf16!("reverse"),
    utf16!("shift"),
    utf16!("slice"),
    utf16!("splice"),
    utf16!("some"),
    utf16!("sort"),
    utf16!("unshift"),
    utf16!("push"),
    utf16!("pop"),
    // String object
    utf16!("String"),
    utf16!("charAt"),
    utf16!("charCodeAt"),
    utf16!("codePointAt"),
    utf16!("concat"),
    utf16!("endsWith"),
    utf16!("fromCharCode"),
    utf16!("fromCodePoint"),
    utf16!("lastIndexOf"),
    utf16!("match"),
    utf16!("matchAll"),
    utf16!("normalize"),
    utf16!("padEnd"),
    utf16!("padStart"),
    utf16!("raw"),
    utf16!("repeat"),
    utf16!("replace"),
    utf16!("replaceAll"),
    utf16!("search"),
    utf16!("split"),
    utf16!("startsWith"),
    utf16!("substr"),
    utf16!("substring"),
    utf16!("toLocaleString"),
    utf16!("toLowerCase"),
    utf16!("toUpperCase"),
    utf16!("trim"),
    utf16!("trimEnd"),
    utf16!("trimStart"),
    // Number object
    utf16!("Number"),
    utf16!("Infinity"),
    utf16!("NaN"),
    utf16!("parseInt"),
    utf16!("parseFloat"),
    utf16!("isFinite"),
    utf16!("isNaN"),
    utf16!("EPSILON"),
    utf16!("MAX_SAFE_INTEGER"),
    utf16!("MIN_SAFE_INTEGER"),
    utf16!("MAX_VALUE"),
    utf16!("MIN_VALUE"),
    utf16!("isSafeInteger"),
    utf16!("isInteger"),
    utf16!("toExponential"),
    utf16!("toFixed"),
    utf16!("toPrecision"),
    // Boolean object
    utf16!("Boolean"),
    // BigInt object
    utf16!("BigInt"),
    utf16!("asIntN"),
    utf16!("asUintN"),
    // RegExp object
    utf16!("RegExp"),
    utf16!("exec"),
    utf16!("test"),
    utf16!("flags"),
    utf16!("index"),
    utf16!("lastIndex"),
    utf16!("hasIndices"),
    utf16!("ignoreCase"),
    utf16!("multiline"),
    utf16!("dotAll"),
    utf16!("unicode"),
    utf16!("sticky"),
    utf16!("source"),
    utf16!("get hasIndices"),
    utf16!("get global"),
    utf16!("get ignoreCase"),
    utf16!("get multiline"),
    utf16!("get dotAll"),
    utf16!("get unicode"),
    utf16!("get sticky"),
    utf16!("get flags"),
    utf16!("get source"),
    // Symbol object
    utf16!("Symbol"),
    utf16!("for"),
    utf16!("keyFor"),
    utf16!("description"),
    utf16!("asyncIterator"),
    utf16!("hasInstance"),
    utf16!("species"),
    utf16!("unscopables"),
    utf16!("iterator"),
    utf16!("toStringTag"),
    utf16!("toPrimitive"),
    utf16!("get description"),
    // Map object
    utf16!("Map"),
    utf16!("clear"),
    utf16!("delete"),
    utf16!("has"),
    utf16!("size"),
    // Set object
    utf16!("Set"),
    utf16!("add"),
    // Reflect object
    utf16!("Reflect"),
    // Proxy object
    utf16!("Proxy"),
    utf16!("revocable"),
    // Error objects
    utf16!("Error"),
    utf16!("AggregateError"),
    utf16!("TypeError"),
    utf16!("RangeError"),
    utf16!("SyntaxError"),
    utf16!("ReferenceError"),
    utf16!("EvalError"),
    utf16!("ThrowTypeError"),
    utf16!("URIError"),
    utf16!("message"),
    // Date object
    utf16!("Date"),
    utf16!("toJSON"),
    utf16!("getDate"),
    utf16!("getDay"),
    utf16!("getFullYear"),
    utf16!("getHours"),
    utf16!("getMilliseconds"),
    utf16!("getMinutes"),
    utf16!("getMonth"),
    utf16!("getSeconds"),
    utf16!("getTime"),
    utf16!("getYear"),
    utf16!("getUTCDate"),
    utf16!("getUTCDay"),
    utf16!("getUTCFullYear"),
    utf16!("getUTCHours"),
    utf16!("getUTCMinutes"),
    utf16!("getUTCMonth"),
    utf16!("getUTCSeconds"),
    utf16!("setDate"),
    utf16!("setFullYear"),
    utf16!("setHours"),
    utf16!("setMilliseconds"),
    utf16!("setMinutes"),
    utf16!("setMonth"),
    utf16!("setSeconds"),
    utf16!("setYear"),
    utf16!("setTime"),
    utf16!("setUTCDate"),
    utf16!("setUTCFullYear"),
    utf16!("setUTCHours"),
    utf16!("setUTCMinutes"),
    utf16!("setUTCMonth"),
    utf16!("setUTCSeconds"),
    utf16!("toDateString"),
    utf16!("toGMTString"),
    utf16!("toISOString"),
    utf16!("toTimeString"),
    utf16!("toUTCString"),
    utf16!("now"),
    utf16!("UTC"),
    // JSON object
    utf16!("JSON"),
    utf16!("parse"),
    utf16!("stringify"),
    // Iterator object
    utf16!("Array Iterator"),
    utf16!("Set Iterator"),
    utf16!("String Iterator"),
    utf16!("Map Iterator"),
    utf16!("For In Iterator"),
    // Math object
    utf16!("Math"),
    utf16!("LN10"),
    utf16!("LN2"),
    utf16!("LOG10E"),
    utf16!("LOG2E"),
    utf16!("PI"),
    utf16!("SQRT1_2"),
    utf16!("SQRT2"),
    utf16!("abs"),
    utf16!("acos"),
    utf16!("acosh"),
    utf16!("asin"),
    utf16!("asinh"),
    utf16!("atan"),
    utf16!("atanh"),
    utf16!("atan2"),
    utf16!("cbrt"),
    utf16!("ceil"),
    utf16!("clz32"),
    utf16!("cos"),
    utf16!("cosh"),
    utf16!("exp"),
    utf16!("expm1"),
    utf16!("floor"),
    utf16!("fround"),
    utf16!("hypot"),
    utf16!("imul"),
    utf16!("log"),
    utf16!("log1p"),
    utf16!("log10"),
    utf16!("log2"),
    utf16!("max"),
    utf16!("min"),
    utf16!("pow"),
    utf16!("random"),
    utf16!("round"),
    utf16!("sign"),
    utf16!("sin"),
    utf16!("sinh"),
    utf16!("sqrt"),
    utf16!("tan"),
    utf16!("tanh"),
    utf16!("trunc"),
    // Intl object
    utf16!("Intl"),
    utf16!("DateTimeFormat"),
    // TypedArray object
    utf16!("TypedArray"),
    utf16!("ArrayBuffer"),
    utf16!("Int8Array"),
    utf16!("Uint8Array"),
    utf16!("Int16Array"),
    utf16!("Uint16Array"),
    utf16!("Int32Array"),
    utf16!("Uint32Array"),
    utf16!("BigInt64Array"),
    utf16!("BigUint64Array"),
    utf16!("Float32Array"),
    utf16!("Float64Array"),
    utf16!("buffer"),
    utf16!("byteLength"),
    utf16!("byteOffset"),
    utf16!("isView"),
    utf16!("subarray"),
    utf16!("get byteLength"),
    utf16!("get buffer"),
    utf16!("get byteOffset"),
    utf16!("get size"),
    utf16!("get length"),
    // DataView object
    utf16!("DataView"),
    utf16!("getBigInt64"),
    utf16!("getBigUint64"),
    utf16!("getFloat32"),
    utf16!("getFloat64"),
    utf16!("getInt8"),
    utf16!("getInt16"),
    utf16!("getInt32"),
    utf16!("getUint8"),
    utf16!("getUint16"),
    utf16!("getUint32"),
    utf16!("setBigInt64"),
    utf16!("setBigUint64"),
    utf16!("setFloat32"),
    utf16!("setFloat64"),
    utf16!("setInt8"),
    utf16!("setInt16"),
    utf16!("setInt32"),
    utf16!("setUint8"),
    utf16!("setUint16"),
    utf16!("setUint32"),
    // Console object
    utf16!("console"),
    utf16!("assert"),
    utf16!("debug"),
    utf16!("error"),
    utf16!("info"),
    utf16!("trace"),
    utf16!("warn"),
    utf16!("exception"),
    utf16!("count"),
    utf16!("countReset"),
    utf16!("group"),
    utf16!("groupCollapsed"),
    utf16!("groupEnd"),
    utf16!("time"),
    utf16!("timeLog"),
    utf16!("timeEnd"),
    utf16!("dir"),
    utf16!("dirxml"),
    // Minified name
    utf16!("a"),
    utf16!("b"),
    utf16!("c"),
    utf16!("d"),
    utf16!("e"),
    utf16!("f"),
    utf16!("g"),
    utf16!("h"),
    utf16!("i"),
    utf16!("j"),
    utf16!("k"),
    utf16!("l"),
    utf16!("m"),
    utf16!("n"),
    utf16!("o"),
    utf16!("p"),
    utf16!("q"),
    utf16!("r"),
    utf16!("s"),
    utf16!("t"),
    utf16!("u"),
    utf16!("v"),
    utf16!("w"),
    utf16!("x"),
    utf16!("y"),
    utf16!("z"),
    utf16!("A"),
    utf16!("B"),
    utf16!("C"),
    utf16!("D"),
    utf16!("E"),
    utf16!("F"),
    utf16!("G"),
    utf16!("H"),
    utf16!("I"),
    utf16!("J"),
    utf16!("K"),
    utf16!("L"),
    utf16!("M"),
    utf16!("N"),
    utf16!("O"),
    utf16!("P"),
    utf16!("Q"),
    utf16!("R"),
    utf16!("S"),
    utf16!("T"),
    utf16!("U"),
    utf16!("V"),
    utf16!("W"),
    utf16!("X"),
    utf16!("Y"),
    utf16!("Z"),
    utf16!("_"),
    utf16!("$"),
    // Well known symbols
    utf16!("Symbol.asyncIterator"),
    utf16!("[Symbol.asyncIterator]"),
    utf16!("Symbol.hasInstance"),
    utf16!("[Symbol.hasInstance]"),
    utf16!("Symbol.isConcatSpreadable"),
    utf16!("[Symbol.isConcatSpreadable]"),
    utf16!("Symbol.iterator"),
    utf16!("[Symbol.iterator]"),
    utf16!("Symbol.match"),
    utf16!("[Symbol.match]"),
    utf16!("Symbol.matchAll"),
    utf16!("[Symbol.matchAll]"),
    utf16!("Symbol.replace"),
    utf16!("[Symbol.replace]"),
    utf16!("Symbol.search"),
    utf16!("[Symbol.search]"),
    utf16!("Symbol.species"),
    utf16!("[Symbol.species]"),
    utf16!("Symbol.split"),
    utf16!("[Symbol.split]"),
    utf16!("Symbol.toPrimitive"),
    utf16!("[Symbol.toPrimitive]"),
    utf16!("Symbol.toStringTag"),
    utf16!("[Symbol.toStringTag]"),
    utf16!("Symbol.unscopables"),
    utf16!("[Symbol.unscopables]"),
];
