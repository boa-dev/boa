use std::{fmt::Display, str::FromStr};

use crate::{object::JsObject, Context, JsNativeError, JsResult, JsValue};

/// `IntlOptions` aggregates the `locale_matcher` selector and any other object
/// property needed for `Intl` object constructors.
///
/// It is used as the type of the `options` parameter in the operation `resolve_locale`.
#[derive(Debug)]
pub(super) struct IntlOptions<O> {
    pub(super) matcher: LocaleMatcher,
    pub(super) service_options: O,
}

/// A type used as an option parameter inside the `Intl` [spec].
///
/// [spec]: https://tc39.es/ecma402
pub(super) trait OptionType: Sized {
    /// Parses a [`JsValue`] into an instance of `Self`.
    ///
    /// Roughly equivalent to the algorithm steps of [9.12.13.3-7][spec], but allows for parsing
    /// steps instead of returning a pure string, number or boolean.
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-getoption
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self>;
}

trait OptionTypeParsable: FromStr {}

impl<T: OptionTypeParsable> OptionType for T
where
    T::Err: Display,
{
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        value
            .to_string(context)?
            .to_std_string_escaped()
            .parse::<Self>()
            .map_err(|err| JsNativeError::range().with_message(err.to_string()).into())
    }
}

/// The default value passed to the [`get_option`] function.
#[derive(Debug, Copy, Clone)]
pub(super) enum GetOptionDefault<T> {
    /// Throw an error if the value is `undefined`.
    Required,
    /// Return `None` if the value is `undefined`.
    None,
    /// Return T if the value is `undefined`.
    Some(T),
}

/// Abstract operation `GetOption ( options, property, type, values, fallback )`
///
/// Extracts the value of the property named `property` from the provided `options` object,
/// converts it to the required `type`, checks whether it is one of a `List` of allowed
/// `values`, and fills in a `fallback` value if necessary. If `values` is
/// undefined, there is no fixed set of values and any is permitted.
///
/// This is a safer alternative to `GetOption`, which tries to parse from the
/// provided property a valid variant of the provided type `T`. It doesn't accept
/// a `type` parameter since the type can specify in its implementation of [`TryFrom`] whether
/// it wants to parse from a [`str`] or convert directly from a boolean or number.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-getoption
pub(super) fn get_option<T: OptionType>(
    options: &JsObject,
    property: &str,
    default: GetOptionDefault<T>,
    context: &mut Context,
) -> JsResult<Option<T>> {
    // 1. Let value be ? Get(options, property).
    let value = options.get(property, context)?;

    // 2. If value is undefined, then
    if value.is_undefined() {
        return match default {
            //     a. If default is required, throw a RangeError exception.
            GetOptionDefault::Required => Err(JsNativeError::range()
                .with_message("GetOption: option value cannot be undefined")
                .into()),
            //     b. Return default.
            GetOptionDefault::None => Ok(None),
            GetOptionDefault::Some(val) => Ok(Some(val)),
        };
    }

    // The steps 3 to 7 must be made for each `OptionType`.
    T::from_value(value, context).map(Some)
}

impl OptionType for bool {
    fn from_value(value: JsValue, _: &mut Context) -> JsResult<Self> {
        // 5. If type is "boolean", then
        //      a. Set value to ! ToBoolean(value).
        Ok(value.to_boolean())
    }
}

impl OptionType for String {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        // 6. If type is "string", then
        //      a. Set value to ? ToString(value).
        value.to_string(context).map(|s| s.to_std_string_escaped())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LocaleMatcher {
    Lookup,
    BestFit,
}

#[derive(Debug)]
pub(super) struct ParseLocaleMatcherError;

impl Display for ParseLocaleMatcherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "provided string was not `lookup` or `best fit`".fmt(f)
    }
}

impl FromStr for LocaleMatcher {
    type Err = ParseLocaleMatcherError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lookup" => Ok(Self::Lookup),
            "best fit" => Ok(Self::BestFit),
            _ => Err(ParseLocaleMatcherError),
        }
    }
}

impl OptionTypeParsable for LocaleMatcher {}

/// Abstract operation `GetNumberOption ( options, property, minimum, maximum, fallback )`
///
/// Extracts the value of the property named `property` from the provided `options`
/// object, converts it to a `Number value`, checks whether it is in the allowed range,
/// and fills in a `fallback` value if necessary.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-getnumberoption
#[allow(unused)]
pub(super) fn get_number_option(
    options: &JsObject,
    property: &str,
    minimum: f64,
    maximum: f64,
    fallback: Option<f64>,
    context: &mut Context,
) -> JsResult<Option<f64>> {
    // 1. Assert: Type(options) is Object.
    // 2. Let value be ? Get(options, property).
    let value = options.get(property, context)?;

    // 3. Return ? DefaultNumberOption(value, minimum, maximum, fallback).
    default_number_option(&value, minimum, maximum, fallback, context)
}

/// Abstract operation `DefaultNumberOption ( value, minimum, maximum, fallback )`
///
/// Converts `value` to a `Number value`, checks whether it is in the allowed range,
/// and fills in a `fallback` value if necessary.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-defaultnumberoption
#[allow(unused)]
pub(super) fn default_number_option(
    value: &JsValue,
    minimum: f64,
    maximum: f64,
    fallback: Option<f64>,
    context: &mut Context,
) -> JsResult<Option<f64>> {
    // 1. If value is undefined, return fallback.
    if value.is_undefined() {
        return Ok(fallback);
    }

    // 2. Set value to ? ToNumber(value).
    let value = value.to_number(context)?;

    // 3. If value is NaN or less than minimum or greater than maximum, throw a RangeError exception.
    if value.is_nan() || value < minimum || value > maximum {
        return Err(JsNativeError::range()
            .with_message("DefaultNumberOption: value is out of range.")
            .into());
    }

    // 4. Return floor(value).
    Ok(Some(value.floor()))
}
