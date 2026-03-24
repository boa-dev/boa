use std::str::FromStr;

use icu_experimental::displaynames::{Fallback, LanguageDisplay, Style};

use crate::{
    Context, JsNativeError, JsResult, JsValue,
    builtins::options::{OptionType, ParsableOptionType},
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum DisplayNamesType {
    Language,
    Region,
    Script,
    Currency,
    Calendar,
    DateTimeField,
}

#[derive(Debug)]
pub(crate) struct ParseDisplayNamesTypeError;

impl std::fmt::Display for ParseDisplayNamesTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "provided string was not `language`, `region`, `script`, `currency`, `calendar` or `dateTimeField`",
        )
    }
}

impl FromStr for DisplayNamesType {
    type Err = ParseDisplayNamesTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "language" => Ok(Self::Language),
            "region" => Ok(Self::Region),
            "script" => Ok(Self::Script),
            "currency" => Ok(Self::Currency),
            "calendar" => Ok(Self::Calendar),
            "dateTimeField" => Ok(Self::DateTimeField),
            _ => Err(ParseDisplayNamesTypeError),
        }
    }
}

impl ParsableOptionType for DisplayNamesType {}

impl OptionType for Style {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_str() {
            "narrow" => Ok(Self::Narrow),
            "short" => Ok(Self::Short),
            "long" => Ok(Self::Long),
            _ => Err(JsNativeError::range()
                .with_message("provided string was not `narrow`, `short` or `long`")
                .into()),
        }
    }
}

impl OptionType for Fallback {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_str() {
            "code" => Ok(Self::Code),
            "none" => Ok(Self::None),
            _ => Err(JsNativeError::range()
                .with_message("provided string was not `code` or `none`")
                .into()),
        }
    }
}

impl OptionType for LanguageDisplay {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_str() {
            "dialect" => Ok(Self::Dialect),
            "standard" => Ok(Self::Standard),
            _ => Err(JsNativeError::range()
                .with_message("provided string was not `dialect` or `standard`")
                .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_displaynames_type_valid() {
        assert!(matches!(
            DisplayNamesType::from_str("language"),
            Ok(DisplayNamesType::Language)
        ));
        assert!(matches!(
            DisplayNamesType::from_str("region"),
            Ok(DisplayNamesType::Region)
        ));
        assert!(matches!(
            DisplayNamesType::from_str("script"),
            Ok(DisplayNamesType::Script)
        ));
        assert!(matches!(
            DisplayNamesType::from_str("currency"),
            Ok(DisplayNamesType::Currency)
        ));
        assert!(matches!(
            DisplayNamesType::from_str("calendar"),
            Ok(DisplayNamesType::Calendar)
        ));
        assert!(matches!(
            DisplayNamesType::from_str("dateTimeField"),
            Ok(DisplayNamesType::DateTimeField)
        ));
    }
}
