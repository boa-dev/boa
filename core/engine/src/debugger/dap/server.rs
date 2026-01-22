//! DAP server implementation
//!
//! This module implements the Debug Adapter Protocol server that handles
//! JSON-RPC communication with DAP clients (like VS Code).

use super::{
    Event, ProtocolMessage, Request, Response,
    messages::{
        AttachRequestArguments, ContinueArguments, EvaluateArguments, InitializeRequestArguments,
        LaunchRequestArguments, NextArguments, ScopesArguments, SetBreakpointsArguments,
        SourceArguments, StackTraceArguments, StepInArguments, StepOutArguments,
        VariablesArguments,
    },
    session::DebugSession,
};
use crate::{JsError, JsNativeError, dbg_log};
use std::sync::{Arc, Mutex};

/// DAP server that handles protocol communication
#[derive(Debug)]
pub struct DapServer {
    /// The debug session
    session: Arc<Mutex<DebugSession>>,

    /// Sequence number for responses and events
    seq: i64,

    /// Whether the server has been initialized
    initialized: bool,
}

impl DapServer {
    /// Creates a new DAP server
    pub fn new(session: Arc<Mutex<DebugSession>>) -> Self {
        Self {
            session,
            seq: 1,
            initialized: false,
        }
    }

    /// Gets the next sequence number
    fn next_seq(&mut self) -> i64 {
        let seq = self.seq;
        self.seq += 1;
        seq
    }

    /// Handles a DAP request and returns responses/events
    pub fn handle_request(&mut self, request: Request) -> Vec<ProtocolMessage> {
        let command = request.command.clone();
        let request_seq = request.seq;

        dbg_log!(
            "[BOA-DAP-DEBUG] Received request: {}",
            serde_json::to_string(&request)
                .unwrap_or_else(|_| format!("{{\"command\":\"{command}\"}}"))
        );

        let result = match command.as_str() {
            "initialize" => self.handle_initialize(&request),
            "launch" => self.handle_launch(&request),
            "attach" => self.handle_attach(&request),
            "configurationDone" => {
                return self.handle_configuration_done(&request);
            }
            "setBreakpoints" => self.handle_set_breakpoints(&request),
            "continue" => self.handle_continue(&request),
            "next" => self.handle_next(&request),
            "stepIn" => self.handle_step_in(&request),
            "stepOut" => self.handle_step_out(&request),
            "stackTrace" => self.handle_stack_trace(&request),
            "scopes" => self.handle_scopes(&request),
            "variables" => self.handle_variables(&request),
            "evaluate" => self.handle_evaluate(&request),
            "threads" => self.handle_threads(&request),
            "source" => self.handle_source(&request),
            "disconnect" => {
                return vec![self.create_response(request_seq, &command, true, None, None)];
            }
            _ => {
                return vec![self.create_response(
                    request_seq,
                    &command,
                    false,
                    Some(format!("Unknown command: {command}")),
                    None,
                )];
            }
        };

        match result {
            Ok(messages) => messages,
            Err(err) => {
                vec![self.create_response(
                    request_seq,
                    &command,
                    false,
                    Some(err.to_string()),
                    None,
                )]
            }
        }
    }

