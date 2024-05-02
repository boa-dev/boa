//! Utilities to parse, validate and get options in builtins.

use std::{fmt, str::FromStr};

use crate::{object::JsObject, string::JsStr, Context, JsNativeError, JsResult, JsString, JsValue};

/// A type used as an option parameter for [`get_option`].
pub(crate) trait OptionType: Sized {
    /// Parses a [`JsValue`] into an instance of `Self`.
    ///
    /// Roughly equivalent to the algorithm steps of [9.12.13.3-7][spec], but allows for parsing
    /// steps instead of returning a pure string, number or boolean.
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-getoption
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self>;
}

/// A type that implements [`OptionType`] by parsing a string.
///
/// This automatically implements `OptionType` for a type if the type implements `FromStr`.
pub(crate) trait ParsableOptionType: FromStr {}

impl<T: ParsableOptionType> OptionType for T
where
    T::Err: fmt::Display,
{
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        value
            .to_string(context)?
            .to_std_string_escaped()
            .parse::<Self>()
            .map_err(|err| JsNativeError::range().with_message(err.to_string()).into())
    }
}

/// Abstract operation [`GetOption ( options, property, type, values, fallback )`][spec]
///
/// Extracts the value of the property named `property` from the provided `options` object,
/// converts it to the required `type` and checks whether it is one of a `List` of allowed
/// `values`. If `values` is undefined, there is no fixed set of values and any is permitted.
/// If the value is `undefined`, `required` would technically determine if the function should
/// return `None` or an `Err`, but handling this on the caller's side using [`Option::ok_or_else`]
/// should provide better context for error messages.
///
/// This is a safer alternative to `GetOption`, which tries to parse from the
/// provided property a valid variant of the provided type `T`. It doesn't accept
/// a `type` parameter since the type can specify in its implementation of [`OptionType`] whether
/// it wants to parse from a [`str`] or convert directly from a boolean or number.
///
/// [spec]: https://tc39.es/ecma402/#sec-getoption
pub(crate) fn get_option<T: OptionType>(
    options: &JsObject,
    property: JsStr<'_>,
    context: &mut Context,
) -> JsResult<Option<T>> {
    // 1. Let value be ? Get(options, property).
    let value = options.get(property, context)?;

    // 2. If value is undefined, then
    if value.is_undefined() {
        // a. If default is required, throw a RangeError exception.
        // b. Return default.
        return Ok(None);
    }

    // The steps 3 to 7 must be made for each `OptionType`.
    T::from_value(value, context).map(Some)
}

/// Abstract operation [`GetOptionsObject ( options )`][spec]
///
/// Returns a [`JsObject`] suitable for use with [`get_option`], either `options` itself or a
/// default empty `JsObject`. It throws a `TypeError` if `options` is not undefined and not a `JsObject`.
///
/// [spec]: https://tc39.es/ecma402/#sec-getoptionsobject
pub(crate) fn get_options_object(options: &JsValue) -> JsResult<JsObject> {
    match options {
        // If options is undefined, then
        JsValue::Undefined => {
            // a. Return OrdinaryObjectCreate(null).
            Ok(JsObject::with_null_proto())
        }
        // 2. If Type(options) is Object, then
        JsValue::Object(obj) => {
            // a. Return options.
            Ok(obj.clone())
        }
        // 3. Throw a TypeError exception.
        _ => Err(JsNativeError::typ()
            .with_message("GetOptionsObject: provided options is not an object")
            .into()),
    }
}

// Common options used in several builtins

impl OptionType for bool {
    fn from_value(value: JsValue, _: &mut Context) -> JsResult<Self> {
        // 5. If type is "boolean", then
        //      a. Set value to ! ToBoolean(value).
        Ok(value.to_boolean())
    }
}

impl OptionType for JsString {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        // 6. If type is "string", then
        //      a. Set value to ? ToString(value).
        value.to_string(context)
    }
}

impl OptionType for f64 {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        let value = value.to_number(context)?;

