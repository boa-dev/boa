//! Interop utilities between Boa and its host.
#![deprecated(note = "All interop APIs were moved to boa_engine")]

pub use boa_engine;
pub use boa_macros;

// Re-export in case some people depend on boa_interop.
#[deprecated(note = "Please use these exports from boa_engine::interop instead.")]
pub use boa_engine::interop::{ContextData, Ignore, JsClass, JsRest};

#[deprecated(note = "Please use these exports from boa_engine instead.")]
pub use boa_engine::{
    IntoJsFunctionCopied, IntoJsModule, UnsafeIntoJsFunction, boa_class, boa_module,
};
