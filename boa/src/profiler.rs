#![allow(missing_copy_implementations, missing_debug_implementations)]

use measureme::{EventId, Profiler, TimingGuard};
use once_cell::sync::OnceCell;
use std::fmt::{self, Debug};
use std::{
    path::Path,
    thread::{current, ThreadId},
};

/// MmapSerializatioSink is faster on macOS and Linux
/// but FileSerializationSink is faster on Windows
#[cfg(not(windows))]
type SerializationSink = measureme::MmapSerializationSink;
#[cfg(windows)]
type SerializationSink = measureme::FileSerializationSink;

pub struct BoaProfiler {
    profiler: Profiler<SerializationSink>,
}

pub static mut INSTANCE: OnceCell<BoaProfiler> = OnceCell::new();

impl BoaProfiler {
    pub fn start_event(&self, label: &str, category: &str) -> TimingGuard<'_, SerializationSink> {
        let kind = self.profiler.alloc_string(category);
        let id = EventId::from_label(self.profiler.alloc_string(label));
        let thread_id = Self::thread_id_to_u32(current().id());
        self.profiler
            .start_recording_interval_event(kind, id, thread_id)
    }

    pub fn default() -> BoaProfiler {
        let profiler = Profiler::new(Path::new("./my_trace")).unwrap();
        BoaProfiler { profiler }
    }

    // init creates a global instance of BoaProfiler which can be used across the whole application
    pub fn init() {
        let profiler = Self::default();
        unsafe {
            INSTANCE
                .set(profiler)
                .expect("Failed to set BoaProfiler globally");
        }
    }

    pub fn global() -> &'static BoaProfiler {
        unsafe { INSTANCE.get().expect("Profiler is not initialized") }
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
