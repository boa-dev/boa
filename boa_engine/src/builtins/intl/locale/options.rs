use icu_locid::extensions::unicode::Value;

use crate::{builtins::intl::options::OptionType, Context, JsNativeError};

impl OptionType for Value {
    fn from_value(value: crate::JsValue, context: &mut dyn Context<'_>) -> crate::JsResult<Self> {
        let val = value
            .to_string(context)?
            .to_std_string_escaped()
            .parse::<Self>()
            .map_err(|e| JsNativeError::range().with_message(e.to_string()))?;

        if val.as_tinystr_slice().is_empty() {
            return Err(JsNativeError::range()
                .with_message("Unicode Locale Identifier `type` cannot be empty")
                .into());
        }

        Ok(val)
    }
}
