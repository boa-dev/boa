//! DAP protocol message types
//!
//! This module defines all the DAP request, response, and event types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Request Arguments
// ============================================================================

/// Arguments for the `initialize` request
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequestArguments {
    /// The ID of the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// The human-readable name of the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    /// The ID of the debug adapter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_id: Option<String>,
    /// The ISO-639 locale of the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// If true, line numbers start at 1; otherwise at 0
    #[serde(default)]
    pub lines_start_at_1: bool,
    /// If true, column numbers start at 1; otherwise at 0
    #[serde(default)]
    pub columns_start_at_1: bool,
    /// The path format to use ('path' or 'uri')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_format: Option<String>,
    /// Client supports the variable type attribute
    #[serde(default)]
    pub supports_variable_type: bool,
    /// Client supports the paging of variables
    #[serde(default)]
    pub supports_variable_paging: bool,
    /// Client supports the runInTerminal request
    #[serde(default)]
    pub supports_run_in_terminal_request: bool,
    /// Client supports memory references
    #[serde(default)]
    pub supports_memory_references: bool,
    /// Client supports progress reporting
    #[serde(default)]
    pub supports_progress_reporting: bool,
    /// Client supports the invalidated event
    #[serde(default)]
    pub supports_invalidated_event: bool,
}

/// Arguments for the `launch` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequestArguments {
    /// If true, launch without debugging
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_debug: Option<bool>,
    /// The program to debug
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    /// Command-line arguments passed to the program
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Working directory of the program
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    /// If true, stop on the first line of the program
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_on_entry: Option<bool>,
}

/// Arguments for the `attach` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachRequestArguments {
    /// The port to attach to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// The address to attach to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

/// Arguments for the `setBreakpoints` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBreakpointsArguments {
    /// The source file for the breakpoints
    pub source: Source,
    /// The breakpoints to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakpoints: Option<Vec<SourceBreakpoint>>,
    /// Deprecated: use `breakpoints` instead
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<Vec<i64>>,
    /// If true, the underlying source has been modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_modified: Option<bool>,
}

/// A breakpoint in source code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBreakpoint {
    /// The line number of the breakpoint
    pub line: i64,
    /// The optional column number of the breakpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    /// An optional expression for conditional breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// An optional expression that controls how many hits are ignored
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_condition: Option<String>,
    /// Optional log message (makes this a logpoint)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_message: Option<String>,
}

/// Arguments for the `continue` request
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueArguments {
    /// The thread to continue
    pub thread_id: i64,
    /// If true, only this thread is continued
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_thread: Option<bool>,
}

/// Arguments for the `next` request (step over)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextArguments {
    /// The thread to step
    pub thread_id: i64,
    /// If true, only this thread is stepped
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_thread: Option<bool>,
    /// The stepping granularity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<String>,
}

/// Arguments for the `stepIn` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepInArguments {
    /// The thread to step
    pub thread_id: i64,
    /// If true, only this thread is stepped
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_thread: Option<bool>,
    /// Optional ID of a specific function to step into
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<i64>,
    /// The stepping granularity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<String>,
}

/// Arguments for the `stepOut` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepOutArguments {
    /// The thread to step
    pub thread_id: i64,
    /// If true, only this thread is stepped
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_thread: Option<bool>,
    /// The stepping granularity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<String>,
}

/// Arguments for the `stackTrace` request
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackTraceArguments {
    /// The thread for which to retrieve the stack trace
    pub thread_id: i64,
    /// The index of the first frame to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_frame: Option<i64>,
    /// The maximum number of frames to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<i64>,
    /// Specifies details on how to format the stack frames
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<StackFrameFormat>,
}

/// Arguments for the `scopes` request
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopesArguments {
    /// The frame for which to retrieve the scopes
    pub frame_id: i64,
}

/// Arguments for the `variables` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablesArguments {
    /// The variable reference to retrieve variables for
    pub variables_reference: i64,
    /// Filter to apply to children ('indexed', 'named')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    /// The index of the first variable to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i64>,
    /// The number of variables to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    /// Specifies details on how to format the variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ValueFormat>,
}

/// Arguments for the `evaluate` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateArguments {
    /// The expression to evaluate
    pub expression: String,
    /// Evaluate in the context of this stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<i64>,
    /// The context in which the evaluated request is used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Specifies details on how to format the result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ValueFormat>,
}

/// Arguments for the `source` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceArguments {
    /// The source to retrieve
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// The reference to the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_reference: Option<i64>,
}

// ============================================================================
// Response Bodies
// ============================================================================

