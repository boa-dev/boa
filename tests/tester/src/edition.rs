//! Edition detection utilities.
//!
//! This module contains the [`SpecEdition`] struct, which is used in the tester to
//! classify all tests per minimum required ECMAScript edition.

use std::fmt::Display;

use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::read::{MetaData, TestFlag};

/// Minimum edition required by a specific feature in the `test262` repository.
static FEATURE_EDITION: phf::Map<&'static str, SpecEdition> = phf::phf_map! {
    // Proposed language features

    // Intl.Locale Info
    // https://github.com/tc39/proposal-intl-locale-info
    "Intl.Locale-info"  => SpecEdition::ESNext,

    // FinalizationRegistry#cleanupSome
    // https://github.com/tc39/proposal-cleanup-some
    "FinalizationRegistry.prototype.cleanupSome" => SpecEdition::ESNext,

    // Intl.NumberFormat V3
    // https://github.com/tc39/proposal-intl-numberformat-v3
    "Intl.NumberFormat-v3" => SpecEdition::ESNext,

    // Legacy RegExp features
    // https://github.com/tc39/proposal-regexp-legacy-features
    "legacy-regexp"  => SpecEdition::ESNext,

    // Import Attributes
    // https://github.com/tc39/proposal-import-attributes/
    "import-attributes" => SpecEdition::ESNext,

    // Import Assertions
    // https://github.com/tc39/proposal-import-assertions/
    "import-assertions"  => SpecEdition::ESNext,

    // JSON modules
    // https://github.com/tc39/proposal-json-modules
    "json-modules"  => SpecEdition::ESNext,

    // ArrayBuffer transfer
    // https://github.com/tc39/proposal-arraybuffer-transfer
    "arraybuffer-transfer" => SpecEdition::ESNext,

    // Temporal
    // https://github.com/tc39/proposal-temporal
    "Temporal" => SpecEdition::ESNext,

    // ShadowRealm, née Callable Boundary Realms
    // https://github.com/tc39/proposal-realms
    "ShadowRealm" => SpecEdition::ESNext,

    // Intl.DurationFormat
    // https://github.com/tc39/proposal-intl-duration-format
    "Intl.DurationFormat" => SpecEdition::ESNext,

    // Decorators
    // https://github.com/tc39/proposal-decorators
    "decorators" => SpecEdition::ESNext,

    // Duplicate named capturing groups
    // https://github.com/tc39/proposal-duplicate-named-capturing-groups
    "regexp-duplicate-named-groups" => SpecEdition::ESNext,

    // Array.fromAsync
    // https://github.com/tc39/proposal-array-from-async
    "Array.fromAsync" => SpecEdition::ESNext,

    // JSON.parse with source
    // https://github.com/tc39/proposal-json-parse-with-source
    "json-parse-with-source" => SpecEdition::ESNext,

    // RegExp.escape
    // https://github.com/tc39/proposal-regex-escaping
    "RegExp.escape" => SpecEdition::ESNext,

    // Regular expression modifiers
    // https://github.com/tc39/proposal-regexp-modifiers
    "regexp-modifiers" => SpecEdition::ESNext,

    // Iterator Helpers
    // https://github.com/tc39/proposal-iterator-helpers
    "iterator-helpers" => SpecEdition::ESNext,

    // Promise.try
    // https://github.com/tc39/proposal-promise-try
    "promise-try" => SpecEdition::ESNext,

    // Explicit Resource Management
    // https://github.com/tc39/proposal-explicit-resource-management
    "explicit-resource-management" => SpecEdition::ESNext,

    // Float16Array + Math.f16round
    // https://github.com/tc39/proposal-float16array
    "Float16Array" => SpecEdition::ESNext,

    // Math.sumPrecise
    // https://github.com/tc39/proposal-math-sum
    "Math.sumPrecise" => SpecEdition::ESNext,

    // Source Phase Imports
    // https://github.com/tc39/proposal-source-phase-imports
    "source-phase-imports" => SpecEdition::ESNext,
    // test262 special specifier
    "source-phase-imports-module-source" => SpecEdition::ESNext,

    // Uint8Array Base64
    // https://github.com/tc39/proposal-arraybuffer-base64
    "uint8array-base64" => SpecEdition::ESNext,

    // Atomics.pause
    // https://github.com/tc39/proposal-atomics-microwait
    "Atomics.pause" => SpecEdition::ESNext,

    // Standard language features
    "AggregateError" => SpecEdition::ES12,
    "Atomics.waitAsync"  => SpecEdition::ES15,
    "align-detached-buffer-semantics-with-web-reality" => SpecEdition::ES12,
    "arbitrary-module-namespace-names" => SpecEdition::ES13,
    "array-grouping" => SpecEdition::ES15,
    "ArrayBuffer" => SpecEdition::ES6,
    "array-find-from-last" => SpecEdition::ES14,
    "Array.prototype.at" => SpecEdition::ES13,
    "Array.prototype.flat" => SpecEdition::ES10,
    "Array.prototype.flatMap" => SpecEdition::ES10,
    "Array.prototype.includes" => SpecEdition::ES7,
    "Array.prototype.values" => SpecEdition::ES6,
    "arrow-function" => SpecEdition::ES6,
    "async-iteration" => SpecEdition::ES9,
    "async-functions" => SpecEdition::ES8,
    "Atomics" => SpecEdition::ES8,
    "BigInt" => SpecEdition::ES11,
    "caller" => SpecEdition::ES5,
    "change-array-by-copy" => SpecEdition::ES14,
    "class" => SpecEdition::ES6,
    "class-fields-private" => SpecEdition::ES13,
    "class-fields-private-in" => SpecEdition::ES13,
    "class-fields-public" => SpecEdition::ES13,
    "class-methods-private" => SpecEdition::ES13,
    "class-static-block" => SpecEdition::ES13,
    "class-static-fields-private" => SpecEdition::ES13,
    "class-static-fields-public" => SpecEdition::ES13,
    "class-static-methods-private" => SpecEdition::ES13,
    "coalesce-expression" => SpecEdition::ES11,
    "computed-property-names" => SpecEdition::ES6,
    "const" => SpecEdition::ES6,
    "cross-realm" => SpecEdition::ES6,
    "DataView" => SpecEdition::ES6,
    "DataView.prototype.getFloat32" => SpecEdition::ES6,
    "DataView.prototype.getFloat64" => SpecEdition::ES6,
    "DataView.prototype.getInt16" => SpecEdition::ES6,
    "DataView.prototype.getInt32" => SpecEdition::ES6,
    "DataView.prototype.getInt8" => SpecEdition::ES6,
    "DataView.prototype.getUint16" => SpecEdition::ES6,
    "DataView.prototype.getUint32" => SpecEdition::ES6,
    "DataView.prototype.setUint8" => SpecEdition::ES6,
    "default-parameters" => SpecEdition::ES6,
    "destructuring-assignment" => SpecEdition::ES6,
    "destructuring-binding" => SpecEdition::ES6,
    "dynamic-import" => SpecEdition::ES11,
    "error-cause" => SpecEdition::ES13,
    "exponentiation" => SpecEdition::ES7,
    "export-star-as-namespace-from-module" => SpecEdition::ES11,
    "FinalizationRegistry" => SpecEdition::ES12,
    "for-in-order" => SpecEdition::ES11,
    "for-of" => SpecEdition::ES6,
    "Float32Array" => SpecEdition::ES6,
    "Float64Array" => SpecEdition::ES6,
    "generators" => SpecEdition::ES6,
    "globalThis" => SpecEdition::ES11,
    "hashbang" => SpecEdition::ES14,
    "import.meta" => SpecEdition::ES11,
    "Int8Array" => SpecEdition::ES6,
    "Int16Array" => SpecEdition::ES6,
    "Int32Array" => SpecEdition::ES6,
    "Intl-enumeration" => SpecEdition::ES14,
    "intl-normative-optional" => SpecEdition::ES8,
    "Intl.DateTimeFormat-datetimestyle" => SpecEdition::ES12,
    "Intl.DateTimeFormat-dayPeriod" => SpecEdition::ES8,
    "Intl.DateTimeFormat-extend-timezonename" => SpecEdition::ES13,
    "Intl.DateTimeFormat-formatRange" => SpecEdition::ES12,
    "Intl.DateTimeFormat-fractionalSecondDigits" => SpecEdition::ES12,
    "Intl.DisplayNames" => SpecEdition::ES12,
    "Intl.DisplayNames-v2" => SpecEdition::ES13,
    "Intl.ListFormat" => SpecEdition::ES12,
    "Intl.Locale" => SpecEdition::ES12,
    "Intl.NumberFormat-unified" => SpecEdition::ES11,
    "Intl.RelativeTimeFormat" => SpecEdition::ES11,
    "Intl.Segmenter" => SpecEdition::ES13,
    "json-superset" => SpecEdition::ES10,
    "let" => SpecEdition::ES6,
    "logical-assignment-operators" => SpecEdition::ES12,
    "Map" => SpecEdition::ES6,
    "new.target" => SpecEdition::ES6,
    "numeric-separator-literal" => SpecEdition::ES12,
    "object-rest" => SpecEdition::ES9,
    "object-spread" => SpecEdition::ES9,
    "Object.fromEntries" => SpecEdition::ES10,
    "Object.hasOwn" => SpecEdition::ES13,
    "Object.is" => SpecEdition::ES6,
    "optional-catch-binding" => SpecEdition::ES10,
    "optional-chaining" => SpecEdition::ES11,
    "Promise" => SpecEdition::ES6,
    "Promise.allSettled" => SpecEdition::ES11,
    "Promise.any" => SpecEdition::ES12,
    "Promise.prototype.finally" => SpecEdition::ES9,
    "Proxy" => SpecEdition::ES6,
    "promise-with-resolvers" => SpecEdition::ES15,
    "proxy-missing-checks" => SpecEdition::ES6,
    "Reflect" => SpecEdition::ES6,
    "Reflect.construct" => SpecEdition::ES6,
    "Reflect.set" => SpecEdition::ES6,
    "Reflect.setPrototypeOf" => SpecEdition::ES6,
    "regexp-dotall" => SpecEdition::ES9,
    "regexp-lookbehind" => SpecEdition::ES9,
    "regexp-match-indices" => SpecEdition::ES13,
    "regexp-named-groups" => SpecEdition::ES9,
    "regexp-unicode-property-escapes" => SpecEdition::ES9,
    "regexp-v-flag" => SpecEdition::ES15,
    "resizable-arraybuffer" => SpecEdition::ES15,
    "rest-parameters" => SpecEdition::ES6,
    "Set" => SpecEdition::ES6,
    "SharedArrayBuffer" => SpecEdition::ES8,
    "string-trimming" => SpecEdition::ES10,
    "String.fromCodePoint" => SpecEdition::ES6,
    "String.prototype.at" => SpecEdition::ES13,
    "String.prototype.endsWith" => SpecEdition::ES6,
    "String.prototype.includes" => SpecEdition::ES6,
    "String.prototype.isWellFormed" => SpecEdition::ES15,
    "String.prototype.matchAll" => SpecEdition::ES11,
    "String.prototype.replaceAll" => SpecEdition::ES12,
    "String.prototype.toWellFormed" => SpecEdition::ES15,
    "String.prototype.trimEnd" => SpecEdition::ES10,
    "String.prototype.trimStart" => SpecEdition::ES10,
    "set-methods" => SpecEdition::ES15,
    "super" => SpecEdition::ES6,
    "Symbol" => SpecEdition::ES6,
    "symbols-as-weakmap-keys" => SpecEdition::ES14,
    "Symbol.asyncIterator" => SpecEdition::ES9,
    "Symbol.hasInstance" => SpecEdition::ES6,
    "Symbol.isConcatSpreadable" => SpecEdition::ES6,
    "Symbol.iterator" => SpecEdition::ES6,
    "Symbol.match" => SpecEdition::ES6,
    "Symbol.matchAll" => SpecEdition::ES11,
    "Symbol.prototype.description" => SpecEdition::ES10,
    "Symbol.replace" => SpecEdition::ES6,
    "Symbol.search" => SpecEdition::ES6,
    "Symbol.species" => SpecEdition::ES6,
    "Symbol.split" => SpecEdition::ES6,
    "Symbol.toPrimitive" => SpecEdition::ES6,
    "Symbol.toStringTag" => SpecEdition::ES6,
    "Symbol.unscopables" => SpecEdition::ES6,
    "tail-call-optimization" => SpecEdition::ES6,
    "template" => SpecEdition::ES6,
    "top-level-await" => SpecEdition::ES13,
    "TypedArray" => SpecEdition::ES6,
    "TypedArray.prototype.at" => SpecEdition::ES13,
    "u180e" => SpecEdition::ES7,
    "Uint8Array" => SpecEdition::ES6,
    "Uint16Array" => SpecEdition::ES6,
    "Uint32Array" => SpecEdition::ES6,
    "Uint8ClampedArray" => SpecEdition::ES6,
    "WeakMap" => SpecEdition::ES6,
    "WeakRef" => SpecEdition::ES12,
    "WeakSet" => SpecEdition::ES6,
    "well-formed-json-stringify" => SpecEdition::ES10,
    "__proto__" => SpecEdition::ES6,
    "__getter__" => SpecEdition::ES8,
    "__setter__" => SpecEdition::ES8,

    // Test-Harness Features

    "IsHTMLDDA" => SpecEdition::ES9,
    "host-gc-required" => SpecEdition::ES5,
};

