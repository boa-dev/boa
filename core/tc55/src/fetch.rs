//! TC55 Fetch API: `fetch`, `Request`, `Response`, `Headers`.
//!
//! Spec: <https://fetch.spec.whatwg.org/>
//!
//! # TC55 Status
//!
//! The Fetch API is required in the `WinterTC` TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `fetch`, `Request`, `Response`, and `Headers` from `boa_runtime::fetch`.

/// Register `fetch`, `Request`, `Response`, and `Headers` into the given context.
///
/// # Errors
///
/// Returns a [`boa_engine::JsError`] if registration fails.
#[cfg(feature = "fetch")]
pub fn register(
    _realm: Option<boa_engine::realm::Realm>,
    _ctx: &mut boa_engine::Context,
) -> boa_engine::JsResult<()> {
    Ok(())
}