/// Debug adapter capabilities
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    /// Adapter supports the `configurationDone` request
    #[serde(default)]
    pub supports_configuration_done_request: bool,
    /// Adapter supports function breakpoints
    #[serde(default)]
    pub supports_function_breakpoints: bool,
    /// Adapter supports conditional breakpoints
    #[serde(default)]
    pub supports_conditional_breakpoints: bool,
    /// Adapter supports hit conditional breakpoints
    #[serde(default)]
    pub supports_hit_conditional_breakpoints: bool,
    /// Adapter supports the `evaluate` request for hover tooltips
    #[serde(default)]
    pub supports_evaluate_for_hovers: bool,
    /// Adapter supports stepping back
    #[serde(default)]
    pub supports_step_back: bool,
    /// Adapter supports the `setVariable` request
    #[serde(default)]
    pub supports_set_variable: bool,
    /// Adapter supports the `restartFrame` request
    #[serde(default)]
    pub supports_restart_frame: bool,
    /// Adapter supports the `gotoTargets` request
    #[serde(default)]
    pub supports_goto_targets_request: bool,
    /// Adapter supports the `stepInTargets` request
    #[serde(default)]
    pub supports_step_in_targets_request: bool,
    /// Adapter supports the `completions` request
    #[serde(default)]
    pub supports_completions_request: bool,
    /// Adapter supports the `modules` request
    #[serde(default)]
    pub supports_modules_request: bool,
    /// Adapter supports the `restart` request
    #[serde(default)]
    pub supports_restart_request: bool,
    /// Adapter supports exception configuration options
    #[serde(default)]
    pub supports_exception_options: bool,
    /// Adapter supports value formatting options
    #[serde(default)]
    pub supports_value_formatting_options: bool,
    /// Adapter supports the `exceptionInfo` request
    #[serde(default)]
    pub supports_exception_info_request: bool,
    /// Adapter supports terminating the debuggee
    #[serde(default)]
    pub supports_terminate_debuggee: bool,
    /// Adapter supports delayed loading of stack traces
    #[serde(default)]
    pub supports_delayed_stack_trace_loading: bool,
    /// Adapter supports the `loadedSources` request
    #[serde(default)]
    pub supports_loaded_sources_request: bool,
    /// Adapter supports logpoints
    #[serde(default)]
    pub supports_log_points: bool,
    /// Adapter supports the `terminateThreads` request
    #[serde(default)]
    pub supports_terminate_threads_request: bool,
    /// Adapter supports the `setExpression` request
    #[serde(default)]
    pub supports_set_expression: bool,
    /// Adapter supports the `terminate` request
    #[serde(default)]
    pub supports_terminate_request: bool,
    /// Adapter supports data breakpoints
    #[serde(default)]
    pub supports_data_breakpoints: bool,
    /// Adapter supports the `readMemory` request
    #[serde(default)]
    pub supports_read_memory_request: bool,
    /// Adapter supports the `disassemble` request
    #[serde(default)]
    pub supports_disassemble_request: bool,
    /// Adapter supports the `cancel` request
    #[serde(default)]
    pub supports_cancel_request: bool,
    /// Adapter supports the `breakpointLocations` request
    #[serde(default)]
    pub supports_breakpoint_locations_request: bool,
    /// Adapter supports clipboard context for evaluate
    #[serde(default)]
    pub supports_clipboard_context: bool,
}

/// Response body for `setBreakpoints` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBreakpointsResponseBody {
    /// Information about the breakpoints that were set
    pub breakpoints: Vec<Breakpoint>,
}

/// Response body for `continue` request
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueResponseBody {
    /// If true, all threads have been continued
    #[serde(default)]
    pub all_threads_continued: bool,
}

/// Response body for `stackTrace` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackTraceResponseBody {
    /// The stack frames
    pub stack_frames: Vec<StackFrame>,
    /// The total number of frames available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_frames: Option<i64>,
}

/// Response body for `scopes` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopesResponseBody {
    /// The scopes for the given frame
    pub scopes: Vec<Scope>,
}

/// Response body for `variables` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablesResponseBody {
    /// The variables
    pub variables: Vec<Variable>,
}

/// Response body for `evaluate` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateResponseBody {
    /// The result of the evaluation
    pub result: String,
    /// The type of the result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// Properties of the result that can be used to determine how to render it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<VariablePresentationHint>,
    /// If the result is structured, a handle for retrieval
    pub variables_reference: i64,
    /// The number of named child variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_variables: Option<i64>,
    /// The number of indexed child variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_variables: Option<i64>,
}

/// Response body for `threads` request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadsResponseBody {
    /// All threads
    pub threads: Vec<Thread>,
}

// ============================================================================
// Types
// ============================================================================

/// A source file or script
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// The short name of the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The path of the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// If `sourceReference > 0`, the client can retrieve source via `source` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_reference: Option<i64>,
    /// A hint for how to present the source in the UI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<String>,
    /// The origin of this source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// A list of sources that are related to this source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<Source>>,
    /// Optional data that a debug adapter might want to loop through
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_data: Option<serde_json::Value>,
    /// The checksums associated with this file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksums: Option<Vec<Checksum>>,
}

/// A checksum for a source file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checksum {
    /// The algorithm used to calculate the checksum
    pub algorithm: String,
    /// The checksum value
    pub checksum: String,
}

