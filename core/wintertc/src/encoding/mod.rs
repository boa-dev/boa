//! TC55 encoding APIs: `TextEncoder`, `TextDecoder`, `TextEncoderStream`, `TextDecoderStream`.
//!
//! Spec: <https://encoding.spec.whatwg.org/>
//!
//! # TC55 Status
//!
//! `TextEncoder`, `TextDecoder`, `TextEncoderStream`, and `TextDecoderStream` are required
//! in the `WinterTC` TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `TextEncoder` and `TextDecoder` from `boa_runtime::text`.
//! - Implement `TextEncoderStream`.
//! - Implement `TextDecoderStream`.

/// Register encoding globals (`TextEncoder`, `TextDecoder`, `TextEncoderStream`,
/// `TextDecoderStream`) into the given context.
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
