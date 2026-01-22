//! Debug evaluation context
//!
//! This module provides a dedicated thread for JavaScript evaluation with the `Context`.
//! Similar to the actor model, this ensures `Context` never needs to be `Send`/`Sync`.

use crate::{Context, JsResult, Source, context::ContextBuilder, dbg_log};
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;

/// Event that can be sent from eval thread to DAP server
#[derive(Debug, Clone)]
pub enum DebugEvent {
    /// Execution stopped (paused)
    Stopped {
        /// Reason for stopping, e.g. "step", "pause"
        reason: String,
        /// Optional description of the stop event
        description: Option<String>,
    },
    /// Program execution completed normally
    Terminated,
    /// Shutdown signal to terminate event forwarder thread
    Shutdown,
}

/// Task to be executed in the evaluation thread
pub(super) enum EvalTask {
    /// Execute JavaScript code (blocking - waits for result)
    Execute {
        source: String,
        result_tx: mpsc::Sender<Result<String, String>>,
    },
    /// Execute JavaScript code non-blocking (doesn't wait for result)
    /// Used for program execution that may hit breakpoints
    ExecuteNonBlocking { file_path: String },
    /// Get stack trace
    GetStackTrace {
        result_tx: mpsc::Sender<Result<Vec<StackFrameInfo>, String>>,
    },
    /// Evaluate expression in current frame
    Evaluate {
        expression: String,
        result_tx: mpsc::Sender<Result<String, String>>,
    },
    /// Terminate the evaluation thread
    Terminate,
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrameInfo {
    /// The name of the function in this frame
    pub function_name: String,
    /// The path to the source file
    pub source_path: String,
    /// The line number in the source file
    pub line_number: u32,
    /// The column number in the source file
    pub column_number: u32,
    /// The program counter (bytecode offset)
    pub pc: usize,
}

/// Debug evaluation context that runs in a dedicated thread
pub struct DebugEvalContext {
    task_tx: mpsc::Sender<EvalTask>,
    handle: Option<thread::JoinHandle<()>>,
    condvar: Arc<Condvar>,
    debugger: Arc<Mutex<crate::debugger::Debugger>>,
    /// Sender for debug events (kept to send shutdown signal)
    event_tx: mpsc::Sender<DebugEvent>,
}

/// Type for context setup function that can be sent across threads
type ContextSetup = Box<dyn FnOnce(&mut Context) -> JsResult<()> + Send>;

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for DebugEvalContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugEvalContext")
            .field("task_tx", &self.task_tx)
            .field("handle", &self.handle.is_some())
            .field("debugger", &"Arc<Mutex<Debugger>>")
            .field("condvar", &"Arc<Condvar>")
            .finish()
    }
}

