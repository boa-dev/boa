mod arguments;
mod array;
mod object;
mod primitives;
mod typed_array;
mod value;

use super::{Display, JsValue, fmt};

/// This object is used for displaying a `Value`.
#[derive(Debug, Clone, Copy)]
pub struct ValueDisplay<'value> {
    pub(super) value: &'value JsValue,
    pub(super) internals: bool,
}

impl ValueDisplay<'_> {
    /// Display internal information about value.
    ///
    /// By default this is `false`.
    #[inline]
    #[must_use]
    pub const fn internals(mut self, yes: bool) -> Self {
        self.internals = yes;
        self
    }
}

impl Display for ValueDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        value::log_value_to(f, self.value, self.internals, true)
    }
}
