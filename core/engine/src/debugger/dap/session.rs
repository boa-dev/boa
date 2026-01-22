//! Debug session management
//!
//! This module implements the debug session that connects the DAP protocol
//! with Boa's debugger API.

use super::{
    eval_context::{DebugEvalContext, DebugEvent},
    messages::{
        AttachRequestArguments, Breakpoint, Capabilities, ContinueArguments, ContinueResponseBody,
        EvaluateArguments, EvaluateResponseBody, InitializeRequestArguments,
        LaunchRequestArguments, NextArguments, Scope, ScopesArguments, ScopesResponseBody,
        SetBreakpointsArguments, SetBreakpointsResponseBody, Source, StackFrame,
        StackTraceArguments, StackTraceResponseBody, StepInArguments, StepOutArguments, Thread,
        ThreadsResponseBody, VariablesArguments, VariablesResponseBody,
    },
};
use crate::{
    Context, JsResult, dbg_log,
    debugger::{BreakpointId, Debugger, ScriptId},
};
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

/// Type alias for debug event handler callback
type EventHandler = Box<dyn Fn(DebugEvent) + Send + 'static>;

/// A debug session manages the connection between DAP and Boa's debugger
#[derive(Debug)]
pub struct DebugSession {
    /// The Boa debugger instance
    debugger: Arc<Mutex<Debugger>>,

    /// Condition variable for pause/resume signaling
    condvar: Arc<Condvar>,

    /// The evaluation context (runs in a dedicated thread)
    eval_context: Option<DebugEvalContext>,

    /// Program path from launch request
    program_path: Option<String>,

    /// Mapping from source paths to script IDs
    source_to_script: HashMap<String, ScriptId>,

    /// Mapping from DAP breakpoint IDs to Boa breakpoint IDs
    breakpoint_mapping: HashMap<i64, BreakpointId>,

    /// Next DAP breakpoint ID
    next_breakpoint_id: i64,

    /// Whether the session is initialized
    initialized: bool,

    /// Whether the session is running
    running: bool,

    /// Current thread ID (Boa is single-threaded, so this is always 1)
    thread_id: i64,

    /// Stopped reason
    stopped_reason: Option<String>,