/// List of ECMAScript editions that can be tested in the `test262` repository.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Serialize_repr,
    Deserialize_repr,
    clap::ValueEnum,
)]
#[repr(u8)]
pub(crate) enum SpecEdition {
    /// ECMAScript 5.1 Edition
    ///
    /// <https://262.ecma-international.org/5.1>
    ES5 = 5,
    /// ECMAScript 6th Edition
    ///
    /// <https://262.ecma-international.org/6.0>
    ES6,
    /// ECMAScript 7th Edition
    ///
    /// <https://262.ecma-international.org/7.0>
    ES7,
    /// ECMAScript 8th Edition
    ///
    /// <https://262.ecma-international.org/8.0>
    ES8,
    /// ECMAScript 9th Edition
    ///
    /// <https://262.ecma-international.org/9.0>
    ES9,
    /// ECMAScript 10th Edition
    ///
    /// <https://262.ecma-international.org/10.0>
    ES10,
    /// ECMAScript 11th Edition
    ///
    /// <https://262.ecma-international.org/11.0>
    ES11,
    /// ECMAScript 12th Edition
    ///
    /// <https://262.ecma-international.org/12.0>
    ES12,
    /// ECMAScript 13th Edition
    ///
    /// <https://262.ecma-international.org/13.0>
    ES13,
    /// ECMAScript 14th Edition
    ///
    /// <https://262.ecma-international.org/14.0>
    ES14,
    /// ECMAScript 15th Edition
    ///
    /// <https://262.ecma-international.org/15.0>
    ES15,
    /// The edition being worked on right now.
    ///
    /// A draft is currently available [here](https://tc39.es/ecma262).
    #[default]
    ESNext = 255,
}

