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
struct SyncWrapper(MutationContext<'static, 'static>);
#[cfg(feature = "oscars_backend")]
unsafe impl Sync for SyncWrapper {}
#[cfg(feature = "oscars_backend")]
unsafe impl Send for SyncWrapper {}

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
        static DUMMY: std::sync::LazyLock<SyncWrapper> =
            std::sync::LazyLock::new(|| SyncWrapper(MutationContext::global()));
        &DUMMY.0
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
        let mc = unsafe { crate::MutationContext::global() };
        crate::Gc::new(&mc, value)
    }

    #[must_use]
    pub fn gc_collector(&self) -> &'static crate::MutationContext<'static, 'static> {
        static DUMMY: SyncWrapperDefault =
            SyncWrapperDefault(unsafe { crate::MutationContext::global() });
        &DUMMY.0
    }
}
