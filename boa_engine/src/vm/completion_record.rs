use crate::{vm::CompletionType, JsError, JsResult, JsValue};

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
}

impl From<CompletionRecord> for JsResult<JsValue> {
    fn from(val: CompletionRecord) -> Self {
        match val.completion_type {
            CompletionType::Throw => Err(JsError::from_opaque(val.value)),
            _ => Ok(val.value),
        }
    }
}