impl DebugEvalContext {
    /// Creates a new debug evaluation context.
    ///
    /// Takes a setup function that will be called after `Context` is built in the eval thread.
    /// Returns `(DebugEvalContext, Receiver<DebugEvent>)` - the receiver should be used to listen for events.
    pub fn new(
        context_setup: ContextSetup,
        debugger: Arc<Mutex<crate::debugger::Debugger>>,
        condvar: Arc<Condvar>,
    ) -> JsResult<(Self, mpsc::Receiver<DebugEvent>)> {
        let (task_tx, task_rx) = mpsc::channel::<EvalTask>();
        let (event_tx, event_rx) = mpsc::channel::<DebugEvent>();

        // Clone event_tx for the hooks, keep one for the struct
        let event_tx_for_hooks = event_tx.clone();

        // Clone event_tx for the message loop, keep one for the struct
        let event_tx_for_message_loop = event_tx.clone();

        // Clone Arc references for the thread
        let debugger_clone = debugger.clone();
        let condvar_clone = condvar.clone();

        // Wrap task_rx in Arc<Mutex> for sharing with hooks
        let task_rx = Arc::new(Mutex::new(task_rx));
        let task_rx_clone = task_rx.clone();

        let handle = thread::spawn(move || {
            // Set up debug hooks
            let hooks = std::rc::Rc::new(DebugHooks {
                debugger: debugger_clone.clone(),
                condvar: condvar_clone.clone(),
                event_tx: event_tx_for_hooks,
                task_rx: task_rx_clone,
            });

            // Build the context with debug hooks IN THIS THREAD
            let mut context = match ContextBuilder::new().host_hooks(hooks).build() {
                Ok(ctx) => ctx,
                Err(e) => {
                    dbg_log!("[DebugEvalContext] Failed to build context: {e}");
                    return;
                }
            };

            // Call the setup function to register console and other runtimes
            if let Err(e) = context_setup(&mut context) {
                dbg_log!("[DebugEvalContext] Context setup failed: {e}");
                return;
            }

            // Attach the debugger to the context
            let attach_result = debugger_clone
                .lock()
                .map_err(|e| format!("Debugger mutex poisoned: {e}"))
                .and_then(|mut dbg| dbg.attach(&mut context).map_err(|e| e.to_string()));

            if let Err(e) = attach_result {
                dbg_log!("[DebugEvalContext] Failed to attach debugger: {e}");
                return;
            }

            dbg_log!("[DebugEvalContext] Context created and debugger attached");

            // Process tasks
            loop {
                let Some(task) = task_rx
                    .lock()
                    .map_err(|e| dbg_log!("[DebugEvalContext] Task receiver mutex poisoned: {e}"))
                    .ok()
                    .and_then(|rx| rx.recv().ok())
                else {
                    break;
                };

                match task {
                    EvalTask::Execute { source, result_tx } => {
                        let result = context.eval(Source::from_bytes(&source));
                        // Convert JsResult<JsValue> to Result<String, String> for sending
                        let send_result = match result {
                            Ok(v) => match v.to_string(&mut context) {
                                Ok(js_str) => Ok(js_str.to_std_string_escaped()),
                                Err(e) => Err(e.to_string()),
                            },
                            Err(e) => Err(e.to_string()),
                        };
                        drop(result_tx.send(send_result));
                    }
                    EvalTask::ExecuteNonBlocking { file_path } => {
                        dbg_log!(
                            "[DebugEvalContext] Starting non-blocking execution of {file_path}"
                        );

                        // Convert string to Path and create Source from a file
                        let path = Path::new(&file_path);
                        let source = match Source::from_filepath(path) {
                            Ok(src) => src,
                            Err(e) => {
                                dbg_log!("[DebugEvalContext] Failed to load file: {e}");
                                continue;
                            }
                        };

                        // Execute the source
                        let result = context.eval(source);

                        match result {
                            Ok(v) => {
                                if v.is_undefined() {
                                    dbg_log!("[DebugEvalContext] Execution completed");
                                } else {
                                    let display = v.display();
                                    dbg_log!(
                                        "[DebugEvalContext] Execution completed with result: {display}"
                                    );
                                }
                            }
                            Err(e) => {
                                dbg_log!("[DebugEvalContext] Execution error: {e}");
                            }
                        }

                        // Run any pending jobs (promises, etc.)
                        if let Err(e) = context.run_jobs() {
                            dbg_log!("[DebugEvalContext] Job execution error: {e}");
                        }

                        // Send terminated event to signal program completed
                        dbg_log!(
                            "[DebugEvalContext] Program execution completed, sending terminated event"
                        );
                        drop(event_tx_for_message_loop.send(DebugEvent::Terminated));
                    }
                    EvalTask::Terminate => {
                        dbg_log!("[DebugEvalContext] Terminating evaluation thread");
                        break;
                    }
                    // Handle inspection tasks using a common helper
                    other => {
                        DebugHooks::process_inspection_task(other, &mut context);
                    }
                }
            } // End task processing loop
        });

        let ctx = Self {
            task_tx,
            handle: Some(handle),
            condvar,
            debugger,
            event_tx,
        };

        Ok((ctx, event_rx))
    }

    /// Executes JavaScript code in the evaluation thread (blocking).
    ///
    /// This will wait for the result, so it should NOT be used for program execution
    /// that may hit breakpoints. Use `execute_async` instead.
    pub fn execute(&self, source: String) -> Result<String, String> {
        let (result_tx, result_rx) = mpsc::channel();

        self.task_tx
            .send(EvalTask::Execute { source, result_tx })
            .map_err(|e| format!("Failed to send task: {e}"))?;

        // This will block the current thread until the result is received
        result_rx
            .recv()
            .map_err(|e| format!("Failed to receive result: {e}"))?
    }

    /// Executes JavaScript code asynchronously without blocking.
    ///
    /// The execution happens in the eval thread and this method returns immediately.
    /// Use this for program execution that may hit breakpoints.
    pub fn execute_async(&self, file_path: String) -> Result<(), String> {
        self.task_tx
            .send(EvalTask::ExecuteNonBlocking { file_path })
            .map_err(|e| format!("Failed to send task: {e}"))?;

        Ok(())
    }

