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
        let mc = MutationContext::global();
        Gc::new(&mc, value)
    }

    #[must_use]
    pub fn gc_collector(&self) -> &'static MutationContext<'static, 'static> {
        thread_local! {
            static COLLECTOR: &'static oscars::collectors::mark_sweep_branded::Collector =
                Box::leak(Box::new(oscars::collectors::mark_sweep_branded::Collector::new()));

            static DUMMY: &'static MutationContext<'static, 'static> = COLLECTOR.with(|c| {
                Box::leak(Box::new(unsafe {
                    MutationContext::from_collector_erased(c)
                }))
            });
        }
        DUMMY.with(|dummy| *dummy)
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
struct SyncWrapperDefault(crate::MutationContext<'static, 'static>);
#[cfg(not(feature = "oscars_backend"))]
unsafe impl Sync for SyncWrapperDefault {}
#[cfg(not(feature = "oscars_backend"))]
unsafe impl Send for SyncWrapperDefault {}

#[cfg(not(feature = "oscars_backend"))]
impl GcContext {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn alloc<T: crate::Trace>(&self, value: T) -> crate::Gc<'static, T> {
        // SAFETY: The global mutation context is used as a fallback during the context threading migration.
        let mc = unsafe { crate::MutationContext::global() };
        crate::Gc::new(&mc, value)
    }

    #[must_use]
    pub fn gc_collector(&self) -> &'static crate::MutationContext<'static, 'static> {
        static DUMMY: SyncWrapperDefault =
            // SAFETY: The global mutation context is used as a fallback during the context threading migration.
            SyncWrapperDefault(unsafe { crate::MutationContext::global() });
        &DUMMY.0
    }
}
