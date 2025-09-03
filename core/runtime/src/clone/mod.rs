//! Module containing all types and functions to implement `structuredClone`.
//!
//! See <https://developer.mozilla.org/en-US/docs/Web/API/Window/structuredClone>.
#![allow(clippy::needless_pass_by_value)]

use boa_engine::realm::Realm;
use boa_engine::value::TryFromJs;
use boa_engine::{Context, JsObject, JsResult};
use boa_interop::boa_macros::boa_module;

/// Options used by `structuredClone`. This is currently unused.
#[derive(Debug, Clone, TryFromJs)]
pub struct StructuredCloneOptions {
    transfer: Option<Vec<JsObject>>,
}

/// JavaScript module containing the `structuredClone` types and functions.
#[boa_module]
pub mod js_module {
    use super::StructuredCloneOptions;
    use crate::store::JsValueStore;
    use boa_engine::value::TryIntoJs;
    use boa_engine::{Context, JsResult, JsValue};

    /// The [`structuredClone()`][mdn] method of the Window interface creates a
    /// deep clone of a given value using the [structured clone algorithm][sca].
    ///
    /// # Errors
    /// Will return an error if the context cannot create objects or copy bytes, or
    /// if any unhandled case by the structured clone algorithm.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Window/structuredClone
    /// [sca]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm
    pub fn structured_clone(
        value: JsValue,
        options: Option<StructuredCloneOptions>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let v = JsValueStore::try_from_js(
            &value,
            context,
            options.and_then(|o| o.transfer).unwrap_or_default(),
        )?;
        v.try_into_js(context)
    }
}

/// Register the `structuredClone` function in the global context.
///
/// # Errors
/// Return an error if the function is already registered.
pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
    js_module::boa_register(realm, context)
}
