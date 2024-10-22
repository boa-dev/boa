use temporal_rs::error::{ErrorKind, TemporalError};

use crate::{JsError, JsNativeError};

impl From<TemporalError> for JsNativeError {
    fn from(value: TemporalError) -> Self {
        match value.kind() {
            ErrorKind::Range => JsNativeError::range().with_message(value.message().to_owned()),
            ErrorKind::Type => JsNativeError::typ().with_message(value.message().to_owned()),
            ErrorKind::Generic => JsNativeError::error().with_message(value.message().to_owned()),
            ErrorKind::Syntax => JsNativeError::syntax().with_message(value.message().to_owned()),
            ErrorKind::Assert => JsNativeError::error().with_message("internal engine error"),
        }
    }
}

impl From<TemporalError> for JsError {
    fn from(value: TemporalError) -> Self {
        let native: JsNativeError = value.into();
        native.into()
    }
}
