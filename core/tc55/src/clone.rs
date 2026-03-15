//! TC55 `structuredClone` implementation.
//!
//! Spec: <https://html.spec.whatwg.org/multipage/structured-data.html#dom-structuredclone>
//!
//! # TC55 Status
//!
//! `structuredClone` is required in the WinterTC TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `structuredClone` from `boa_runtime::clone`.

/// Register `structuredClone` into the given context.
///
/// # Errors
///
/// Returns a [`boa_engine::JsError`] if registration fails.
pub fn register(
    _realm: Option<boa_engine::realm::Realm>,
    _ctx: &mut boa_engine::Context,
) -> boa_engine::JsResult<()> {
    Ok(())
}
