//! Module for the register options type.

#[cfg(feature = "fetch")]
use crate::fetch;

use crate::{DefaultLogger, Logger};
use boa_engine::realm::Realm;

/// Create a generic type declaration based on features.
/// There is a limitation currently in Rust where you can optionally declare a
/// generic argument on the `impl<>` token tree, but not on the `StructName<>`
/// one. This makes this macro necessary.
/// Please note this might not be scalable later on as we add more providers.
/// We'll likely need a new design for passing the options to the register
/// function.
#[cfg(feature = "fetch")]
macro_rules! register_options_type {
    ($f: ty, $l: ty) => {
        RegisterOptions<$f, $l>
    }
}
#[cfg(not(feature = "fetch"))]
macro_rules! register_options_type {
    ($f: ty, $l: ty) => {
        RegisterOptions<$l>
    }
}

pub(crate) use register_options_type;

/// Options used when registering all built-in objects and functions of the `WebAPI` runtime.
#[derive(Debug)]
pub struct RegisterOptions<#[cfg(feature = "fetch")] F: fetch::Fetcher, L: Logger> {
    pub(crate) realm: Option<Realm>,
    pub(crate) console_logger: L,

    #[cfg(feature = "fetch")]
    pub(crate) fetcher: Option<F>,
}

impl Default for register_options_type![fetch::ErrorFetcher, DefaultLogger] {
    fn default() -> Self {
        Self {
            realm: None,
            console_logger: DefaultLogger,
            #[cfg(feature = "fetch")]
            fetcher: None,
        }
    }
}

impl register_options_type![fetch::ErrorFetcher, DefaultLogger] {
    /// Create a new `RegisterOptions` with the default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<#[cfg(feature = "fetch")] F: fetch::Fetcher, L: Logger> register_options_type![F, L] {
    /// Set the realm to which we should register the APIs.
    #[must_use]
    pub fn with_realm(self, realm: Realm) -> Self {
        Self {
            realm: Some(realm),
            ..self
        }
    }

    /// Set the logger for the console object.
    pub fn with_console_logger<L2: Logger>(self, logger: L2) -> register_options_type![F, L2] {
        RegisterOptions {
            realm: self.realm,
            console_logger: logger,
            #[cfg(feature = "fetch")]
            fetcher: self.fetcher,
        }
    }
}

#[cfg(feature = "fetch")]
impl<F: fetch::Fetcher, L: Logger> RegisterOptions<F, L> {
    /// Set the fetch provider for the fetch API.
    pub fn with_fetcher<F2: fetch::Fetcher>(
        self,
        new_fetcher: F2,
    ) -> register_options_type![F2, L] {
        RegisterOptions {
            realm: self.realm,
            fetcher: Some(new_fetcher),
            console_logger: self.console_logger,
        }
    }
}
