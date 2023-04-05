//! Runtime and host related types.

mod hooks;
pub use hooks::{DefaultHooks, HostHooks};
#[cfg(feature = "intl")]
pub(crate) mod icu;
#[cfg(feature = "intl")]
pub use icu::BoaProvider;

use std::cell::{RefCell, RefMut};

use crate::object::JsObject;

/// A runtime context.
///
/// [`Runtime`] is a representation of global data shared between execution
/// contexts (see [`Context`]); this is roughly equivalent to what the specification defines as an
/// [**Agent**].
///
/// This is the place where [**Hosts**] can define hooks (see [`HostHooks`]) to customize the behaviour
/// of the engine for their specific implementation of ECMAScript. It's also where hosts
/// initialize the [`ICU4X`] data provider to enable `Intl` support for any `Context` constructed
/// from this runtime, provided that the `intl` feature flag is enabled.
#[cfg_attr(feature = "intl", doc = "(See [`RuntimeBuilder::icu_provider`])")]
///
/// [`Context`]: crate::Context
/// [**Agent**]: https://tc39.es/ecma262/#sec-agents
/// [**Hosts**]: https://tc39.es/ecma262/#sec-hosts-and-implementations
/// [`ICU4X`]: https://github.com/unicode-org/icu4x
pub struct Runtime<'host> {
    /// Objects kept alive by `WeakRef`s.
    kept_alive: RefCell<Vec<JsObject>>,

    /// ICU related utilities.
    #[cfg(feature = "intl")]
    icu: icu::Icu<'host>,

    /// Hooks defined by the host environment.
    host_hooks: &'host dyn HostHooks,
}

impl Default for Runtime<'_> {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl std::fmt::Debug for Runtime<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Runtime");

        debug.field("hooks", &"HostHooks");

        #[cfg(feature = "intl")]
        debug.field("icu", &self.icu);

        debug.finish_non_exhaustive()
    }
}

impl<'host> Runtime<'host> {
    /// Create a new [`RuntimeBuilder`] to specify the host
    /// hooks and other additional config.
    #[must_use]
    pub fn builder() -> RuntimeBuilder<'static, 'static> {
        RuntimeBuilder::default()
    }

    /// Gets a reference to the runtime's host hooks.
    pub fn host_hooks(&self) -> &'host dyn HostHooks {
        self.host_hooks
    }

    /// Abstract operation [`AddToKeptObjects ( object )`][add].
    ///
    /// Adds `object` to the `[[KeptAlive]]` field of the current [`surrounding agent`][agent], which
    /// is represented by the `Runtime`.
    ///
    /// [add]: https://tc39.es/ecma262/#sec-addtokeptobjects
    /// [agent]: https://tc39.es/ecma262/#sec-agents
    pub(crate) fn add_to_kept_objects(&self, object: JsObject) {
        self.kept_alive_mut().push(object);
    }

    /// Abstract operation [`ClearKeptObjects`][clear].
    ///
    /// Clears all objects maintained alive by calls to the [`AddToKeptObjects`][add] abstract
    /// operation, used within the [`WeakRef`][weak] constructor.
    ///
    /// [clear]: https://tc39.es/ecma262/#sec-clear-kept-objects
    /// [add]: https://tc39.es/ecma262/#sec-addtokeptobjects
    /// [weak]: https://tc39.es/ecma262/#sec-weak-ref-objects
    #[inline]
    pub fn clear_kept_objects(&self) {
        self.kept_alive_mut().clear();
    }

    /// Gets a mutable reference to the kept alive objects.
    ///
    /// # Panics
    ///
    /// Panics if the kept alive objects are currently borrowed.
    pub(crate) fn kept_alive_mut(&self) -> RefMut<'_, Vec<JsObject>> {
        self.kept_alive.borrow_mut()
    }

    /// Gets a reference to the runtime's ICU4X data.
    #[cfg(feature = "intl")]
    pub(crate) const fn icu(&self) -> &icu::Icu<'host> {
        &self.icu
    }
}

/// Builder for the [`Runtime`] type.
#[derive(Default)]
pub struct RuntimeBuilder<'icu, 'hooks> {
    host_hooks: Option<&'hooks dyn HostHooks>,
    #[cfg(feature = "intl")]
    icu: Option<icu::Icu<'icu>>,
    #[cfg(not(feature = "intl"))]
    icu: std::marker::PhantomData<&'icu ()>,
}

impl std::fmt::Debug for RuntimeBuilder<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = f.debug_struct("RuntimeBuilder");

        out.field("host_hooks", &"HostHooks");

        #[cfg(feature = "intl")]
        out.field("icu", &self.icu);

        out.finish()
    }
}

impl<'icu, 'hooks> RuntimeBuilder<'icu, 'hooks> {
    /// Creates a new [`ContextBuilder`] with a default empty [`Interner`]
    /// and a default [`BoaProvider`] if the `intl` feature is enabled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Provides an icu data provider to the [`Context`].
    ///
    /// This function is only available if the `intl` feature is enabled.
    ///
    /// # Errors
    ///
    /// This returns `Err` if the provided provider doesn't have the required locale information
    /// to construct both a [`LocaleCanonicalizer`] and a [`LocaleExpander`]. Note that this doesn't
    /// mean that the provider will successfully construct all `Intl` services; that check is made
    /// until the creation of an instance of a service.
    ///
    /// [`LocaleCanonicalizer`]: icu_locid_transform::LocaleCanonicalizer
    /// [`LocaleExpander`]: icu_locid_transform::LocaleExpander
    #[cfg(feature = "intl")]
    pub fn icu_provider(
        self,
        provider: BoaProvider<'_>,
    ) -> Result<RuntimeBuilder<'_, 'hooks>, icu_locid_transform::LocaleTransformError> {
        Ok(RuntimeBuilder {
            icu: Some(icu::Icu::new(provider)?),
            ..self
        })
    }

    /// Initializes the [`HostHooks`] for the context.
    ///
    /// [`Host Hooks`]: https://tc39.es/ecma262/#sec-host-hooks-summary
    #[must_use]
    pub fn host_hooks(self, host_hooks: &dyn HostHooks) -> RuntimeBuilder<'icu, '_> {
        RuntimeBuilder {
            host_hooks: Some(host_hooks),
            ..self
        }
    }

    /// Builds a new [`Context`] with the provided parameters, and defaults
    /// all missing parameters to their default values.
    #[must_use]
    pub fn build<'host>(self) -> Runtime<'host>
    where
        'icu: 'host,
        'hooks: 'host,
    {
        Runtime {
            kept_alive: RefCell::new(Vec::new()),
            #[cfg(feature = "intl")]
            icu: self.icu.unwrap_or_else(|| {
                let provider = BoaProvider::Buffer(boa_icu_provider::buffer());
                icu::Icu::new(provider).expect("Failed to initialize default icu data.")
            }),
            host_hooks: self.host_hooks.unwrap_or(&DefaultHooks),
        }
    }
}
