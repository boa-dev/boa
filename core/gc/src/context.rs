#[cfg(feature = "oscars_backend")]
use oscars::collectors::mark_sweep_branded::{Gc, MutationContext};

#[cfg(feature = "oscars_backend")]
#[derive(Debug, Clone, Copy)]
pub struct GcContext;

#[cfg(feature = "oscars_backend")]
impl Default for GcContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "oscars_backend")]
impl GcContext {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn alloc<T: crate::Trace + crate::Finalize + 'static>(&self, value: T) -> Gc<'static, T> {
        // As a bridge, we use the global MutationContext until explicit
        // context threading is natively supported by the oscars backend.
        let mc = MutationContext::global();
        Gc::new(&mc, value)
    }

    #[must_use]
    pub fn gc_collector(&self) -> &MutationContext<'static, 'static> {
        // Just return a dummy global mutation context
        // This is safe for the bridge phase.
        unimplemented!("Not supported natively without closure yet, use MutationContext::global()")
    }
}

#[cfg(not(feature = "oscars_backend"))]
#[derive(Debug, Clone, Copy)]
pub struct GcContext;

#[cfg(not(feature = "oscars_backend"))]
impl Default for GcContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "oscars_backend"))]
impl GcContext {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn alloc<T: crate::Trace>(&self, value: T) -> crate::Gc<'static, T> {
        let mc = unsafe { crate::MutationContext::global() };
        crate::Gc::new(&mc, value)
    }

    #[must_use]
    pub fn gc_collector(&self) -> &crate::MutationContext<'static, 'static> {
        // Just return a dummy global mutation context
        static DUMMY: crate::MutationContext<'static, 'static> =
            unsafe { crate::MutationContext::global() };
        &DUMMY
    }
}
