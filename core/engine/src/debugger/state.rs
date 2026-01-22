//! Core debugger state management

use super::{Breakpoint, BreakpointId, BreakpointSite, DebuggerHooks, ScriptId};
use crate::{Context, JsResult};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Step execution mode for the debugger
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    /// Not stepping - run normally until next breakpoint
    None,
    /// Step to the next instruction (step in)
    StepIn,
    /// Step over the current instruction (don't enter function calls)
    StepOver,
    /// Step out of the current frame
    StepOut,
    /// Continue until specific frame depth
    StepToFrame(usize),
}

/// Current state of the debugger
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuggerState {
    /// Debugger is running normally
    Running,
    /// Debugger is paused (e.g., at a breakpoint)
    Paused,
    /// Debugger is stepping through code
    Stepping(StepMode),
}

/// The main Debugger struct that manages debugging state and operations
///
/// This is inspired by SpiderMonkey's Debugger class and provides a comprehensive
/// API for debugging JavaScript code running in Boa.
///
/// # Architecture
///
/// The Debugger maintains:
/// - Breakpoint state (locations, conditions, hit counts)
/// - Stepping state (step-in, step-over, step-out)
/// - Hook callbacks for debugger events
/// - Weak references to debuggee contexts (via compartment isolation)
///
/// # Example
///
/// ```rust,ignore
/// use boa_engine::{Context, debugger::Debugger};
///
/// let mut context = Context::default();
/// let mut debugger = Debugger::new();
///
/// // Attach to context
/// debugger.attach(&mut context);
///
/// // Set breakpoint
/// debugger.set_breakpoint_by_line("main.js", 10);
///
/// // Execute code
/// context.eval(Source::from_bytes("function test() { debugger; } test();"));
/// ```
pub struct Debugger {
    /// Current state of the debugger
    state: DebuggerState,

    /// Breakpoints indexed by script ID and program counter
    breakpoints: HashMap<ScriptId, HashMap<u32, Breakpoint>>,

    /// Breakpoint sites (unique locations where breakpoints can be set)
    breakpoint_sites: HashMap<BreakpointId, BreakpointSite>,

    /// Next breakpoint ID to assign
    next_breakpoint_id: AtomicUsize,

    /// Currently enabled breakpoint IDs
    enabled_breakpoints: HashSet<BreakpointId>,

    /// Current frame depth when stepping
    step_frame_depth: Option<usize>,

    /// Whether the debugger is attached to a context
    attached: AtomicBool,

    /// Whether the debugger is shutting down
    shutting_down: AtomicBool,

    /// Event hooks for debugger events
    hooks: Option<Box<dyn DebuggerHooks + 'static>>,
}

impl std::fmt::Debug for Debugger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Debugger")
            .field("state", &self.state)
            .field("breakpoints", &self.breakpoints)
            .field("breakpoint_sites", &self.breakpoint_sites)
            .field("next_breakpoint_id", &self.next_breakpoint_id)
            .field("enabled_breakpoints", &self.enabled_breakpoints)
            .field("step_frame_depth", &self.step_frame_depth)
            .field("attached", &self.attached)
            .field("shutting_down", &self.shutting_down)
            .field("hooks", &self.hooks.as_ref().map(|_| "Box<dyn DebuggerHooks>"))
            .finish()
    }
}