    /// Gets the current stack trace from the evaluation thread
    pub fn get_stack_trace(&self) -> Result<Vec<StackFrameInfo>, String> {
        let (result_tx, result_rx) = mpsc::channel();

        self.task_tx
            .send(EvalTask::GetStackTrace { result_tx })
            .map_err(|e| format!("Failed to send task: {e}"))?;

        // Notify condvar ONLY if the debugger is paused
        // This wakes wait_for_resume to process the task immediately
        if self
            .debugger
            .lock()
            .map_err(|e| format!("Debugger mutex poisoned: {e}"))?
            .is_paused()
        {
            self.condvar.notify_all();
        }

        result_rx
            .recv()
            .map_err(|e| format!("Failed to receive result: {e}"))?
    }

    /// Evaluates an expression in the current frame
    pub fn evaluate(&self, expression: String) -> Result<String, String> {
        let (result_tx, result_rx) = mpsc::channel();

        self.task_tx
            .send(EvalTask::Evaluate {
                expression,
                result_tx,
            })
            .map_err(|e| format!("Failed to send task: {e}"))?;

        // Notify condvar ONLY if the debugger is paused
        // This wakes wait_for_resume to process the task immediately
        if self
            .debugger
            .lock()
            .map_err(|e| format!("Debugger mutex poisoned: {e}"))?
            .is_paused()
        {
            self.condvar.notify_all();
        }

        result_rx
            .recv()
            .map_err(|e| format!("Failed to receive result: {e}"))?
    }
}

impl Drop for DebugEvalContext {
    fn drop(&mut self) {
        dbg_log!("[DebugEvalContext] Dropping - initiating shutdown");

        // Signal shutdown to break any wait_for_resume loops
        // In Drop, we can't propagate errors, so we log and continue
        {
            match self.debugger.lock() {
                Ok(mut debugger) => debugger.shutdown(),
                Err(e) => {
                    dbg_log!("[DebugEvalContext] Debugger mutex poisoned during drop: {e}");
                }
            }
        }

        // Wake up any threads waiting on the condvar
        self.condvar.notify_all();

        // Send shutdown event to terminate any event forwarder threads
        drop(self.event_tx.send(DebugEvent::Shutdown));

        // Send terminate signal to eval thread
        drop(self.task_tx.send(EvalTask::Terminate));

        // Wait for the thread to finish with a timeout
        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Ok(()) => dbg_log!("[DebugEvalContext] Thread joined successfully"),
                Err(e) => dbg_log!("[DebugEvalContext] Thread join failed: {e:?}"),
            }
        }
    }
}

/// Host hooks for the debug evaluation context
struct DebugHooks {
    debugger: Arc<Mutex<crate::debugger::Debugger>>,
    condvar: Arc<Condvar>,
    event_tx: mpsc::Sender<DebugEvent>,
    task_rx: Arc<Mutex<mpsc::Receiver<EvalTask>>>,
}

impl crate::context::HostHooks for DebugHooks {
    fn on_debugger_statement(&self, context: &mut Context) -> JsResult<()> {
        let frame = crate::debugger::DebugApi::get_current_frame(context);
        dbg_log!("[DebugHooks] Debugger statement hit at {frame}");

        // Pause execution
        self.debugger
            .lock()
            .map_err(|e| {
                crate::JsNativeError::error().with_message(format!("Debugger mutex poisoned: {e}"))
            })?
            .pause();

        // Send stopped event to DAP server
        drop(self.event_tx.send(DebugEvent::Stopped {
            reason: "pause".to_string(),
            description: Some(format!("Paused on debugger statement at {frame}")),
        }));

        // Wait for resume using condition variable
        // Returns error if shutting down
        // Passes context to allow processing inspection tasks while paused
        self.wait_for_resume(context)?;

        Ok(())
    }

    fn on_step(&self, context: &mut Context) -> JsResult<()> {
        if self
            .debugger
            .lock()
            .map_err(|e| {
                crate::JsNativeError::error().with_message(format!("Debugger mutex poisoned: {e}"))
            })?
            .is_paused()
        {
            dbg_log!("[DebugHooks] Paused - waiting for resume...");

            // Send stopped event to DAP server
            drop(self.event_tx.send(DebugEvent::Stopped {
                reason: "step".to_string(),
                description: Some("Paused on step".to_string()),
            }));

            // Returns error if shutting down
            // Passes context to allow processing inspection tasks while paused
            self.wait_for_resume(context)?;
        }

        Ok(())
    }
}

