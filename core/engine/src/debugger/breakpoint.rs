//! Breakpoint management for the debugger

use super::ScriptId;
use std::fmt;

/// Unique identifier for a breakpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BreakpointId(pub(crate) usize);

impl fmt::Display for BreakpointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bp#{}", self.0)
    }
}

/// A breakpoint location in the debuggee
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Unique identifier for this breakpoint
    pub id: BreakpointId,

    /// The script this breakpoint is in
    pub script_id: ScriptId,

    /// The program counter (bytecode offset) where the breakpoint is set
    pub pc: u32,

    /// Optional condition that must evaluate to true for the breakpoint to trigger
    pub condition: Option<String>,

    /// Number of times this breakpoint has been hit
    pub hit_count: u32,

    /// Whether this breakpoint is currently enabled
    pub enabled: bool,

    /// Optional log message to print when a breakpoint is hit (instead of pausing)
    pub log_message: Option<String>,
}

impl Breakpoint {
    /// Creates a new breakpoint
    #[must_use]
    pub fn new(id: BreakpointId, script_id: ScriptId, pc: u32) -> Self {
        Self {
            id,
            script_id,
            pc,
            condition: None,
            hit_count: 0,
            enabled: true,
            log_message: None,
        }
    }

    /// Creates a conditional breakpoint
    #[must_use]
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Creates a log breakpoint (doesn't pause, just logs)
    #[must_use]
    pub fn with_log_message(mut self, message: String) -> Self {
        self.log_message = Some(message);
        self
    }

    /// Increments the hit count and returns the new count
    #[must_use]
    pub fn increment_hit_count(&mut self) -> u32 {
        self.hit_count += 1;
        self.hit_count
    }

    /// Checks if the breakpoint should trigger based on its condition
    ///
    /// Returns true if there's no condition or if the condition evaluates to true
    #[must_use]
    pub fn should_trigger(&self, _context: &crate::Context) -> bool {
        // TODO: Implement condition evaluation
        // For now, always trigger if there's no condition
        self.condition.is_none()
    }

    /// Whether this is a log breakpoint (logs but doesn't pause)
    #[must_use]
    pub fn is_log_breakpoint(&self) -> bool {
        self.log_message.is_some()
    }
}

/// A breakpoint site represents a unique location where a breakpoint can be set
///
/// Multiple breakpoints might map to the same site (e.g., different conditions
/// at the same location)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BreakpointSite {
    /// The script this site is in
    pub script_id: ScriptId,

    /// Program counter (bytecode offset) of this site
    pub pc: u32,
}

impl BreakpointSite {
    /// Creates a new breakpoint site
    #[must_use]
    pub fn new(script_id: ScriptId, pc: u32) -> Self {
        Self { script_id, pc }
    }
}

/// Options for creating a breakpoint
#[derive(Debug, Clone, Default)]
pub struct BreakpointOptions {
    /// Optional condition expression
    pub condition: Option<String>,

    /// Optional log message (makes this a logpoint)
    pub log_message: Option<String>,

    /// Whether the breakpoint is initially enabled
    pub enabled: bool,
}

impl BreakpointOptions {
    /// Creates new breakpoint options with default values
    #[must_use]
    pub fn new() -> Self {
        Self {
            condition: None,
            log_message: None,
            enabled: true,
        }
    }

    /// Sets a condition for the breakpoint
    #[must_use]
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Sets a log message (makes this a logpoint)
    #[must_use]
    pub fn with_log_message(mut self, message: String) -> Self {
        self.log_message = Some(message);
        self
    }

    /// Sets whether the breakpoint is enabled
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