    fn handle_initialize(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: InitializeRequestArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        let capabilities = self
            .session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_initialize(args)?;
        self.initialized = true;

        let body = serde_json::to_value(capabilities)
            .map_err(|e| JsNativeError::typ().with_message(format!("Failed to serialize: {e}")))?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            Some(body),
        )])
    }

    fn handle_launch(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: LaunchRequestArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        // Note: In practice, dap.rs intercepts launch and handles context creation
        // This path is just for completeness
        let setup = Box::new(|_ctx: &mut crate::Context| Ok(()));
        let event_handler = Box::new(|_event| {}); // No-op event handler for stdio mode
        self.session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_launch(&args, setup, event_handler)?;

        // For stdio mode in engine, we don't use events
        // TCP mode (in CLI) provides an actual event handler

        // No execution result since execution happens asynchronously
        let body = None;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            body,
        )])
    }

    fn handle_attach(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: AttachRequestArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        self.session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_attach(args)?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            None,
        )])
    }

    fn handle_configuration_done(&mut self, request: &Request) -> Vec<ProtocolMessage> {
        vec![self.create_response(request.seq, &request.command, true, None, None)]
    }

    fn handle_set_breakpoints(
        &mut self,
        request: &Request,
    ) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: SetBreakpointsArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        let response_body = self
            .session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_set_breakpoints(&args)?;

        let body = serde_json::to_value(response_body)
            .map_err(|e| JsNativeError::typ().with_message(format!("Failed to serialize: {e}")))?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            Some(body),
        )])
    }

    fn handle_continue(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: ContinueArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        let response_body = self
            .session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_continue(args)?;

        let body = serde_json::to_value(response_body)
            .map_err(|e| JsNativeError::typ().with_message(format!("Failed to serialize: {e}")))?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            Some(body),
        )])
    }

    fn handle_next(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: NextArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        // TODO: Get actual frame depth from context
        self.session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_next(args, 0)?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            None,
        )])
    }

    fn handle_step_in(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: StepInArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        self.session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_step_in(args)?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            None,
        )])
    }

    fn handle_step_out(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: StepOutArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        // TODO: Get actual frame depth from context
        self.session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_step_out(args, 0)?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            None,
        )])
    }

    fn handle_stack_trace(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: StackTraceArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        let response_body = self
            .session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_stack_trace(args)?;

        let body = serde_json::to_value(response_body)
            .map_err(|e| JsNativeError::typ().with_message(format!("Failed to serialize: {e}")))?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            Some(body),
        )])
    }

    fn handle_scopes(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: ScopesArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        let response_body = self
            .session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_scopes(args)?;

        let body = serde_json::to_value(response_body)
            .map_err(|e| JsNativeError::typ().with_message(format!("Failed to serialize: {e}")))?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            Some(body),
        )])
    }

    fn handle_variables(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: VariablesArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        let response_body = self
            .session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_variables(args)?;

        let body = serde_json::to_value(response_body)
            .map_err(|e| JsNativeError::typ().with_message(format!("Failed to serialize: {e}")))?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            Some(body),
        )])
    }

    fn handle_evaluate(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: EvaluateArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        let response_body = self
            .session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_evaluate(&args)?;

        let body = serde_json::to_value(response_body)
            .map_err(|e| JsNativeError::typ().with_message(format!("Failed to serialize: {e}")))?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            Some(body),
        )])
    }

    fn handle_threads(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let response_body = self
            .session
            .lock()
            .map_err(|e| {
                JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
            })?
            .handle_threads()?;

        let body = serde_json::to_value(response_body)
            .map_err(|e| JsNativeError::typ().with_message(format!("Failed to serialize: {e}")))?;

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            Some(body),
        )])
    }

    fn handle_source(&mut self, request: &Request) -> Result<Vec<ProtocolMessage>, JsError> {
        let args: SourceArguments =
            serde_json::from_value(request.arguments.clone().unwrap_or(serde_json::Value::Null))
                .map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid arguments: {e}"))
                })?;

        // Get the source path from arguments
        let source_path = if let Some(source) = &args.source {
            if let Some(path) = &source.path {
                path.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Handle special case: "replinput" is for REPL/debug console input
        // This doesn't have an actual source file content, return empty
        #[allow(clippy::doc_markdown)]
        let content = if source_path == "replinput" {
            String::new()
        } else if !source_path.is_empty() {
            // Try to read the actual file
            match std::fs::read_to_string(&source_path) {
                Ok(content) => content,
                Err(e) => {
                    return Err(JsNativeError::typ()
                        .with_message(format!("Failed to read source file {source_path}: {e}"))
                        .into());
                }
            }
        } else {
            // No path provided, check if we have a program path from launch
            let program_path = self
                .session
                .lock()
                .map_err(|e| {
                    JsNativeError::error().with_message(format!("DebugSession mutex poisoned: {e}"))
                })?
                .get_program_path()
                .map(ToString::to_string);

            if let Some(path) = program_path {
                match std::fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(e) => {
                        return Err(JsNativeError::typ()
                            .with_message(format!("Failed to read program file: {e}"))
                            .into());
                    }
                }
            } else {
                String::new()
            }
        };

        // Return success response with source content
        let body = serde_json::json!({
            "content": content,
            "mimeType": "text/javascript"
        });

        Ok(vec![self.create_response(
            request.seq,
            &request.command,
            true,
            None,
            Some(body),
        )])
    }

    /// Creates a response message
    fn create_response(
        &mut self,
        request_seq: i64,
        command: &str,
        success: bool,
        message: Option<String>,
        body: Option<serde_json::Value>,
    ) -> ProtocolMessage {
        ProtocolMessage::Response(Response {
            seq: self.next_seq(),
            request_seq,
            success,
            command: command.to_string(),
            message,
            body,
        })
    }

    /// Creates an event message
    pub fn create_event(
        &mut self,
        event: &str,
        body: Option<serde_json::Value>,
    ) -> ProtocolMessage {
        ProtocolMessage::Event(Event {
            seq: self.next_seq(),
            event: event.to_string(),
            body,
        })
    }
}
