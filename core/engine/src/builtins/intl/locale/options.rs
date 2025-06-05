use icu_locale::{
    extensions::unicode::Value,
    preferences::extensions::unicode::keywords::HourCycle,
    subtags::{Language, Region, Script},
};

use crate::{builtins::options::OptionType, Context, JsNativeError};

impl OptionType for Value {
    fn from_value(value: crate::JsValue, context: &mut Context) -> crate::JsResult<Self> {
        let val = value.to_string(context)?.to_std_string_escaped();

        if val.len() < 3 {
            return Err(JsNativeError::range()
                .with_message("nonterminal `type` must be at least 3 characters long")
                .into());
        }

        val.parse::<Self>()
            .map_err(|e| JsNativeError::range().with_message(e.to_string()).into())
    }
}

impl OptionType for Language {
    fn from_value(value: crate::JsValue, context: &mut Context) -> crate::JsResult<Self> {
        value
            .to_string(context)?
            .to_std_string_escaped()
            .parse::<Self>()
            .map_err(|e| JsNativeError::range().with_message(e.to_string()).into())
    }
}

impl OptionType for Script {
    fn from_value(value: crate::JsValue, context: &mut Context) -> crate::JsResult<Self> {
        value
            .to_string(context)?
            .to_std_string_escaped()
            .parse::<Self>()
            .map_err(|e| JsNativeError::range().with_message(e.to_string()).into())
    }
}

impl OptionType for Region {
    fn from_value(value: crate::JsValue, context: &mut Context) -> crate::JsResult<Self> {
        value
            .to_string(context)?
            .to_std_string_escaped()
            .parse::<Self>()
            .map_err(|e| JsNativeError::range().with_message(e.to_string()).into())
    }
}

impl OptionType for HourCycle {
    fn from_value(value: crate::JsValue, context: &mut Context) -> crate::JsResult<Self> {
        let value = Value::from_value(value, context)?;
        HourCycle::try_from(&value).map_err(|e| {
            JsNativeError::range()
                .with_message("invalid keyword value for `hc`")
                .into()
        })
    }
}
