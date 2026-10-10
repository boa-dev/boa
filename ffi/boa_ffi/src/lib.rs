//! C bindings for the Boa JavaScript engine.
//!
//! Rust applications should use `boa_engine` directly.

/// Entry points used by the generated C bindings.
#[diplomat::bridge]
pub mod ffi {
    use boa_engine::{Context, context::ContextBuilder};

    /// An owned Boa context, exposed as an opaque C handle.
    ///
    /// Keep the handle on the thread that created it. Destroy it exactly once
    /// with `BoaContext_destroy`; copying the pointer does not copy ownership.
    /// The destructor requires a live, non-null handle. Do not call `free` on it.
    #[derive(Debug)]
    #[diplomat::opaque_mut]
    pub struct BoaContext {
        _context: Context,
    }

    impl BoaContext {
        /// Creates a context, or returns null if engine initialization fails.
        ///
        /// A successful call transfers ownership to the caller. Multiple contexts
        /// can coexist on one thread. Unexpected Rust panics and allocation
        /// failures may abort the process rather than return null.
        #[must_use]
        pub fn new() -> Option<Box<BoaContext>> {
            Self::from_builder(Context::builder())
        }

        fn from_builder(builder: ContextBuilder) -> Option<Box<Self>> {
            builder
                .build()
                .ok()
                .map(|context| Box::new(Self { _context: context }))
        }
    }
}