    /// Variable references for scopes and objects
    variable_references: HashMap<i64, VariableReference>,
    next_variable_reference: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum VariableReference {
    Scope {
        frame_id: i64,
        scope_type: ScopeType,
    },
    Object {
        object_id: String,
    },
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum ScopeType {
    Local,
    Global,
    Closure,
}

impl DebugSession {
    /// Creates a new debug session
    #[must_use]
    pub fn new(debugger: Arc<Mutex<Debugger>>) -> Self {
        Self {
            debugger,
            condvar: Arc::new(Condvar::new()),
            eval_context: None,
            program_path: None,
            source_to_script: HashMap::new(),
            breakpoint_mapping: HashMap::new(),
            next_breakpoint_id: 1,
            initialized: false,
            running: false,
            thread_id: 1,
            stopped_reason: None,
            variable_references: HashMap::new(),
            next_variable_reference: 1,
        }
    }

    /// Pauses execution
    pub fn pause(&mut self) -> JsResult<()> {
        self.debugger
            .lock()
            .map_err(|e| {
                crate::JsNativeError::error().with_message(format!("Debugger mutex poisoned: {e}"))
            })?
            .pause();
        Ok(())
    }

    /// Resumes execution and notifies waiting threads
    pub fn resume(&mut self) -> JsResult<()> {
        self.debugger
            .lock()
            .map_err(|e| {
                crate::JsNativeError::error().with_message(format!("Debugger mutex poisoned: {e}"))
            })?
            .resume();
        self.running = true;
        self.stopped_reason = None;
        // Wake up all threads waiting on the condition variable
        self.condvar.notify_all();
        Ok(())
    }

    /// Checks if the debugger is paused
    pub fn is_paused(&self) -> JsResult<bool> {
        Ok(self
            .debugger
            .lock()
            .map_err(|e| {
                crate::JsNativeError::error().with_message(format!("Debugger mutex poisoned: {e}"))
            })?
            .is_paused())
    }

    /// Handles the initialized request
    pub fn handle_initialize(
        &mut self,
        _args: InitializeRequestArguments,
    ) -> JsResult<Capabilities> {
        self.initialized = true;

        Ok(Capabilities {
            supports_configuration_done_request: true,
            supports_function_breakpoints: false,
            supports_conditional_breakpoints: true,
            supports_hit_conditional_breakpoints: true,
            supports_evaluate_for_hovers: true,
            supports_step_back: false,
            supports_set_variable: false,
            supports_restart_frame: false,
            supports_goto_targets_request: false,
            supports_step_in_targets_request: false,
            supports_completions_request: false,
            supports_modules_request: false,
            supports_restart_request: false,
            supports_exception_options: false,
            supports_value_formatting_options: true,
            supports_exception_info_request: false,
            supports_terminate_debuggee: true,
            supports_delayed_stack_trace_loading: false,
            supports_loaded_sources_request: false,
            supports_log_points: true,
            supports_terminate_threads_request: false,
            supports_set_expression: false,
            supports_terminate_request: true,
            supports_data_breakpoints: false,
            supports_read_memory_request: false,
            supports_disassemble_request: false,
            supports_cancel_request: false,
            supports_breakpoint_locations_request: false,
            supports_clipboard_context: false,
        })
    }

    /// Handles the `launch` request.
    ///
    /// Creates the evaluation context in a dedicated thread.
    /// Takes a setup function that will be called in the eval thread after `Context` is created.
    /// Takes an event handler that will be called for each debug event (for TCP mode).
    /// Spawns event forwarder thread BEFORE executing the program to avoid missing events.
    /// If a program path is provided, automatically reads and executes it.
    #[allow(clippy::type_complexity)]
    pub fn handle_launch(
        &mut self,
        args: &LaunchRequestArguments,
        context_setup: Box<dyn FnOnce(&mut Context) -> JsResult<()> + Send>,
        event_handler: EventHandler,
    ) -> JsResult<()> {
        // Store the program path for later execution
        self.program_path.clone_from(&args.program);

        // Create the evaluation context, passing the setup function to the thread
        let (eval_context, event_rx) =
            DebugEvalContext::new(context_setup, self.debugger.clone(), self.condvar.clone())?;

        self.eval_context = Some(eval_context);
        self.running = false;

        dbg_log!("[DebugSession] Evaluation context created");

        // Spawn event forwarder thread BEFORE executing the program
        // This ensures no events are missed from the first program execution
        std::thread::spawn(move || {
            dbg_log!("[DebugSession] Event forwarder thread started");

            // Block on receiver - clean, no polling, no locks
            while let Ok(event) = event_rx.recv() {
                match &event {
                    DebugEvent::Shutdown => {
                        dbg_log!("[DebugSession] Shutdown signal received");
                        event_handler(event);
                        break;
                    }
                    DebugEvent::Stopped { reason, .. } => {
                        dbg_log!("[DebugSession] Forwarding stopped event: {reason}");
                        event_handler(event);
                    }
                    DebugEvent::Terminated => {
                        dbg_log!("[DebugSession] Forwarding terminated event");
                        event_handler(event);
                    }
                }
            }

            dbg_log!("[DebugSession] Event forwarder thread terminated cleanly");
        });

        dbg_log!("[DebugSession] Event forwarder thread spawned");

        // NOW execute the program after forwarder is ready
        // If we have a program path, read and start executing it asynchronously
        // Don't wait for the result as execution may hit breakpoints
        if let Some(program_path) = &self.program_path {
            dbg_log!("[DebugSession] Starting program execution: {program_path}");

            // Execute the program asynchronously (non-blocking)
            // The eval thread will process it and can be interrupted by breakpoints
            if let Some(ctx) = &self.eval_context {
                ctx.execute_async(program_path.clone()).map_err(|e| {
                    crate::JsNativeError::error()
                        .with_message(format!("Failed to start execution: {e}"))
                })?;
            }

            dbg_log!("[DebugSession] Program execution started (non-blocking)");
        }

        Ok(())
    }

    /// Gets the program path from the launch request
    #[must_use]
    pub fn get_program_path(&self) -> Option<&str> {
        self.program_path.as_deref()
    }

    /// Executes JavaScript code in the evaluation thread
    pub fn execute(&self, source: String) -> Result<String, String> {
        match &self.eval_context {
            Some(ctx) => ctx.execute(source),
            None => {
                Err("Evaluation context not initialized. Call handle_launch first.".to_string())
            }
        }
    }

    /// Handles the `attach` request
    pub fn handle_attach(&mut self, _args: AttachRequestArguments) -> JsResult<()> {
        // Attach will be handled by the CLI tool
        Ok(())
    }

    /// Handles setting breakpoints
    pub fn handle_set_breakpoints(
        &mut self,
        args: &SetBreakpointsArguments,
    ) -> JsResult<SetBreakpointsResponseBody> {
        let mut breakpoints = Vec::new();

        // Get the source path
        let source_path = args.source.path.clone().unwrap_or_else(|| {
            args.source
                .name
                .clone()
                .unwrap_or_else(|| "unknown".to_string())
        });

        // Get the script ID for this source
        // For now, we'll use a placeholder since we need line-to-PC mapping
        let script_id = *self
            .source_to_script
            .entry(source_path.clone())
            .or_insert(ScriptId(0));

        if let Some(source_breakpoints) = &args.breakpoints {
            for bp in source_breakpoints {
                // TODO: Map line number to PC offset
                // For now, we'll create a placeholder
                let boa_bp_id = {
                    let mut debugger = self.debugger.lock().map_err(|e| {
                        crate::JsNativeError::error()
                            .with_message(format!("Debugger mutex poisoned: {e}"))
                    })?;
                    debugger.set_breakpoint(script_id, bp.line as u32)
                };

                let dap_bp_id = self.next_breakpoint_id;
                self.next_breakpoint_id += 1;

                self.breakpoint_mapping.insert(dap_bp_id, boa_bp_id);

                breakpoints.push(Breakpoint {
                    id: Some(dap_bp_id),
                    verified: true,
                    message: None,
                    source: Some(args.source.clone()),
                    line: Some(bp.line),
                    column: bp.column,
                    end_line: None,
                    end_column: None,
                });
            }
        }

        Ok(SetBreakpointsResponseBody { breakpoints })
    }

    /// Handles the `continue` request
    pub fn handle_continue(&mut self, _args: ContinueArguments) -> JsResult<ContinueResponseBody> {
        self.resume()?;

        Ok(ContinueResponseBody {
            all_threads_continued: true,
        })
    }

    /// Handles the next (step over) request
    pub fn handle_next(&mut self, _args: NextArguments, frame_depth: usize) -> JsResult<()> {
        self.debugger
            .lock()
            .map_err(|e| {
                crate::JsNativeError::error().with_message(format!("Debugger mutex poisoned: {e}"))
            })?
            .step_over(frame_depth);
        self.running = true;
        self.stopped_reason = None;
        Ok(())
    }

    /// Handles the step in request
    pub fn handle_step_in(&mut self, _args: StepInArguments) -> JsResult<()> {
        self.debugger
            .lock()
            .map_err(|e| {
                crate::JsNativeError::error().with_message(format!("Debugger mutex poisoned: {e}"))
            })?
            .step_in();
        self.running = true;
        self.stopped_reason = None;
        Ok(())
    }

    /// Handles the step-out request
    pub fn handle_step_out(&mut self, _args: StepOutArguments, frame_depth: usize) -> JsResult<()> {
        self.debugger
            .lock()
            .map_err(|e| {
                crate::JsNativeError::error().with_message(format!("Debugger mutex poisoned: {e}"))
            })?
            .step_out(frame_depth);
        self.running = true;
        self.stopped_reason = None;
        Ok(())
    }

    /// Handles the stack trace request
    pub fn handle_stack_trace(
        &mut self,
        _args: StackTraceArguments,
    ) -> JsResult<StackTraceResponseBody> {
        let frames = match &self.eval_context {
            Some(ctx) => ctx
                .get_stack_trace()
                .map_err(|e| crate::JsNativeError::error().with_message(e))?,
            None => Vec::new(),
        };

        let stack_frames: Vec<StackFrame> = frames
            .iter()
            .enumerate()
            .map(|(i, frame)| {
                let source = Source {
                    name: Some(frame.function_name.clone()),
                    path: Some(frame.source_path.clone()),
                    source_reference: None,
                    presentation_hint: None,
                    origin: None,
                    sources: None,
                    adapter_data: None,
                    checksums: None,
                };

                StackFrame {
                    id: i as i64,
                    name: frame.function_name.clone(),
                    source: Some(source),
                    line: i64::from(frame.line_number),
                    column: i64::from(frame.column_number),
                    end_line: None,
                    end_column: None,
                    can_restart: false,
                    instruction_pointer_reference: Some(format!("{}", frame.pc)),
                    module_id: None,
                    presentation_hint: None,
                }
            })
            .collect();

        Ok(StackTraceResponseBody {
            stack_frames,
            total_frames: Some(frames.len() as i64),
        })
    }

    /// Handles the scopes request
    pub fn handle_scopes(&mut self, args: ScopesArguments) -> JsResult<ScopesResponseBody> {
        // Create variable references for different scopes
        let local_ref = self.next_variable_reference;
        self.next_variable_reference += 1;
        self.variable_references.insert(
            local_ref,
            VariableReference::Scope {
                frame_id: args.frame_id,
                scope_type: ScopeType::Local,
            },
        );

        let global_ref = self.next_variable_reference;
        self.next_variable_reference += 1;
        self.variable_references.insert(
            global_ref,
            VariableReference::Scope {
                frame_id: args.frame_id,
                scope_type: ScopeType::Global,
            },
        );

        let scopes = vec![
            Scope {
                name: "Local".to_string(),
                presentation_hint: Some("locals".to_string()),
                variables_reference: local_ref,
                named_variables: None,
                indexed_variables: None,
                expensive: false,
                source: None,
                line: None,
                column: None,
                end_line: None,
                end_column: None,
            },
            Scope {
                name: "Global".to_string(),
                presentation_hint: Some("globals".to_string()),
                variables_reference: global_ref,
                named_variables: None,
                indexed_variables: None,
                expensive: false,
                source: None,
                line: None,
                column: None,
                end_line: None,
                end_column: None,
            },
        ];

        Ok(ScopesResponseBody { scopes })
    }

    /// Handles the variables request
    pub fn handle_variables(
        &mut self,
        _args: VariablesArguments,
    ) -> JsResult<VariablesResponseBody> {
        // TODO: Implement variable inspection using DebuggerFrame::eval()
        // For now, return empty list
        Ok(VariablesResponseBody { variables: vec![] })
    }

    /// Handles the evaluate request
    pub fn handle_evaluate(&mut self, args: &EvaluateArguments) -> JsResult<EvaluateResponseBody> {
        let result = if let Some(ctx) = &self.eval_context {
            ctx.evaluate(args.expression.clone())
                .map_err(|e| crate::JsNativeError::error().with_message(e))?
        } else {
            let expression = &args.expression;
            format!("Evaluation context not initialized: {expression}")
        };

        Ok(EvaluateResponseBody {
            result,
            type_: Some("string".to_string()),
            presentation_hint: None,
            variables_reference: 0,
            named_variables: None,
            indexed_variables: None,
        })
    }

    /// Handles the threads request
    pub fn handle_threads(&mut self) -> JsResult<ThreadsResponseBody> {
        Ok(ThreadsResponseBody {
            threads: vec![Thread {
                id: self.thread_id,
                name: "Main Thread".to_string(),
            }],
        })
    }

    /// Notifies the session that execution has stopped
    pub fn notify_stopped(&mut self, reason: String) {
        self.running = false;
        self.stopped_reason = Some(reason);
    }

    /// Gets the current thread ID
    #[must_use]
    pub fn thread_id(&self) -> i64 {
        self.thread_id
    }

    /// Checks if the session is running
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Gets the stopped reason
    #[must_use]
    pub fn stopped_reason(&self) -> Option<&str> {
        self.stopped_reason.as_deref()
    }
}
