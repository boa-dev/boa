use boa_gc::{custom_trace, Finalize, GcRefCell, Trace};

use crate::{JsNativeError, JsObject, JsResult, JsValue};

use super::PoisonableEnvironment;

#[derive(Debug, Trace, Finalize)]
pub(crate) struct FunctionEnvironment {
    inner: PoisonableEnvironment,
    slots: FunctionSlots,
}

impl FunctionEnvironment {
    /// Creates a new `FunctionEnvironment`.
    pub(crate) fn new(bindings: usize, poisoned: bool, with: bool, slots: FunctionSlots) -> Self {
        Self {
            inner: PoisonableEnvironment::new(bindings, poisoned, with),
            slots,
        }
    }

    /// Gets the slots of this function environment.
    pub(crate) const fn slots(&self) -> &FunctionSlots {
        &self.slots
    }

    /// Gets the `poisonable_environment` of this function environment.
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

    /// `BindThisValue`
    ///
    /// Sets the given value as the `this` binding of the environment.
    /// Returns `false` if the `this` binding has already been initialized.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bindthisvalue
    pub(crate) fn bind_this_value(&self, value: JsObject) -> JsResult<()> {
        let mut this = self.slots.this.borrow_mut();
        match &*this {
            ThisBindingStatus::Lexical => {
                unreachable!("1. Assert: envRec.[[ThisBindingStatus]] is not lexical.")
            }
            ThisBindingStatus::Initialized(_) => {
                // 2. If envRec.[[ThisBindingStatus]] is initialized, throw a ReferenceError exception.
                return Err(JsNativeError::reference()
                    .with_message("cannot reinitialize `this` binding")
                    .into());
            }
            ThisBindingStatus::Uninitialized => {
                // 3. Set envRec.[[ThisValue]] to V.
                // 4. Set envRec.[[ThisBindingStatus]] to initialized.
                *this = ThisBindingStatus::Initialized(value.into());
            }
        }

        // 5. Return V.
        Ok(())
    }

    /// `HasSuperBinding`
    ///
    /// Returns `true` if the environment has a `super` binding.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-hassuperbinding
    ///
    /// # Panics
    ///
    /// Panics if the function object of the environment is not a function.
    pub(crate) fn has_super_binding(&self) -> bool {
        // 1.If envRec.[[ThisBindingStatus]] is lexical, return false.
        if matches!(&*self.slots.this.borrow(), ThisBindingStatus::Lexical) {
            return false;
        }

        // 2. If envRec.[[FunctionObject]].[[HomeObject]] is undefined, return false; otherwise, return true.
        self.slots
            .function_object
            .borrow()
            .as_function()
            .expect("function object must be function")
            .get_home_object()
            .is_some()
    }

    /// `HasThisBinding`
    ///
    /// Returns `true` if the environment has a `this` binding.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-hasthisbinding
    pub(crate) fn has_this_binding(&self) -> bool {
        // 1. If envRec.[[ThisBindingStatus]] is lexical, return false; otherwise, return true.
        !matches!(&*self.slots.this.borrow(), ThisBindingStatus::Lexical)
    }

    /// `GetThisBinding`
    ///
    /// Returns the `this` binding of the current environment.
    ///
    /// Differs slightly from the spec where lexical this (arrow functions) doesn't get asserted,
    /// but instead is returned as `None`.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-getthisbinding
    pub(crate) fn get_this_binding(&self) -> JsResult<Option<JsValue>> {
        match &*self.slots.this.borrow() {
            ThisBindingStatus::Lexical => Ok(None),
            // 2. If envRec.[[ThisBindingStatus]] is uninitialized, throw a ReferenceError exception.
            ThisBindingStatus::Uninitialized => Err(JsNativeError::reference()
                .with_message(
                    "Must call super constructor in derived \
                class before accessing 'this' or returning from derived constructor",
                )
                .into()),
            // 3. Return envRec.[[ThisValue]].
            ThisBindingStatus::Initialized(this) => Ok(Some(this.clone())),
        }
    }
}

/// Describes the status of a `this` binding in function environments.
#[derive(Clone, Debug, Finalize)]
pub(crate) enum ThisBindingStatus {
    /// Function doesn't have a `this` binding. (arrow functions and async arrow functions)
    Lexical,
    /// Function has a `this` binding, but is uninitialized. (derived constructors)
    Uninitialized,
    /// Funciton has an initialized `this` binding. (base constructors and most callable objects)
    Initialized(JsValue),
}

unsafe impl Trace for ThisBindingStatus {
    custom_trace!(this, {
        match this {
            Self::Initialized(obj) => mark(obj),
            Self::Lexical | Self::Uninitialized => {}
        }
    });
}

/// Holds the internal slots of a function environment.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) struct FunctionSlots {
    /// The `[[ThisValue]]` and `[[ThisBindingStatus]]`  internal slots.
    this: GcRefCell<ThisBindingStatus>,

    /// The `[[FunctionObject]]` internal slot.
    function_object: JsObject,

    /// The `[[NewTarget]]` internal slot.
    new_target: Option<JsObject>,
}

impl FunctionSlots {
    /// Creates a new `FunctionSluts`.
    pub(crate) fn new(
        this: ThisBindingStatus,
        function_object: JsObject,
        new_target: Option<JsObject>,
    ) -> Self {
        Self {
            this: GcRefCell::new(this),
            function_object,
            new_target,
        }
    }

    /// Returns the value of the `[[FunctionObject]]` internal slot.
    pub(crate) const fn function_object(&self) -> &JsObject {
        &self.function_object
    }

    /// Returns the value of the `[[NewTarget]]` internal slot.
    pub(crate) const fn new_target(&self) -> Option<&JsObject> {
        self.new_target.as_ref()
    }
}
