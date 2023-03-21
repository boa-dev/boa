use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::read::{MetaData, TestFlag};

static FEATURE_EDITION: phf::Map<&'static str, SpecEdition> = phf::phf_map! {
    // Proposed language features

    // Hashbang Grammar
    // https://github.com/tc39/proposal-hashbang
    "hashbang" => SpecEdition::ES13,

    // Intl.Locale Info
    // https://github.com/tc39/proposal-intl-locale-info
    "Intl.Locale-info"  => SpecEdition::ES13,

    // FinalizationRegistry#cleanupSome
    // https://github.com/tc39/proposal-cleanup-some
    "FinalizationRegistry.prototype.cleanupSome" => SpecEdition::ES13,

    // Intl.NumberFormat V3
    // https://github.com/tc39/proposal-intl-numberformat-v3
    "Intl.NumberFormat-v3" => SpecEdition::ES13,

    // Legacy RegExp features
    // https://github.com/tc39/proposal-regexp-legacy-features
    "legacy-regexp"  => SpecEdition::ES13,

    // Atomics.waitAsync
    // https://github.com/tc39/proposal-atomics-wait-async
    "Atomics.waitAsync"  => SpecEdition::ES13,

    // Import Assertions
    // https://github.com/tc39/proposal-import-assertions/
    "import-assertions"  => SpecEdition::ES13,

    // JSON modules
    // https://github.com/tc39/proposal-json-modules
    "json-modules"  => SpecEdition::ES13,

    // Resizable Arraybuffer
    // https://github.com/tc39/proposal-resizablearraybuffer
    "resizable-arraybuffer" => SpecEdition::ES13,

    // ArrayBuffer transfer
    // https://github.com/tc39/proposal-arraybuffer-transfer
    "arraybuffer-transfer" => SpecEdition::ES13,

    // Temporal
    // https://github.com/tc39/proposal-temporal
    "Temporal" => SpecEdition::ES13,

    // ShadowRealm, née Callable Boundary Realms
    // https://github.com/tc39/proposal-realms
    "ShadowRealm" => SpecEdition::ES13,

    // Array.prototype.findLast & Array.prototype.findLastIndex
    // https://github.com/tc39/proposal-array-find-from-last
    "array-find-from-last" => SpecEdition::ES13,

    // Array.prototype.group & Array.prototype.groupToMap
    // https://github.com/tc39/proposal-array-grouping
    "array-grouping" => SpecEdition::ES13,

    // Intl.DurationFormat
    // https://github.com/tc39/proposal-intl-duration-format
    "Intl.DurationFormat" => SpecEdition::ES13,

    // RegExp set notation + properties of strings
    // https://github.com/tc39/proposal-regexp-set-notation
    "regexp-v-flag" => SpecEdition::ES13,

    // Decorators
    // https://github.com/tc39/proposal-decorators
    "decorators" => SpecEdition::ES13,

    // Duplicate named capturing groups
    // https://github.com/tc39/proposal-duplicate-named-capturing-groups
    "regexp-duplicate-named-groups" => SpecEdition::ES13,

    // Symbols as WeakMap keys
    // https://github.com/tc39/proposal-symbols-as-weakmap-keys
    "symbols-as-weakmap-keys" => SpecEdition::ES13,

    // Array.prototype.toReversed, Array.prototype.toSorted, Array.prototype.toSpliced,
    // Array.prototype.with and the equivalent TypedArray methods.
    // https://github.com/tc39/proposal-change-array-by-copy/
    "change-array-by-copy" => SpecEdition::ES13,

    // https://tc39.es/proposal-array-from-async/
    "Array.fromAsync" => SpecEdition::ES13,

    // Well-formed Unicode strings
    // https://github.com/tc39/proposal-is-usv-string
    "String.prototype.isWellFormed" => SpecEdition::ES13,
    "String.prototype.toWellFormed" => SpecEdition::ES13,

    // Standard language features

    "AggregateError" => SpecEdition::ES13,
    "align-detached-buffer-semantics-with-web-reality" => SpecEdition::ES13,
    "arbitrary-module-namespace-names" => SpecEdition::ES13,
    "ArrayBuffer" => SpecEdition::ES6,
    "Array.prototype.at" => SpecEdition::ES13,
    "Array.prototype.flat" => SpecEdition::ES13,
    "Array.prototype.flatMap" => SpecEdition::ES13,
    "Array.prototype.values" => SpecEdition::ES6,
    "arrow-function" => SpecEdition::ES6,
    "async-iteration" => SpecEdition::ES13,
    "async-functions" => SpecEdition::ES13,
    "Atomics" => SpecEdition::ES13,
    "BigInt" => SpecEdition::ES13,
    "caller" => SpecEdition::ES5,
    "class" => SpecEdition::ES6,
    "class-fields-private" => SpecEdition::ES13,
    "class-fields-private-in" => SpecEdition::ES13,
    "class-fields-public" => SpecEdition::ES13,
    "class-methods-private" => SpecEdition::ES13,
    "class-static-block" => SpecEdition::ES13,
    "class-static-fields-private" => SpecEdition::ES13,
    "class-static-fields-public" => SpecEdition::ES13,
    "class-static-methods-private" => SpecEdition::ES13,
    "coalesce-expression" => SpecEdition::ES13,
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
    "dynamic-import" => SpecEdition::ES13,
    "error-cause" => SpecEdition::ES13,
    "export-star-as-namespace-from-module" => SpecEdition::ES13,
    "FinalizationRegistry" => SpecEdition::ES13,
    "for-in-order" => SpecEdition::ES5,
    "for-of" => SpecEdition::ES6,
    "Float32Array" => SpecEdition::ES6,
    "Float64Array" => SpecEdition::ES6,
    "generators" => SpecEdition::ES6,
    "globalThis" => SpecEdition::ES13,
    "import.meta" => SpecEdition::ES13,
    "Int8Array" => SpecEdition::ES6,
    "Int16Array" => SpecEdition::ES6,
    "Int32Array" => SpecEdition::ES6,
    "Intl-enumeration" => SpecEdition::ES13,
    "intl-normative-optional" => SpecEdition::ES13,
    "Intl.DateTimeFormat-datetimestyle" => SpecEdition::ES13,
    "Intl.DateTimeFormat-dayPeriod" => SpecEdition::ES13,
    "Intl.DateTimeFormat-extend-timezonename" => SpecEdition::ES13,
    "Intl.DateTimeFormat-formatRange" => SpecEdition::ES13,
    "Intl.DateTimeFormat-fractionalSecondDigits" => SpecEdition::ES13,
    "Intl.DisplayNames" => SpecEdition::ES13,
    "Intl.DisplayNames-v2" => SpecEdition::ES13,
    "Intl.ListFormat" => SpecEdition::ES13,
    "Intl.Locale" => SpecEdition::ES13,
    "Intl.NumberFormat-unified" => SpecEdition::ES13,
    "Intl.RelativeTimeFormat" => SpecEdition::ES13,
    "Intl.Segmenter" => SpecEdition::ES13,
    "json-superset" => SpecEdition::ES5,
    "let" => SpecEdition::ES6,
    "logical-assignment-operators" => SpecEdition::ES13,
    "Map" => SpecEdition::ES6,
    "new.target" => SpecEdition::ES6,
    "numeric-separator-literal" => SpecEdition::ES13,
    "object-rest" => SpecEdition::ES13,
    "object-spread" => SpecEdition::ES13,
    "Object.fromEntries" => SpecEdition::ES13,
    "Object.hasOwn" => SpecEdition::ES13,
    "Object.is" => SpecEdition::ES6,
    "optional-catch-binding" => SpecEdition::ES13,
    "optional-chaining" => SpecEdition::ES13,
    "Promise" => SpecEdition::ES6,
    "Promise.allSettled" => SpecEdition::ES13,
    "Promise.any" => SpecEdition::ES13,
    "Promise.prototype.finally" => SpecEdition::ES13,
    "Proxy" => SpecEdition::ES6,
    "proxy-missing-checks" => SpecEdition::ES6,
    "Reflect" => SpecEdition::ES6,
    "Reflect.construct" => SpecEdition::ES6,
    "Reflect.set" => SpecEdition::ES6,
    "Reflect.setPrototypeOf" => SpecEdition::ES6,
    "regexp-dotall" => SpecEdition::ES13,
    "regexp-lookbehind" => SpecEdition::ES13,
    "regexp-match-indices" => SpecEdition::ES13,
    "regexp-named-groups" => SpecEdition::ES13,
    "regexp-unicode-property-escapes" => SpecEdition::ES13,
    "rest-parameters" => SpecEdition::ES6,
    "Set" => SpecEdition::ES6,
    "SharedArrayBuffer" => SpecEdition::ES13,
    "string-trimming" => SpecEdition::ES5,
    "String.fromCodePoint" => SpecEdition::ES6,
    "String.prototype.at" => SpecEdition::ES13,
    "String.prototype.endsWith" => SpecEdition::ES6,
    "String.prototype.includes" => SpecEdition::ES6,
    "String.prototype.matchAll" => SpecEdition::ES13,
    "String.prototype.replaceAll" => SpecEdition::ES13,
    "String.prototype.trimEnd" => SpecEdition::ES13,
    "String.prototype.trimStart" => SpecEdition::ES13,
    "super" => SpecEdition::ES6,
    "Symbol" => SpecEdition::ES6,
    "Symbol.asyncIterator" => SpecEdition::ES13,
    "Symbol.hasInstance" => SpecEdition::ES6,
    "Symbol.isConcatSpreadable" => SpecEdition::ES6,
    "Symbol.iterator" => SpecEdition::ES6,
    "Symbol.match" => SpecEdition::ES6,
    "Symbol.matchAll" => SpecEdition::ES13,
    "Symbol.prototype.description" => SpecEdition::ES13,
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
    "u180e" => SpecEdition::ES5,
    "Uint8Array" => SpecEdition::ES6,
    "Uint16Array" => SpecEdition::ES6,
    "Uint32Array" => SpecEdition::ES6,
    "Uint8ClampedArray" => SpecEdition::ES6,
    "WeakMap" => SpecEdition::ES6,
    "WeakRef" => SpecEdition::ES13,
    "WeakSet" => SpecEdition::ES6,
    "well-formed-json-stringify" => SpecEdition::ES5,
    "__proto__" => SpecEdition::ES6,
    "__getter__" => SpecEdition::ES13,
    "__setter__" => SpecEdition::ES13,

    // Test-Harness Features

    "IsHTMLDDA" => SpecEdition::ES13,
    "host-gc-required" => SpecEdition::ES13,
};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    clap::ValueEnum,
)]
#[serde(untagged)]
pub(crate) enum SpecEdition {
    ES5 = 5,
    ES6,
    ES7,
    ES8,
    ES9,
    ES10,
    ES11,
    ES12,
    #[default]
    ES13,
}

impl Display for SpecEdition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ES5 => "ECMAScript 5",
            Self::ES6 => "ECMAScript 6",
            Self::ES7 => "ECMAScript 7",
            Self::ES8 => "ECMAScript 8",
            Self::ES9 => "ECMAScript 9",
            Self::ES10 => "ECMAScript 10",
            Self::ES11 => "ECMAScript 11",
            Self::ES12 => "ECMAScript 12",
            Self::ES13 => "ECMAScript 13",
        }
        .fmt(f)
    }
}

impl SpecEdition {
    pub(crate) fn from_test_metadata(metadata: &MetaData) -> Self {
        if metadata.es5id.is_some() {
            return Self::ES5;
        }
        if metadata.es6id.is_some() {
            return Self::ES6;
        }

        let mut min_edition = if metadata.flags.contains(&TestFlag::Async) {
            Self::ES8
        } else if metadata.flags.contains(&TestFlag::Module) {
            Self::ES6
        } else {
            Self::ES5
        };

        for feature in &*metadata.features {
            let feature_edition = FEATURE_EDITION.get(feature).copied().unwrap_or_default();

            min_edition = std::cmp::max(min_edition, feature_edition);
        }

        min_edition
    }

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
        ]
        .into_iter()
    }
}
