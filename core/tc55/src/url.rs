//! TC55 URL APIs: `URL`, `URLSearchParams`.
//!
//! Spec: <https://url.spec.whatwg.org/>
//!
//! # TC55 Status
//!
//! `URL` and `URLSearchParams` are required in the WinterTC TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `URL` from `boa_runtime::url`.
//! - Implement `URLSearchParams`.

/// Register `URL` and `URLSearchParams` into the given context.
///
/// # Errors
///
/// Returns a [`boa_engine::JsError`] if registration fails.
#[cfg(feature = "url")]
pub fn register(
    _realm: Option<boa_engine::realm::Realm>,
    _ctx: &mut boa_engine::Context,
) -> boa_engine::JsResult<()> {
    Ok(())
}
