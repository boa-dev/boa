use super::PoisonableEnvironment;
use crate::JsValue;
use boa_gc::{Finalize, Trace};

#[derive(Debug, Trace, Finalize)]
pub(crate) struct GlobalEnvironment {
    inner: PoisonableEnvironment,
}

impl GlobalEnvironment {
    /// Creates a new `GlobalEnvironment`.
    pub(crate) fn new() -> Self {
        Self {
            inner: PoisonableEnvironment::new(0, false, false),
        }
    }

    /// Gets the `poisonable_environment` of this global environment.
    pub(crate) const fn poisonable_environment(&self) -> &PoisonableEnvironment {
        &self.inner
    }

    /// Gets the binding value from the environment by it's index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range or not initialized.
    #[track_caller]
    pub(crate) fn get(&self, index: u32) -> Option<JsValue> {
        self.inner.get(index)
    }

    /// Sets the binding value from the environment by index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    pub(crate) fn set(&self, index: u32, value: JsValue) {
        self.inner.set(index, value);
    }
}
