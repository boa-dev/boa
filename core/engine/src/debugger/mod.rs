//! Boa's JavaScript Debugger API
//!
//! This module provides a comprehensive debugging interface for JavaScript code
//! running in the Boa engine, inspired by SpiderMonkey's debugger architecture.
//!
//! # Overview
//!
//! The debugger API consists of several key parts:
//!
//! - [`Debugger`]: The main debugger interface that can be attached to a context
//! - [`DebugApi`]: Static API for debugger operations and event notifications
//! - Reflection objects: Safe wrappers for inspecting debuggee state
//!   - [`DebuggerFrame`]: Represents a call frame
//!   - [`DebuggerScript`]: Represents a compiled script/function
//!   - [`DebuggerObject`]: Represents an object in the debuggee
//!
//! # Architecture
//!
//! The debugger uses an event-based hook system similar to SpiderMonkey:
//!
//! - `on_debugger_statement`: Called when `debugger`; statement is executed
//! - `on_enter_frame`: Called when entering a new call frame
//! - `on_exit_frame`: Called when exiting a call frame
//! - `on_exception_unwind`: Called when an exception is being unwound
//! - `on_new_script`: Called when a new script/function is compiled

pub mod api;
pub mod breakpoint;
pub mod dap;
pub mod hooks;
pub mod reflection;
pub mod state;

pub use api::DebugApi;
pub use breakpoint::{Breakpoint, BreakpointId, BreakpointSite};
pub use hooks::{DebuggerEventHandler, DebuggerHooks};
pub use reflection::{DebuggerFrame, DebuggerObject, DebuggerScript};
pub use state::{Debugger, DebuggerState, StepMode};

use crate::JsResult;

/// Result type for debugger operations.
pub type DebugResult<T> = JsResult<T>;

/// Unique identifier for a script or code block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptId(pub(crate) usize);

/// Unique identifier for a call frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameId(pub(crate) usize);

use std::sync::OnceLock;

/// Static flag to check if debugger logging is enabled
static DEBUGGER_LOG_ENABLED: OnceLock<bool> = OnceLock::new();

/// Checks if debugger logging is enabled via `BOA_DAP_DEBUG` environment variable
#[must_use]
pub fn is_debugger_log_enabled() -> bool {
    *DEBUGGER_LOG_ENABLED.get_or_init(|| {
        std::env::var("BOA_DAP_DEBUG")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    })
}

/// Macro for conditional debug logging in debugger modules
/// Only prints when `BOA_DAP_DEBUG` environment variable is set to "1" or "true"
#[macro_export]
macro_rules! dbg_log {
    ($($arg:tt)*) => {
        if $crate::debugger::is_debugger_log_enabled() {
            println!($($arg)*);
        }
    };
}