impl Debugger {
    /// Creates a new debugger instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: DebuggerState::Running,
            breakpoints: HashMap::new(),
            breakpoint_sites: HashMap::new(),
            next_breakpoint_id: AtomicUsize::new(0),
            enabled_breakpoints: HashSet::new(),
            step_frame_depth: None,
            attached: AtomicBool::new(false),
            shutting_down: AtomicBool::new(false),
            hooks: None,
        }
    }

    /// Attaches the debugger to a context
    ///
    /// This enables debugging for the given context. The debugger will receive
    /// events for all code execution in the context.
    pub fn attach(&mut self, _context: &mut Context) -> JsResult<()> {
        self.attached.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Detaches the debugger from the context
    ///
    /// After detaching, the debugger will no longer receive events.
    pub fn detach(&mut self) -> JsResult<()> {
        self.attached.store(false, Ordering::SeqCst);
        self.state = DebuggerState::Running;
        Ok(())
    }

    /// Checks if the debugger is attached
    pub fn is_attached(&self) -> bool {
        self.attached.load(Ordering::SeqCst)
    }

    /// Gets the current debugger state
    pub fn state(&self) -> DebuggerState {
        self.state
    }

    /// Sets the debugger state
    pub fn set_state(&mut self, state: DebuggerState) {
        self.state = state;
    }

    /// Pauses execution at the next opportunity
    pub fn pause(&mut self) {
        self.state = DebuggerState::Paused;
    }

    /// Resumes execution
    pub fn resume(&mut self) {
        self.state = DebuggerState::Running;
        self.step_frame_depth = None;
    }

    /// Signals that the debugger is shutting down
    pub fn shutdown(&mut self) {
        self.shutting_down.store(true, Ordering::SeqCst);
        self.state = DebuggerState::Running; // Ensure we don't stay paused
    }

    /// Checks if the debugger is shutting down
    pub fn is_shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::SeqCst)
    }

    /// Checks if the debugger is paused
    pub fn is_paused(&self) -> bool {
        matches!(self.state, DebuggerState::Paused)
    }

    /// Steps to the next instruction (step in)
    pub fn step_in(&mut self) {
        self.state = DebuggerState::Stepping(StepMode::StepIn);
    }

    /// Steps over the current instruction (don't enter function calls)
    pub fn step_over(&mut self, current_depth: usize) {
        self.step_frame_depth = Some(current_depth);
        self.state = DebuggerState::Stepping(StepMode::StepOver);
    }

    /// Steps out of the current frame
    pub fn step_out(&mut self, current_depth: usize) {
        if current_depth > 0 {
            self.step_frame_depth = Some(current_depth - 1);
            self.state = DebuggerState::Stepping(StepMode::StepOut);
        }
    }

    /// Allocates a new breakpoint ID
    fn allocate_breakpoint_id(&self) -> BreakpointId {
        BreakpointId(self.next_breakpoint_id.fetch_add(1, Ordering::SeqCst))
    }

    /// Sets a breakpoint at the given program counter in a script
    pub fn set_breakpoint(&mut self, script_id: ScriptId, pc: u32) -> BreakpointId {
        let id = self.allocate_breakpoint_id();
        let breakpoint = Breakpoint::new(id, script_id, pc);

        self.breakpoints
            .entry(script_id)
            .or_default()
            .insert(pc, breakpoint.clone());

        let site = BreakpointSite::new(script_id, pc);
        self.breakpoint_sites.insert(id, site);
        self.enabled_breakpoints.insert(id);

        id
    }

    /// Removes a breakpoint by ID
    pub fn remove_breakpoint(&mut self, id: BreakpointId) -> bool {
        if let Some(site) = self.breakpoint_sites.remove(&id) {
            if let Some(script_breakpoints) = self.breakpoints.get_mut(&site.script_id) {
                script_breakpoints.remove(&site.pc);
            }
            self.enabled_breakpoints.remove(&id);
            true
        } else {
            false
        }
    }

    /// Checks if there's a breakpoint at the given location
    pub fn has_breakpoint(&self, script_id: ScriptId, pc: u32) -> bool {
        self.breakpoints
            .get(&script_id)
            .and_then(|bps| bps.get(&pc))
            .map_or(false, |bp| self.enabled_breakpoints.contains(&bp.id))
    }

    /// Enables a breakpoint
    pub fn enable_breakpoint(&mut self, id: BreakpointId) -> bool {
        if self.breakpoint_sites.contains_key(&id) {
            self.enabled_breakpoints.insert(id);
            true
        } else {
            false
        }
    }

    /// Disables a breakpoint
    pub fn disable_breakpoint(&mut self, id: BreakpointId) -> bool {
        self.enabled_breakpoints.remove(&id)
    }

    /// Gets all breakpoints for a script
    pub fn get_breakpoints(&self, script_id: ScriptId) -> Vec<&Breakpoint> {
        self.breakpoints
            .get(&script_id)
            .map(|bps| bps.values().collect())
            .unwrap_or_default()
    }

    /// Checks if we should pause at the current location based on stepping state
    pub fn should_pause_for_step(&mut self, frame_depth: usize) -> bool {
        match self.state {
            DebuggerState::Paused => {
                // Already paused, should continue waiting
                true
            }
            DebuggerState::Running | DebuggerState::Stepping(StepMode::None) => {
                // Running normally
                false
            }
            DebuggerState::Stepping(StepMode::StepIn) => {
                // Always pause on next instruction
                self.state = DebuggerState::Paused;
                true
            }
            DebuggerState::Stepping(StepMode::StepOver) => {
                // Pause if we're at or above the original frame depth
                if let Some(target_depth) = self.step_frame_depth
                    && frame_depth <= target_depth
                {
                    self.state = DebuggerState::Paused;
                    self.step_frame_depth = None;
                    return true;
                }
                false
            }
            DebuggerState::Stepping(StepMode::StepOut) => {
                // Pause if we've returned to a shallower frame
                if let Some(target_depth) = self.step_frame_depth
                    && frame_depth <= target_depth
                {
                    self.state = DebuggerState::Paused;
                    self.step_frame_depth = None;
                    return true;
                }
                false
            }
            DebuggerState::Stepping(StepMode::StepToFrame(target)) => {
                if frame_depth == target {
                    self.state = DebuggerState::Paused;
                    self.step_frame_depth = None;
                    return true;
                }
                false
            }
        }
    }

    /// Sets custom event hooks for debugger events
    pub fn set_hooks(&mut self, hooks: Box<dyn DebuggerHooks + 'static>) {
        self.hooks = Some(hooks);
    }

    /// Gets a reference to the event hooks
    pub fn hooks(&self) -> Option<&(dyn DebuggerHooks + 'static)> {
        self.hooks.as_deref()
    }

    /// Gets a mutable reference to the event hooks
    pub fn hooks_mut(&mut self) -> Option<&mut (dyn DebuggerHooks + 'static)> {
        self.hooks.as_deref_mut()
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}
