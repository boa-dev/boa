//! This module implements the global `Symbol` object.
//!
//! The data type symbol is a primitive data type.
//! The `Symbol()` function returns a value of type symbol, has static properties that expose
//! several members of built-in objects, has static methods that expose the global symbol registry,
//! and resembles a built-in object class, but is incomplete as a constructor because it does not
//! support the syntax "`new Symbol()`".
//!
//! Every symbol value returned from `Symbol()` is unique.
//!
//! More information:
//! - [MDN documentation][mdn]
//! - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-symbol-value
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol

mod rcsymbol;

use crate::{
    gc::{Finalize, Trace},
    value::RcString,
};

pub use rcsymbol::RcSymbol;

/// A structure that contains the JavaScript well known symbols.
#[derive(Debug, Clone)]
pub struct WellKnownSymbols {
    async_iterator: RcSymbol,
    has_instance: RcSymbol,
    is_concat_spreadable: RcSymbol,
    iterator: RcSymbol,
    match_: RcSymbol,
    match_all: RcSymbol,
    replace: RcSymbol,
    search: RcSymbol,
    species: RcSymbol,
    split: RcSymbol,
    to_primitive: RcSymbol,
    to_string_tag: RcSymbol,
    unscopables: RcSymbol,
}

impl WellKnownSymbols {
    pub(crate) fn new() -> (Self, u64) {
        let mut count = 0;

        let async_iterator = Symbol::new(count, Some("Symbol.asyncIterator".into())).into();
        count += 1;
        let has_instance = Symbol::new(count, Some("Symbol.hasInstance".into())).into();
        count += 1;
        let is_concat_spreadable =
            Symbol::new(count, Some("Symbol.isConcatSpreadable".into())).into();
        count += 1;
        let iterator = Symbol::new(count, Some("Symbol.iterator".into())).into();
        count += 1;
        let match_ = Symbol::new(count, Some("Symbol.match".into())).into();
        count += 1;
        let match_all = Symbol::new(count, Some("Symbol.matchAll".into())).into();
        count += 1;
        let replace = Symbol::new(count, Some("Symbol.replace".into())).into();
        count += 1;
        let search = Symbol::new(count, Some("Symbol.search".into())).into();
        count += 1;
        let species = Symbol::new(count, Some("Symbol.species".into())).into();
        count += 1;
        let split = Symbol::new(count, Some("Symbol.split".into())).into();
        count += 1;
        let to_primitive = Symbol::new(count, Some("Symbol.toPrimitive".into())).into();
        count += 1;
        let to_string_tag = Symbol::new(count, Some("Symbol.toStringTag".into())).into();
        count += 1;
        let unscopables = Symbol::new(count, Some("Symbol.unscopables".into())).into();
        count += 1;

        (
            Self {
                async_iterator,
                has_instance,
                is_concat_spreadable,
                iterator,
                match_,
                match_all,
                replace,
                search,
                species,
                split,
                to_primitive,
                to_string_tag,
                unscopables,
            },
            count,
        )
    }

    /// The `Symbol.asyncIterator` well known symbol.
    ///
    /// A method that returns the default AsyncIterator for an object.
    /// Called by the semantics of the `for-await-of` statement.
    #[inline]
    pub fn async_iterator_symbol(&self) -> RcSymbol {
        self.async_iterator.clone()
    }

    /// The `Symbol.hasInstance` well known symbol.
    ///
    /// A method that determines if a `constructor` object
    /// recognizes an object as one of the `constructor`'s instances.
    /// Called by the semantics of the instanceof operator.
    #[inline]
    pub fn has_instance_symbol(&self) -> RcSymbol {
        self.has_instance.clone()
    }

    /// The `Symbol.isConcatSpreadable` well known symbol.
    ///
    /// A Boolean valued property that if `true` indicates that
    /// an object should be flattened to its array elements
    /// by `Array.prototype.concat`.
    #[inline]
    pub fn is_concat_spreadable_symbol(&self) -> RcSymbol {
        self.is_concat_spreadable.clone()
    }

    /// The `Symbol.iterator` well known symbol.
    ///
    /// A method that returns the default Iterator for an object.
    /// Called by the semantics of the `for-of` statement.
    #[inline]
    pub fn iterator_symbol(&self) -> RcSymbol {
        self.iterator.clone()
    }

    /// The `Symbol.match` well known symbol.
    ///
    /// A regular expression method that matches the regular expression
    /// against a string. Called by the `String.prototype.match` method.
    #[inline]
    pub fn match_symbol(&self) -> RcSymbol {
        self.match_.clone()
    }

    /// The `Symbol.matchAll` well known symbol.
    ///
    /// A regular expression method that returns an iterator, that yields
    /// matches of the regular expression against a string.
    /// Called by the `String.prototype.matchAll` method.
    #[inline]
    pub fn match_all_symbol(&self) -> RcSymbol {
        self.match_all.clone()
    }

    /// The `Symbol.replace` well known symbol.
    ///
    /// A regular expression method that replaces matched substrings
    /// of a string. Called by the `String.prototype.replace` method.
    #[inline]
    pub fn replace_symbol(&self) -> RcSymbol {
        self.replace.clone()
    }

    /// The `Symbol.search` well known symbol.
    ///
    /// A regular expression method that returns the index within a
    /// string that matches the regular expression.
    /// Called by the `String.prototype.search` method.
    #[inline]
    pub fn search_symbol(&self) -> RcSymbol {
        self.search.clone()
    }

    /// The `Symbol.species` well known symbol.
    ///
    /// A function valued property that is the `constructor` function
    /// that is used to create derived objects.
    #[inline]
    pub fn species_symbol(&self) -> RcSymbol {
        self.species.clone()
    }

    /// The `Symbol.split` well known symbol.
    ///
    /// A regular expression method that splits a string at the indices
    /// that match the regular expression.
    /// Called by the `String.prototype.split` method.
    #[inline]
    pub fn split_symbol(&self) -> RcSymbol {
        self.split.clone()
    }

    /// The `Symbol.toPrimitive` well known symbol.
    ///
    /// A method that converts an object to a corresponding primitive value.
    /// Called by the `ToPrimitive` (`Value::to_primitve`) abstract operation.
    #[inline]
    pub fn to_primitive_symbol(&self) -> RcSymbol {
        self.to_primitive.clone()
    }

    /// The `Symbol.toStringTag` well known symbol.
    ///
    /// A String valued property that is used in the creation of the default
    /// string description of an object.
    /// Accessed by the built-in method `Object.prototype.toString`.
    #[inline]
    pub fn to_string_tag_symbol(&self) -> RcSymbol {
        self.to_string_tag.clone()
    }

    /// The `Symbol.unscopables` well known symbol.
    ///
    /// An object valued property whose own and inherited property names are property
    /// names that are excluded from the `with` environment bindings of the associated object.
    #[inline]
    pub fn unscopables_symbol(&self) -> RcSymbol {
        self.unscopables.clone()
    }
}

#[derive(Debug, Finalize, Trace, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    pub(crate) hash: u64,
    pub(crate) description: Option<RcString>,
}

impl Symbol {
    pub(crate) fn new(hash: u64, description: Option<RcString>) -> Self {
        Self { hash, description }
    }

    /// Returns the `Symbol`s description.
    #[inline]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the `Symbol`s hash.
    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }
}
