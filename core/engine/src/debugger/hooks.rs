//! Event hooks for the debugger
//!
//! This module provides the hook system that allows the debugger to receive
//! notifications about events in the VM execution.

use crate::{Context, JsResult, JsValue, vm::CallFrame};
use boa_engine::dbg_log;

/// Trait for handling debugger events
///
/// Implement this trait to receive notifications about debugger events
/// such as entering/exiting frames, hitting breakpoints, exceptions, etc.
///
/// This is inspired by SpiderMonkey's hook system.
pub trait DebuggerHooks: Send {
    /// Called when a `debugger;` statement is executed
    ///
    /// # Arguments
    ///
    /// * `context` - The current execution context
    /// * `frame` - The current call frame
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` to pause execution, `Ok(false)` to continue
    fn on_debugger_statement(
        &mut self,
        _context: &mut Context,
        _frame: &CallFrame,
    ) -> JsResult<bool> {
        Ok(true) // Default: pause on debugger statement
    }

    /// Called when entering a new call frame
    ///
    /// # Arguments
    ///
    /// * `context` - The current execution context
    /// * `frame` - The frame being entered
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` to pause execution, `Ok(false)` to continue
    fn on_enter_frame(&mut self, _context: &mut Context, _frame: &CallFrame) -> JsResult<bool> {
        Ok(false) // Default: don't pause on frame entry
    }

    /// Called when exiting a call frame
    ///
    /// # Arguments
    ///
    /// * `context` - The current execution context
    /// * `frame` - The frame being exited
    /// * `return_value` - The value being returned from the frame
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` to pause execution, `Ok(false)` to continue
    fn on_exit_frame(
        &mut self,
        _context: &mut Context,
        _frame: &CallFrame,
        _return_value: &JsValue,
    ) -> JsResult<bool> {
        Ok(false) // Default: don't pause on frame exit
    }

    /// Called when an exception is being unwound through a frame
    ///
    /// # Arguments
    ///
    /// * `context` - The current execution context
    /// * `frame` - The frame through which the exception is unwinding
    /// * `exception` - The exception being thrown
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` to pause execution, `Ok(false)` to continue
    fn on_exception_unwind(
        &mut self,
        _context: &mut Context,
        _frame: &CallFrame,
        _exception: &JsValue,
    ) -> JsResult<bool> {
        Ok(false) // Default: don't pause on exceptions
    }

    /// Called when a new script or function is compiled
    ///
    /// # Arguments
    ///
    /// * `context` - The current execution context
    /// * `script_id` - The ID of the newly compiled script
    /// * `source` - The source code of the script
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success
    fn on_new_script(
        &mut self,
        _context: &mut Context,
        _script_id: super::ScriptId,
        _source: &str,
    ) -> JsResult<()> {
        Ok(()) // Default: no-op
    }

    /// Called when a breakpoint is hit
    ///
    /// # Arguments
    ///
    /// * `context` - The current execution context
    /// * `frame` - The current call frame
    /// * `breakpoint_id` - The ID of the breakpoint that was hit
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` to pause execution, `Ok(false)` to continue
    fn on_breakpoint(
        &mut self,
        _context: &mut Context,
        _frame: &CallFrame,
        _breakpoint_id: super::BreakpointId,
    ) -> JsResult<bool> {
        Ok(true) // Default: pause on breakpoint
    }

    /// Called before each instruction is executed
    ///
    /// This can be used to implement single-stepping.
    ///
    /// # Arguments
    ///
    /// * `context` - The current execution context
    /// * `frame` - The current call frame
    /// * `pc` - The program counter (bytecode offset) of the next instruction
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` to pause execution, `Ok(false)` to continue
    fn on_step(&mut self, _context: &mut Context, _frame: &CallFrame, _pc: u32) -> JsResult<bool> {
        Ok(false) // Default: don't pause at every step
    }
}

/// A simple event handler that can be used for basic debugging
///
/// This implementation provides simple logging of debugger events.
#[derive(Debug, Clone, Copy)]
pub struct LoggingEventHandler {
    /// Whether to log frame entry/exit
    pub log_frames: bool,

    /// Whether to log exceptions
    pub log_exceptions: bool,

    /// Whether to log new scripts
    pub log_scripts: bool,
}

impl LoggingEventHandler {
    /// Creates a new logging event handler with all logging enabled
    #[must_use]
    pub fn new() -> Self {
        Self {
            log_frames: true,
            log_exceptions: true,
            log_scripts: true,
        }
    }

    /// Creates a minimal logging handler that only logs essential events
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            log_frames: false,
            log_exceptions: true,
            log_scripts: false,
        }
    }
}

impl Default for LoggingEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DebuggerHooks for LoggingEventHandler {
    fn on_debugger_statement(
        &mut self,
        _context: &mut Context,
        frame: &CallFrame,
    ) -> JsResult<bool> {
        let location = frame.position();
        dbg_log!(
            "[Debugger] Statement hit at {}:{}",
            location.path,
            location
                .position
                .map_or_else(|| "?".to_string(), |p| p.line_number().to_string())
        );
        Ok(true)
    }

    fn on_enter_frame(&mut self, _context: &mut Context, frame: &CallFrame) -> JsResult<bool> {
        if self.log_frames {
            let location = frame.position();
            dbg_log!(
                "[Debugger] Entering frame: {}",
                location.function_name.to_std_string_escaped()
            );
        }
        Ok(false)
    }

    fn on_exit_frame(
        &mut self,
        _context: &mut Context,
        frame: &CallFrame,
        _return_value: &JsValue,
    ) -> JsResult<bool> {
        if self.log_frames {
            let location = frame.position();
            dbg_log!(
                "[Debugger] Exiting frame: {}",
                location.function_name.to_std_string_escaped()
            );
        }
        Ok(false)
    }

    fn on_exception_unwind(
        &mut self,
        _context: &mut Context,
        frame: &CallFrame,
        exception: &JsValue,
    ) -> JsResult<bool> {
        if self.log_exceptions {
            let location = frame.position();
            dbg_log!(
                "[Debugger] Exception in {}: {:?}",
                location.function_name.to_std_string_escaped(),
                exception
            );
        }
        Ok(false)
    }

    fn on_new_script(
        &mut self,
        _context: &mut Context,
        script_id: super::ScriptId,
        _source: &str,
    ) -> JsResult<()> {
        if self.log_scripts {
            dbg_log!("[Debugger] New script compiled: {script_id:?}");
        }
        Ok(())
    }

    fn on_breakpoint(
        &mut self,
        _context: &mut Context,
        frame: &CallFrame,
        breakpoint_id: super::BreakpointId,
    ) -> JsResult<bool> {
        let location = frame.position();
        let line = location
            .position
            .map_or_else(|| "?".to_string(), |p| p.line_number().to_string());
        let path = &location.path;
        dbg_log!("[Debugger] Breakpoint {breakpoint_id} hit at {path}:{line}");
        Ok(true)
    }
}

/// A debugger event handler that can be used as a callback
pub trait DebuggerEventHandler: DebuggerHooks {}

impl<T: DebuggerHooks> DebuggerEventHandler for T {}
