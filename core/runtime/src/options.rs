//! Module for the register options type.

use crate::fetch::fetchers::ErrorFetcher;
use crate::{DefaultLogger, Logger, fetch};
use boa_engine::realm::Realm;

/// Create a generic type declaration based on features.
/// There is a limitation currently in Rust where you can optionally declare a
/// generic argument on the `impl<>` token tree, but not on the `StructName<>`
/// one. This makes this macro necessary.
/// Please note this might not be scalable later on as we add more providers.
/// We'll likely need a new design for passing the options to the register
/// function.
#[cfg(feature = "fetch")]
macro_rules! RegisterOptionsType {
    ($f: ident, $l: ident) => {
        RegisterOptions<$f, $l>
    }
}
#[cfg(not(feature = "fetch"))]
macro_rules! RegisterOptionsType {
    ($f: ident, $l: ident) => {
        RegisterOptions<$l>
    }
}

pub(crate) use RegisterOptionsType;

/// Options used when registering all built-in objects and functions of the `WebAPI` runtime.
#[derive(Debug)]
pub struct RegisterOptions<#[cfg(feature = "fetch")] F: fetch::Fetcher, L: Logger> {
    pub(crate) realm: Option<Realm>,
    pub(crate) console_logger: L,

    #[cfg(feature = "fetch")]
    pub(crate) fetcher: Option<F>,
}

#[cfg(feature = "fetch")]
impl Default for RegisterOptions<ErrorFetcher, DefaultLogger> {
    fn default() -> Self {
        Self {
            realm: None,
            console_logger: DefaultLogger,
            fetcher: None,
        }
    }
}

#[cfg(feature = "fetch")]
impl RegisterOptions<ErrorFetcher, DefaultLogger> {
    /// Create a new `RegisterOptions` with the default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<#[cfg(feature = "fetch")] F: fetch::Fetcher, L: Logger> RegisterOptionsType![F, L] {
    /// Set the realm to which we should register the APIs.
    #[must_use]
    pub fn with_realm(self, realm: Realm) -> Self {
        Self {
            realm: Some(realm),
            ..self
        }
    }

    /// Set the logger for the console object.
    pub fn with_console_logger<L2: Logger>(self, logger: L2) -> RegisterOptionsType![F, L2] {
        RegisterOptions::<F, L2> {
            realm: self.realm,
            console_logger: logger,
            fetcher: self.fetcher,
        }
    }
}

#[cfg(feature = "fetch")]
impl<F: fetch::Fetcher, L: Logger> RegisterOptions<F, L> {
    /// Set the fetch provider for the fetch API.
    pub fn with_fetcher<F2: fetch::Fetcher>(self, new_fetcher: F2) -> RegisterOptions<F2, L> {
        RegisterOptions::<F2, L> {
            realm: self.realm,
            fetcher: Some(new_fetcher),
            console_logger: self.console_logger,
        }
    }
}
