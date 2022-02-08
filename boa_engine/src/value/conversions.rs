use super::{Display, JsValue};

impl From<&Self> for JsValue {
    #[inline]
    fn from(value: &Self) -> Self {
        value.clone()
    }
}

impl From<char> for JsValue {
    #[inline]
    fn from(value: char) -> Self {
        Self::new(value.to_string())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TryFromCharError;

impl Display for TryFromCharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert value to a char type")
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TryFromObjectError;

impl Display for TryFromObjectError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert value to an Object type")
    }
}

impl<T> From<Option<T>> for JsValue
where
    T: Into<Self>,
{
    #[inline]
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Self::undefined(),
        }
    }
}
