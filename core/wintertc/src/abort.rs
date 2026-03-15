//! TC55 `AbortController` and `AbortSignal` implementation.
//!
//! Spec: <https://dom.spec.whatwg.org/#interface-abortcontroller>
//!
//! # TC55 Status
//!
//! `AbortController` and `AbortSignal` are required in the `WinterTC` TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `AbortController` and `AbortSignal` from `boa_runtime::abort`.

/// Register `AbortController` and `AbortSignal` into the given context.
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
