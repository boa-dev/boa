use boa_temporal::error::{ErrorKind, TemporalError};

use crate::{JsError, JsNativeError};

impl From<TemporalError> for JsNativeError {
    fn from(value: TemporalError) -> Self {
        match value.kind() {
            ErrorKind::Range => JsNativeError::range().with_message(value.message()),
            ErrorKind::Type => JsNativeError::typ().with_message(value.message()),
            ErrorKind::Generic => JsNativeError::error().with_message(value.message()),
        }
    }
}

impl From<TemporalError> for JsError {
    fn from(value: TemporalError) -> Self {
        let native: JsNativeError = value.into();
        native.into()
    }
}
