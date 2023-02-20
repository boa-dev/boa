use crate::{vm::CompletionType, Context, JsError, JsResult, JsValue};

#[derive(Debug, Clone)]
pub(crate) struct CompletionRecord {
    completion_type: CompletionType,
    value: JsValue,
}

// ---- CompletionRecord creation and destruction methods ----
impl CompletionRecord {
    /// Create a new `CompletionRecord` with a provided `CompletionType` and `JsValue`.
    pub(crate) const fn new(completion_type: CompletionType, value: JsValue) -> Self {
        Self {
            completion_type,
            value,
        }
    }
}

// ---- `CompletionRecord` methods ----
impl CompletionRecord {
    pub(crate) fn is_normal_completion(&self) -> bool {
        self.completion_type == CompletionType::Normal
    }

    pub(crate) fn is_throw_completion(&self) -> bool {
        self.completion_type == CompletionType::Throw
    }

    pub(crate) fn is_return_completion(&self) -> bool {
        self.completion_type == CompletionType::Return
    }

    pub(crate) fn value(&self) -> JsValue {
        self.value.clone()
    }

    pub(crate) fn convert(self, context: &mut Context<'_>) -> JsResult<JsValue> {
        match self.completion_type {
            CompletionType::Throw => {
                let err = JsError::from_opaque(self.value);
                if let Ok(native) = err.try_native(context) {
                    Err(JsError::from_native(native))
                } else {
                    Err(err)
                }
            }
            _ => Ok(self.value),
        }
    }
}
