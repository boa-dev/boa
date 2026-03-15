//! TC55 `console` object implementation.
//!
//! Spec: <https://console.spec.whatwg.org/>
//!
//! # TC55 Status
//!
//! `console` is required in the WinterTC TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `Console` from `boa_runtime::console`.

/// Register the `console` object into the given context.
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
