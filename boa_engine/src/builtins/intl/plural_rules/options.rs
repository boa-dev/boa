use icu_plurals::PluralRuleType;

use crate::{builtins::intl::options::OptionType, Context, JsNativeError, JsResult, JsValue};

impl OptionType for PluralRuleType {
    fn from_value(value: JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_str() {
            "cardinal" => Ok(Self::Cardinal),
            "ordinal" => Ok(Self::Ordinal),
            _ => Err(JsNativeError::range()
                .with_message("provided string was not `cardinal` or `ordinal`")
                .into()),
        }
    }
}
