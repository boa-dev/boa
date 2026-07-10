//! TC55 timer APIs: `setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`.
//!
//! Spec: <https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#timers>
//!
//! # TC55 Status
//!
//! Timer APIs are required in the `WinterTC` TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `setTimeout`, `clearTimeout`, `setInterval`, `clearInterval` from `boa_runtime::interval`.

/// Register timer globals (`setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`)
/// into the given context.
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