/// Information about a breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakpoint {
    /// The unique ID of the breakpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// If true, the breakpoint could be set
    pub verified: bool,
    /// An optional message about the breakpoint state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The source where the breakpoint is located
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// The line number of the breakpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    /// The column number of the breakpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    /// The optional end line of the breakpoint range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i64>,
    /// The optional end column of the breakpoint range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i64>,
}

/// A stack frame
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    /// The unique ID of the stack frame
    pub id: i64,
    /// The name of the stack frame (typically function name)
    pub name: String,
    /// The source of the frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// The line within the source of the frame
    pub line: i64,
    /// The column within the line
    pub column: i64,
    /// The optional end line of the range covered by the stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i64>,
    /// The optional end column of the range covered by the stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i64>,
    /// If true, the frame can be restarted
    #[serde(default)]
    pub can_restart: bool,
    /// A memory reference for the current instruction pointer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_pointer_reference: Option<String>,
    /// The module associated with this frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_id: Option<serde_json::Value>,
    /// A hint for how to present this frame in the UI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<String>,
}

/// A scope (such as 'Locals', 'Globals', 'Closure')
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    /// Name of the scope (e.g., 'Locals', 'Globals')
    pub name: String,
    /// A hint for how to present this scope in the UI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<String>,
    /// The variables reference for this scope
    pub variables_reference: i64,
    /// The number of named variables in this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_variables: Option<i64>,
    /// The number of indexed variables in this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_variables: Option<i64>,
    /// If true, the number of variables is large or expensive to retrieve
    pub expensive: bool,
    /// Optional source for this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// Optional start line of the range covered by this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    /// Optional start column of the range covered by this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    /// Optional end line of the range covered by this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i64>,
    /// Optional end column of the range covered by this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i64>,
}

/// A variable
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variable {
    /// The name of the variable
    pub name: String,
    /// The value of the variable as a string
    pub value: String,
    /// The type of the variable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// Properties of the variable that can be used to determine how to render it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<VariablePresentationHint>,
    /// The expression to use in `evaluate` requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluate_name: Option<String>,
    /// If the value is structured, a handle for retrieval
    pub variables_reference: i64,
    /// The number of named child variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_variables: Option<i64>,
    /// The number of indexed child variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_variables: Option<i64>,
    /// A memory reference to the variable's value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_reference: Option<String>,
}

/// A thread
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    /// The unique ID of the thread
    pub id: i64,
    /// The name of the thread
    pub name: String,
}

/// Formatting options for stack frames
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrameFormat {
    /// Display parameters for the stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<bool>,
    /// Display parameter types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_types: Option<bool>,
    /// Display parameter names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_names: Option<bool>,
    /// Display parameter values
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_values: Option<bool>,
    /// Display line number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<bool>,
    /// Display module name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<bool>,
    /// Include all available format options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_all: Option<bool>,
}

/// Formatting options for values
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueFormat {
    /// Display integers in hexadecimal format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hex: Option<bool>,
}

/// Optional properties of a variable that can be used to determine how to render it
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablePresentationHint {
    /// The kind of variable (e.g., 'property', 'method', 'class')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Set of attributes represented as an array of strings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,
    /// Visibility of the variable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
}

// ============================================================================
// Event Bodies
// ============================================================================

/// Event body for `stopped` event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoppedEventBody {
    /// The reason for the stop event
    pub reason: String,
    /// Additional information about the stop event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The thread which was stopped
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i64>,
    /// If true, the UI should not change focus
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_focus_hint: Option<bool>,
    /// Additional textual information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// If true, all threads have been stopped
    #[serde(default)]
    pub all_threads_stopped: bool,
    /// IDs of the breakpoints that triggered the stop
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_breakpoint_ids: Option<Vec<i64>>,
}

/// Event body for `continued` event
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuedEventBody {
    /// The thread which continued
    pub thread_id: i64,
    /// If true, all threads have been continued
    #[serde(default)]
    pub all_threads_continued: bool,
}

/// Event body for `thread` event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadEventBody {
    /// The reason for the thread event ('started' or 'exited')
    pub reason: String,
    /// The ID of the thread
    pub thread_id: i64,
}

/// Event body for `output` event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputEventBody {
    /// The output category (e.g., 'console', 'stdout', 'stderr')
    pub category: Option<String>,
    /// The output to report
    pub output: String,
    /// Support for keeping output in groups
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// If the output is structured, a handle for retrieval
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables_reference: Option<i64>,
    /// Optional source location of the output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// Optional line number where the output was generated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    /// Optional column number where the output was generated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    /// Optional additional data to report
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Event body for `terminated` event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminatedEventBody {
    /// A debug adapter may set 'restart' to true to request a restart of the session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<serde_json::Value>,
}

/// Event body for `exited` event
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitedEventBody {
    /// The exit code returned from the debuggee
    pub exit_code: i64,
}
