use crate::{JsError, JsResult, JsValue};

#[derive(Debug, Clone)]
pub(crate) enum CompletionRecord {
    Normal(JsValue),
    Return(JsValue),
    Throw(JsError),
}

// ---- `CompletionRecord` methods ----
impl CompletionRecord {
    pub(crate) const fn is_throw_completion(&self) -> bool {
        matches!(self, CompletionRecord::Throw(_))
    }

    /// This function will consume the current `CompletionRecord` and return a `JsResult<JsValue>`
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn consume(self) -> JsResult<JsValue> {
        match self {
            Self::Throw(error) => Err(error),
            Self::Normal(value) | Self::Return(value) => Ok(value),
        }
    }
}
