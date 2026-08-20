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
thread_local! {
    static COLLECTOR: &'static oscars::collectors::mark_sweep_branded::Collector =
        Box::leak(Box::new(oscars::collectors::mark_sweep_branded::Collector::new()));

    static DUMMY: &'static MutationContext<'static, 'static> = COLLECTOR.with(|c| {
        Box::leak(Box::new(unsafe {
            MutationContext::from_collector_erased(*c)
        }))
    });

    static TRACKER: std::cell::RefCell<Option<oscars::collectors::mark_sweep_branded::Root<'static, crate::scope_tracker::HandleScopeTracker>>> = std::cell::RefCell::new(None);
}

#[cfg(feature = "oscars_backend")]
impl GcContext {
    #[must_use]
    pub fn new() -> Self {
        TRACKER.with(|tracker| {
            if tracker.borrow().is_none() {
                let mc = DUMMY.with(|dummy| *dummy);
                let handle = Gc::new(mc, crate::scope_tracker::HandleScopeTracker);
                let root = mc.root(handle).expect("Failed to root HandleScopeTracker");
                *tracker.borrow_mut() = Some(root);
            }
        });
        Self
    }

    pub fn alloc<T: crate::Trace + crate::Finalize + 'static>(&self, value: T) -> Gc<'static, T> {
        let mc = self.gc_collector();
        Gc::new(mc, value)
    }

    #[must_use]
    pub fn gc_collector(&self) -> &'static MutationContext<'static, 'static> {
        DUMMY.with(|dummy| *dummy)
    }

    pub fn force_collect(&self) {
        COLLECTOR.with(|c| c.collect());
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
