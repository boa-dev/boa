# Locale and Decimal Usage Patterns in BOA INTL

## 1. **Getting Language Identifier from `icu_locale::Locale`**

### Method: `language()`
The primary method to extract the language identifier from a `Locale` object:

```rust
// From: core/engine/src/builtins/intl/number_format/mod.rs:82
let lang = self.locale.language().as_str();
```

This returns the language code as a string slice. Used in `get_percent_symbol()` to determine locale-specific formatting.

### Related Locale Methods (Observed Patterns)
- `locale.language()` - Gets language identifier
- `locale.to_string()` - Converts locale to full string representation
- Methods for accessing individual locale components are used but manipulation is typically done through canonicalization

### Imports Pattern
```rust
use icu_locale::{Locale, extensions::unicode::Value};
use icu_locale::{LanguageIdentifier, Locale, LocaleCanonicalizer};
```

The `LanguageIdentifier` is also available, but `Locale` is preferred for full locale information.

---

## 2. **Decimal from `fixed_decimal` - Manipulation Patterns**

### Creation Methods
```rust
// From f64 with precision handling
Decimal::try_from_f64(x, FloatPrecision::RoundTrip)

// From string
Decimal::try_from_str(&s).ok()

// From BigInt string representation
Decimal::try_from_str(&bi.to_string())

// From integer constant
Decimal::from(100u32)  // For percent multiplication
Decimal::from(0)       // Zero value
```

### Arithmetic Operations
```rust
// Multiplication (e.g., for percent conversion)
// From: core/engine/src/builtins/intl/number_format/mod.rs:532
x = x * Decimal::from(100u32);
```

### Key Methods on Decimal
```rust
// Formatting operations
number.round_with_mode_and_increment(position, mode, multiple);
number.trim_end();
number.pad_end(min_msb);
number.trim_end_if_integer();
number.pad_start(i16::from(self.minimum_integer_digits));

// Magnitude/Exponent queries
number.nonzero_magnitude_start()           // Get MSB position
number.magnitude_range().end()             // Get magnitude end (for compact notation)

// Sign operations
number.apply_sign_display(self.sign_display);
```

### CompactDecimal Construction
```rust
// From: core/engine/src/builtins/intl/plural_rules/mod.rs:493
let exp = (*fixed.magnitude_range().end()).max(0) as u8;
let compact = CompactDecimal::from_significand_and_exponent(fixed.clone(), exp);
```

---

## 3. **Imports and Module Organization**

### Number Format Imports
```rust
use fixed_decimal::{Decimal, FloatPrecision, SignDisplay};
use fixed_decimal::{
    Decimal, FloatPrecision, RoundingIncrement as BaseMultiple, SignDisplay, SignedRoundingMode,
    UnsignedRoundingMode,
};

use icu_decimal::{
    DecimalFormatter, DecimalFormatterPreferences, FormattedDecimal,
    options::{DecimalFormatterOptions, GroupingStrategy},
    preferences::NumberingSystem,
    provider::{DecimalDigitsV1, DecimalSymbolsV1},
};

use icu_locale::{Locale, extensions::unicode::Value};
```

### Plural Rules Imports
```rust
use fixed_decimal::{CompactDecimal, Decimal, SignedRoundingMode, UnsignedRoundingMode};
use icu_locale::Locale;
```

### Locale Utilities
```rust
use icu_locale::{LanguageIdentifier, Locale, LocaleCanonicalizer};
use icu_locale::extensions::unicode::value;
```

---

## 4. **Common Usage Patterns in `core/engine/src/builtins/intl/`**

### Pattern 1: Locale Resolution
```rust
// From locale/utils.rs
let locale = resolve_locale::<Self>(
    requested_locales,
    &mut intl_options,
    context.intl_provider(),
)?;
```

### Pattern 2: Decimal Formatting with Sign Display
```rust
// From number_format/mod.rs:74-75
self.digit_options.format_fixed_decimal(value);
value.apply_sign_display(self.sign_display);
self.formatter.format(value)
```

### Pattern 3: Percent Formatting
```rust
// From number_format/mod.rs:526-532
let is_percent = nf_data.unit_options.style() == Style::Percent;

if is_percent {
    x = x * Decimal::from(100u32);
}
// ... formatting happens
if is_percent {
    format!("{}{}", formatted, nf_data.get_percent_symbol())
}
```

### Pattern 4: Compact Notation with Exponent
```rust
// From plural_rules/mod.rs:493-495
let exp = (*fixed.magnitude_range().end()).max(0) as u8;
let compact = CompactDecimal::from_significand_and_exponent(fixed.clone(), exp);
plural_rules.native.rules().category_for(&compact)
```

### Pattern 5: Decimal Construction from Numbers
```rust
// From number_format/options.rs:932
let mut number = Decimal::try_from_f64(number, FloatPrecision::RoundTrip)
    .expect("`number` must be finite");
```

---

## 5. **Error Handling Patterns**

### Decimal Parsing Errors
```rust
Decimal::try_from_str(&s)
    .map_err(|err| JsNativeError::range()
        .with_message(err.to_string()).into())
```

### Float Conversion
```rust
Decimal::try_from_f64(x, FloatPrecision::RoundTrip)
    .map_err(|err| JsNativeError::range()
        .with_message(err.to_string()).into())
```

---

## 6. **File Locations for Reference**

| File | Purpose |
|------|---------|
| `core/engine/src/builtins/intl/number_format/mod.rs` | NumberFormat class, locale language access, percent symbol lookup, Decimal multiplication |
| `core/engine/src/builtins/intl/number_format/options.rs` | DigitFormatOptions, Decimal rounding & formatting, FixedDecimal API usage |
| `core/engine/src/builtins/intl/plural_rules/mod.rs` | CompactDecimal construction, magnitude/exponent handling |
| `core/engine/src/builtins/intl/locale/utils.rs` | Locale resolution, canonicalization, language identifier extraction |
| `core/engine/src/builtins/intl/` | General INTL module structure with Service trait usage |

---

## Summary

- **Locale Language Access**: Use `locale.language().as_str()` to get the language ID as a string
- **Decimal Creation**: Prefer `try_from_f64()` for numbers or `try_from_str()` for strings
- **Decimal Arithmetic**: Simple operations via operator overloading (e.g., `*` for multiplication)
- **Decimal Formatting**: Use methods like `round_with_mode_and_increment()`, `trim_end()`, `pad_start()`
- **Exponent Access**: Use `magnitude_range()` to get exponent information for compact notation
- **Compact Decimal**: Use `CompactDecimal::from_significand_and_exponent()` with magnitude data
