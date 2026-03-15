//! TC55 `atob` / `btoa` implementation.
//!
//! Spec: <https://html.spec.whatwg.org/multipage/webappapis.html#atob>
//!
//! # TC55 Status
//!
//! `atob` and `btoa` are required in the `WinterTC` TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `atob` and `btoa` from `boa_runtime::base64`.

/// Register `atob` and `btoa` into the given context.
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
