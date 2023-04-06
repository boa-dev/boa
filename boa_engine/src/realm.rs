//! Boa's implementation of ECMAScript's `Realm Records`
//!
//! Conceptually, a realm consists of a set of intrinsic objects, an ECMAScript global environment,
//! all of the ECMAScript code that is loaded within the scope of that global environment,
//! and other associated state and resources.
//!
//! A realm is represented in this implementation as a Realm struct with the fields specified from the spec.

use crate::{
    context::{intrinsics::Intrinsics, HostHooks},
    environments::DeclarativeEnvironment,
    object::JsObject,
};
use boa_gc::{Finalize, Gc, Trace};
use boa_profiler::Profiler;

/// Representation of a Realm.
///
/// In the specification these are called Realm Records.
#[derive(Clone, Debug, Trace, Finalize)]
pub struct Realm {
    inner: Gc<Inner>,
}

impl PartialEq for Realm {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.inner, &*other.inner)
    }
}

#[derive(Debug, Trace, Finalize)]
struct Inner {
    intrinsics: Intrinsics,
    environment: Gc<DeclarativeEnvironment>,
    global_object: JsObject,
    global_this: JsObject,
}

impl Realm {
    /// Create a new Realm.
    #[inline]
    pub fn create(hooks: &dyn HostHooks) -> Self {
        let _timer = Profiler::global().start_event("Realm::create", "realm");

        let intrinsics = Intrinsics::default();
        let global_object = hooks.create_global_object(&intrinsics);
        let global_this = hooks
            .create_global_this(&intrinsics)
            .unwrap_or_else(|| global_object.clone());

        let realm = Self {
            inner: Gc::new(Inner {
                intrinsics,
                environment: Gc::new(DeclarativeEnvironment::new_global()),
                global_object,
                global_this,
            }),
        };

        realm.initialize();

        realm
    }

    /// Gets the intrinsics of this `Realm`.
    pub fn intrinsics(&self) -> &Intrinsics {
        &self.inner.intrinsics
    }

    pub(crate) fn environment(&self) -> &Gc<DeclarativeEnvironment> {
        &self.inner.environment
    }

    pub(crate) fn global_object(&self) -> &JsObject {
        &self.inner.global_object
    }

    pub(crate) fn global_this(&self) -> &JsObject {
        &self.inner.global_this
    }

    /// Resizes the number of bindings on the global environment.
    pub(crate) fn resize_global_env(&self) {
        let binding_number = self.environment().compile_env().borrow().num_bindings();

        let mut bindings = self.environment().bindings().borrow_mut();
        if bindings.len() < binding_number {
            bindings.resize(binding_number, None);
        }
    }
}
