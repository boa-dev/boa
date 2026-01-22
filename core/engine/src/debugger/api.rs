//! Static API for debugger operations
//!
//! This module provides a static interface for debugger operations,
//! similar to SpiderMonkey's `DebugAPI`.

use super::{DebuggerFrame, ScriptId};
use crate::{Context, JsResult, vm::CallFrame};

/// Static API for debugger operations and event notifications
///
/// This provides a centralized interface for debugger functionality
/// that can be called from various parts of the VM.
#[derive(Debug, Clone, Copy)]
pub struct DebugApi;

impl DebugApi {
    /// Notifies the debugger that a new script has been compiled
    ///
    /// This should be called whenever a new script or function is compiled.
    pub fn on_new_script(
        _context: &mut Context,
        _script_id: ScriptId,
        _source: &str,
    ) -> JsResult<()> {
        // TODO(al): Implement integration with debugger hooks
        Ok(())
    }

    /// Notifies the debugger that a frame is being entered
    ///
    /// This should be called when pushing a new call frame.
    pub fn on_enter_frame(_context: &mut Context, _frame: &CallFrame) -> JsResult<bool> {
        // TODO(al): Implement integration with debugger hooks
        Ok(false)
    }

    /// Notifies the debugger that a frame is being exited
    ///
    /// This should be called when popping a call frame.
    pub fn on_exit_frame(_context: &mut Context, _frame: &CallFrame) -> JsResult<bool> {
        // TODO(al): Implement integration with debugger hooks
        Ok(false)
    }

    /// Notifies the debugger that an exception is being unwound
    ///
    /// This should be called during exception handling.
    pub fn on_exception_unwind(_context: &mut Context, _frame: &CallFrame) -> JsResult<bool> {
        // TODO(al): Implement integration with debugger hooks
        Ok(false)
    }

    /// Checks if there's a breakpoint at the current location
    ///
    /// Returns true if execution should pause.
    pub fn check_breakpoint(
        _context: &mut Context,
        _script_id: ScriptId,
        _pc: u32,
    ) -> JsResult<bool> {
        // TODO(al): Implement breakpoint checking
        Ok(false)
    }

    /// Creates a `DebuggerFrame` from the current execution state
    ///
    /// This is a helper for creating reflection objects.
    pub fn get_current_frame(context: &Context) -> DebuggerFrame {
        let frame = context.vm.frame();
        let depth = context.vm.frames.len();
        DebuggerFrame::from_call_frame(frame, depth)
    }

    /// Gets the call stack as a list of `DebuggerFrames`
    ///
    /// This is useful for showing a backtrace.
    pub fn get_call_stack(context: &Context) -> Vec<DebuggerFrame> {
        let mut frames = Vec::new();

        // Add the current frame
        frames.push(Self::get_current_frame(context));

        // Add frames from the stack
        for (i, frame) in context.vm.frames.iter().enumerate().rev() {
            frames.push(DebuggerFrame::from_call_frame(frame, i));
        }

        frames
    }

    /// Gets the frame depth (number of active call frames)
    pub fn get_frame_depth(context: &Context) -> usize {
        context.vm.frames.len()
    }
}
