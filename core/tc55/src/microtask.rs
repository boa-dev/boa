//! TC55 `queueMicrotask` implementation.
//!
//! Spec: <https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#microtask-queuing>
//!
//! # TC55 Status
//!
//! `queueMicrotask` is required in the WinterTC TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `queueMicrotask` from `boa_runtime::microtask`.

/// Register `queueMicrotask` into the given context.
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
