//! Boa's **`boa_wintertc`** crate implements the [WinterTC (TC55) Minimum Common Web API](https://min-common-api.proposal.wintertc.org/)
//! for the `boa_engine` crate.
//!
//! `WinterTC` (TC55) is an Ecma International Technical Committee working towards a baseline set
//! of Web Platform APIs that all server-side JavaScript runtimes (Deno, Bun, Cloudflare Workers,
//! Node.js, etc.) agree to implement, enabling portable server-side JavaScript.
//!
//! # Relationship to `boa_runtime`
//!
//! `boa_wintertc` is a standalone crate that depends only on `boa_engine`.
//! `boa_runtime` depends on `boa_wintertc` and re-exports its APIs, so users of `boa_runtime`
//! automatically get TC55 compliance without any extra setup.
//!
//! If you only want the TC55-mandated APIs and nothing else, depend on `boa_wintertc` directly.
//!
//! # Example: Registering all TC55 APIs
//!
//! ```no_run
//! use boa_engine::Context;
//!
//! let mut context = Context::default();
//!
//! boa_wintertc::register(None, &mut context)
//!     .expect("failed to register TC55 APIs");
//! ```
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![allow(
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate,
    clippy::let_unit_value
)]

pub mod abort;
pub mod base64;
pub mod clone;
pub mod console;
pub mod encoding;
pub mod events;
#[cfg(feature = "fetch")]
pub mod fetch;
pub mod microtask;
pub mod timers;
#[cfg(feature = "url")]
pub mod url;

/// Register all TC55-mandated Web APIs into the given [`boa_engine::Context`].
///
/// This registers the Minimum Common Web API as specified by `WinterTC` (TC55):
/// <https://min-common-api.proposal.wintertc.org/>
///
/// # Errors
///
/// Returns a [`boa_engine::JsError`] if any API fails to register (e.g. a global
/// object already exists with a conflicting name).
#[allow(clippy::needless_pass_by_value)]
pub fn register(
    realm: Option<boa_engine::realm::Realm>,
    ctx: &mut boa_engine::Context,
) -> boa_engine::JsResult<()> {
    console::register(realm.clone(), ctx)?;
    timers::register(realm.clone(), ctx)?;
    encoding::register(realm.clone(), ctx)?;
    microtask::register(realm.clone(), ctx)?;
    clone::register(realm.clone(), ctx)?;
    base64::register(realm.clone(), ctx)?;
    abort::register(realm.clone(), ctx)?;
    #[cfg(feature = "url")]
    url::register(realm.clone(), ctx)?;
    #[cfg(feature = "fetch")]
    fetch::register(realm, ctx)?;

    Ok(())
}
