use boa_gc::{Finalize, Trace};

use crate::{JsObject, JsValue};

use super::PoisonableEnvironment;

#[derive(Debug, Trace, Finalize)]
pub(crate) struct GlobalEnvironment {
    inner: PoisonableEnvironment,
    global_this: JsObject,
}

impl GlobalEnvironment {
    /// Creates a new `GlobalEnvironment`.
    pub(crate) fn new(global_this: JsObject) -> Self {
        Self {
            inner: PoisonableEnvironment::new(0, false, false),
            global_this,
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
    pub(crate) fn get(&self, index: usize) -> Option<JsValue> {
        self.inner.get(index)
    }

    /// Sets the binding value from the environment by index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    pub(crate) fn set(&self, index: usize, value: JsValue) {
        self.inner.set(index, value);
    }

    /// `GetThisBinding`
    ///
    /// Returns the `this` binding on the global environment.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-getthisbinding
    pub(crate) fn get_this_binding(&self) -> JsObject {
        self.global_this.clone()
    }
}