impl DebugHooks {
    /// Process inspection tasks (`GetStackTrace`, `Evaluate`) that can run while paused.
    ///
    /// Returns `true` if a task was processed, `false` if it should be skipped.
    fn process_inspection_task(task: EvalTask, context: &mut Context) -> bool {
        match task {
            EvalTask::GetStackTrace { result_tx } => {
                let stack = crate::debugger::DebugApi::get_call_stack(context);
                let frames = stack
                    .iter()
                    .map(|frame| StackFrameInfo {
                        function_name: frame.function_name().to_std_string_escaped(),
                        source_path: frame.source_path().to_string(),
                        line_number: frame.line_number().unwrap_or(0),
                        column_number: frame.column_number().unwrap_or(0),
                        pc: frame.pc() as usize,
                    })
                    .collect();
                drop(result_tx.send(Ok(frames)));
                true
            }
            EvalTask::Evaluate {
                expression,
                result_tx,
            } => {
                // TODO: Implement proper frame evaluation
                drop(result_tx.send(Ok(format!("Evaluation not yet implemented: {expression}"))));
                true
            }
            _ => false, // Not an inspection task
        }
    }

    /// Wait for resume while continuing to process inspection tasks.
    ///
    /// This prevents deadlock when the DAP client requests `stackTrace`/`evaluate` while paused.
    fn wait_for_resume(&self, context: &mut Context) -> JsResult<()> {
        dbg_log!("[DebugHooks] Entering wait_for_resume - will process tasks while waiting");

        loop {
            // Process any pending inspection tasks before waiting
            // Do this WITHOUT holding debugger lock to avoid contention
            loop {
                let try_recv_result = self
                    .task_rx
                    .lock()
                    .map_err(|e| {
                        crate::JsNativeError::error()
                            .with_message(format!("Task receiver mutex poisoned: {e}"))
                    })
                    .map(|rx| rx.try_recv());

                match try_recv_result {
                    Ok(Ok(task)) => {
                        // Handle non-inspection tasks specially
                        match task {
                            EvalTask::Execute { .. } | EvalTask::ExecuteNonBlocking { .. } => {
                                dbg_log!(
                                    "[DebugHooks] Dropping execution task received while paused"
                                );
                            }
                            EvalTask::Terminate => {
                                dbg_log!("[DebugHooks] Terminate signal received while paused");
                                return Err(crate::JsNativeError::error()
                                    .with_message("Eval thread terminating")
                                    .into());
                            }
                            // Process inspection tasks
                            other => {
                                if Self::process_inspection_task(other, context) {
                                    dbg_log!("[DebugHooks] Processed inspection task while paused");
                                }
                            }
                        }
                    }
                    Ok(Err(mpsc::TryRecvError::Empty)) => {
                        // No more pending tasks - exit drain loop
                        break;
                    }
                    Ok(Err(mpsc::TryRecvError::Disconnected)) => {
                        dbg_log!("[DebugHooks] Task channel disconnected");
                        return Err(crate::JsNativeError::error()
                            .with_message("Task channel closed")
                            .into());
                    }
                    Err(e) => {
                        // Mutex error - convert JsNativeError to JsError
                        return Err(e.into());
                    }
                }
            }

            // NOW lock debugger once to check state and wait
            // This is the ONLY lock acquisition per loop iteration
            let mut debugger_guard = self.debugger.lock().map_err(|e| {
                crate::JsNativeError::error().with_message(format!("Debugger mutex poisoned: {e}"))
            })?;

            // Check if we should exit
            if debugger_guard.is_paused() {
                // Check for shutdown
                if debugger_guard.is_shutting_down() {
                    dbg_log!("[DebugHooks] Shutting down - aborting execution");
                    return Err(crate::JsNativeError::error()
                        .with_message("Debugger shutting down")
                        .into());
                }
            } else {
                dbg_log!("[DebugHooks] Resumed!");
                return Ok(());
            }

            // Still paused - wait on condvar (keeps debugger_guard locked)
            // Will be woken by:
            // 1. resume() - to continue execution
            // 2. notify_all() from get_stack_trace/evaluate - to process inspection tasks
            // 3. shutdown() - to terminate cleanly
            dbg_log!("[DebugHooks] Waiting on condvar...");
            debugger_guard = self.condvar.wait(debugger_guard).map_err(|e| {
                crate::JsNativeError::error()
                    .with_message(format!("Condvar wait failed (mutex poisoned): {e}"))
            })?;
            dbg_log!("[DebugHooks] Condvar woken - checking for tasks and state");

            // Explicitly drop to show we're releasing the lock before the next iteration
            drop(debugger_guard);
        }
    }
}
