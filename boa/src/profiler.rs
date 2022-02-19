#![allow(missing_copy_implementations, missing_debug_implementations)]

#[cfg(feature = "profiler")]
use measureme::{EventId, Profiler, TimingGuard};
#[cfg(feature = "profiler")]
use once_cell::sync::OnceCell;
use std::fmt::{self, Debug};
#[cfg(feature = "profiler")]
use std::{
    path::Path,
    thread::{current, ThreadId},
};

#[cfg(feature = "profiler")]
pub struct BoaProfiler {
    profiler: Profiler,
}

/// This static instance should never be public, and its only access should be done through the `global()` and `drop()` methods
/// This is because `get_or_init` manages synchronisation and the case of an empty value
#[cfg(feature = "profiler")]
static mut INSTANCE: OnceCell<BoaProfiler> = OnceCell::new();

#[cfg(feature = "profiler")]
impl BoaProfiler {
    pub fn start_event(&self, label: &str, category: &str) -> TimingGuard<'_> {
        let kind = self.profiler.alloc_string(category);
        let id = EventId::from_label(self.profiler.alloc_string(label));
        let thread_id = Self::thread_id_to_u32(current().id());
        self.profiler
            .start_recording_interval_event(kind, id, thread_id)
    }

    pub fn default() -> Self {
        let profiler = Profiler::new(Path::new("./my_trace")).expect("must be able to create file");
        Self { profiler }
    }

    pub fn global() -> &'static Self {
        unsafe { INSTANCE.get_or_init(Self::default) }
    }

    pub fn drop(&self) {
        // In order to drop the INSTANCE we need to get ownership of it, which isn't possible on a static unless you make it a mutable static
        // mutating statics is unsafe, so we need to wrap it as so.
        // This is actually safe though because init and drop are only called at the beginning and end of the application
        unsafe {
            INSTANCE
                .take()
                .expect("Could not take back profiler instance");
        }
    }

    // Sadly we need to use the unsafe method until this is resolved:
    // https://github.com/rust-lang/rust/issues/67939
    // Once `as_64()` is in stable we can do this:
    // https://github.com/rust-lang/rust/pull/68531/commits/ea42b1c5b85f649728e3a3b334489bac6dce890a
    // Until then our options are: use rust-nightly or use unsafe {}
    fn thread_id_to_u32(tid: ThreadId) -> u32 {
        unsafe { std::mem::transmute::<ThreadId, u64>(tid) as u32 }
    }
}

impl Debug for BoaProfiler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt("no debug implemented", f)
    }
}

#[cfg(not(feature = "profiler"))]
pub struct BoaProfiler;

#[allow(clippy::unused_unit, clippy::unused_self)]
#[cfg(not(feature = "profiler"))]
impl BoaProfiler {
    pub fn start_event(&self, _label: &str, _category: &str) -> () {}

    pub fn drop(&self) {}

    pub fn global() -> Self {
        Self
    }
}