        if !value.is_finite() {
            return Err(JsNativeError::range()
                .with_message("roundingIncrement must be finite.")
                .into());
        }

        Ok(value)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) enum RoundingMode {
    Ceil,
    Floor,
    Expand,
    Trunc,
    HalfCeil,
    HalfFloor,
    #[default]
    HalfExpand,
    HalfTrunc,
    HalfEven,
}

impl RoundingMode {
    #[cfg(feature = "intl")]
    pub(crate) fn to_js_string(self) -> JsString {
        use crate::js_string;
        match self {
            Self::Ceil => js_string!("ceil"),
            Self::Floor => js_string!("floor"),
            Self::Expand => js_string!("expand"),
            Self::Trunc => js_string!("trunc"),
            Self::HalfCeil => js_string!("halfCeil"),
            Self::HalfFloor => js_string!("halfFloor"),
            Self::HalfExpand => js_string!("halfExpand"),
            Self::HalfTrunc => js_string!("halfTrunc"),
            Self::HalfEven => js_string!("halfEven"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ParseRoundingModeError;

impl fmt::Display for ParseRoundingModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided string was not a valid rounding mode")
    }
}

impl FromStr for RoundingMode {
    type Err = ParseRoundingModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ceil" => Ok(Self::Ceil),
            "floor" => Ok(Self::Floor),
            "expand" => Ok(Self::Expand),
            "trunc" => Ok(Self::Trunc),
            "halfCeil" => Ok(Self::HalfCeil),
            "halfFloor" => Ok(Self::HalfFloor),
            "halfExpand" => Ok(Self::HalfExpand),
            "halfTrunc" => Ok(Self::HalfTrunc),
            "halfEven" => Ok(Self::HalfEven),
            _ => Err(ParseRoundingModeError),
        }
    }
}

impl ParsableOptionType for RoundingMode {}

// TODO: remove once confirmed.
#[cfg(feature = "temporal")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnsignedRoundingMode {
    Infinity,
    Zero,
    HalfInfinity,
    HalfZero,
    HalfEven,
}

impl RoundingMode {
    // TODO: remove once confirmed.
    #[cfg(feature = "temporal")]
    #[allow(dead_code)]
    pub(crate) const fn negate(self) -> Self {
        use RoundingMode::{
            Ceil, Expand, Floor, HalfCeil, HalfEven, HalfExpand, HalfFloor, HalfTrunc, Trunc,
        };

        match self {
            Ceil => Self::Floor,
            Floor => Self::Ceil,
            HalfCeil => Self::HalfFloor,
            HalfFloor => Self::HalfCeil,
            Trunc => Self::Trunc,
            Expand => Self::Expand,
            HalfTrunc => Self::HalfTrunc,
            HalfExpand => Self::HalfExpand,
            HalfEven => Self::HalfEven,
        }
    }

    // TODO: remove once confirmed.
    #[cfg(feature = "temporal")]
    #[allow(dead_code)]
    pub(crate) const fn get_unsigned_round_mode(self, is_negative: bool) -> UnsignedRoundingMode {
        use RoundingMode::{
            Ceil, Expand, Floor, HalfCeil, HalfEven, HalfExpand, HalfFloor, HalfTrunc, Trunc,
        };

        match self {
            Ceil if !is_negative => UnsignedRoundingMode::Infinity,
            Ceil => UnsignedRoundingMode::Zero,
            Floor if !is_negative => UnsignedRoundingMode::Zero,
            Floor | Trunc | Expand => UnsignedRoundingMode::Infinity,
            HalfCeil if !is_negative => UnsignedRoundingMode::HalfInfinity,
            HalfCeil | HalfTrunc => UnsignedRoundingMode::HalfZero,
            HalfFloor if !is_negative => UnsignedRoundingMode::HalfZero,
            HalfFloor | HalfExpand => UnsignedRoundingMode::HalfInfinity,
            HalfEven => UnsignedRoundingMode::HalfEven,
        }
    }
}