impl Display for SpecEdition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ESNext => write!(f, "ECMAScript Next"),
            Self::ES5 => write!(f, "ECMAScript 5.1"),
            v => write!(f, "ECMAScript {}", v as u8),
        }
    }
}

impl SpecEdition {
    /// Gets the minimum required ECMAScript edition of a test from its metadata.
    ///
    /// If the function finds unknown features in `metadata`, returns an `Err(Vec<&str>)` containing
    /// the list of unknown features.
    pub(crate) fn from_test_metadata(metadata: &MetaData) -> Result<Self, Vec<&str>> {
        let mut min_edition = if metadata.flags.contains(&TestFlag::Async) {
            Self::ES8
        } else if metadata.flags.contains(&TestFlag::Module)
            || metadata.esid.is_some()
            || metadata.es6id.is_some()
        {
            Self::ES6
        } else {
            Self::ES5
        };

        let mut unknowns = Vec::new();
        for feature in &*metadata.features {
            let Some(feature_edition) = FEATURE_EDITION.get(feature).copied() else {
                unknowns.push(&**feature);
                continue;
            };
            min_edition = std::cmp::max(min_edition, feature_edition);
        }

        if unknowns.is_empty() {
            Ok(min_edition)
        } else {
            Err(unknowns)
        }
    }

    /// Gets an iterator of all currently available editions.
    pub(crate) fn all_editions() -> impl Iterator<Item = Self> {
        [
            Self::ES5,
            Self::ES6,
            Self::ES7,
            Self::ES8,
            Self::ES9,
            Self::ES10,
            Self::ES11,
            Self::ES12,
            Self::ES13,
            Self::ES14,
            Self::ES15,
            Self::ESNext,
        ]
        .into_iter()
    }
}
