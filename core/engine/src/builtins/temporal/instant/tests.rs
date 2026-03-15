use crate::{JsNativeErrorKind, TestAction, run_test_actions};

#[test]
#[cfg(feature = "intl_bundled")]
fn instant_to_locale_string() {
    run_test_actions([TestAction::assert(
        "typeof new Temporal.Instant(0n).toLocaleString() === 'string'",
    )]);
}

#[test]
#[cfg(feature = "intl_bundled")]
fn instant_to_locale_string_invalid_this() {
    run_test_actions([TestAction::assert_native_error(
        "Temporal.Instant.prototype.toLocaleString.call({})",
        JsNativeErrorKind::Type,
        "the this object must be a Temporal.Instant object.",
    )]);
}
