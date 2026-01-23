//! DAP debugger for Boa CLI
//!
//! This module provides the Debug Adapter Protocol integration for the Boa CLI.
//! It intercepts DAP messages, manages the JavaScript context with runtime,
//! and handles execution and output capture.

use boa_engine::{
    Context, JsResult, dbg_log,
    debugger::{
        Debugger,
        dap::{
            DapServer, DebugEvent, Event, ProtocolMessage, Request, Response,
            messages::{LaunchRequestArguments, OutputEventBody, StoppedEventBody},
            session::DebugSession,
        },
    },
    js_error,
};
use boa_gc::{Finalize, Trace};
use boa_runtime::console::{Console, ConsoleState, Logger};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Runs the DAP server on the specified TCP port
///
/// This creates a debugger instance, wraps it in a `DebugSession`,
/// and runs the `DapServer` to handle all protocol communication.
/// The `DapServer` in `boa_engine` handles all DAP messages, breakpoints,
/// stepping, variable inspection, etc.
///
/// Set `BOA_DAP_DEBUG=1` environment variable to enable debug logging.
pub(crate) fn run_dap_server_with_mode(port: u16) -> JsResult<()> {
    dbg_log!("[DAP] Starting Boa Debug Adapter (TCP on port {port})");

    // Run TCP server
    run_tcp_server(port).map_err(|e| js_error!("TCP server error: {}", e))?;

    dbg_log!("[DAP] Server stopped");
    Ok(())
}

/// Runs the DAP server as a TCP server (raw socket, not HTTP)
/// Creates a new `DebugSession` for each accepted connection
fn run_tcp_server(port: u16) -> io::Result<()> {
    use std::net::TcpListener;

    let addr = format!("127.0.0.1:{port}");
    dbg_log!("[BOA-DAP] Starting TCP server on {addr}");

    let listener = TcpListener::bind(&addr)?;
    dbg_log!("[BOA-DAP] Server listening on {addr}");
    dbg_log!("[BOA-DAP] Ready to accept connections");

    // Accept connections in a loop
    loop {
        match listener.accept() {
            Ok((stream, peer_addr)) => {
                dbg_log!("[BOA-DAP] Client connected from {peer_addr}");

                // Handle this client connection with its own session
                if let Err(e) = handle_tcp_client(stream) {
                    dbg_log!("[BOA-DAP] Client handler error: {e}");
                    // Continue accepting new connections even if one fails
                    continue;
                }

                dbg_log!("[BOA-DAP] Client session ended");
                // Continue accepting more connections (removed break)
            }
            Err(e) => {
                dbg_log!("[BOA-DAP] Error accepting connection: {e}");
                return Err(e);
            }
        }
    }
}

/// Custom Logger that sends console output directly as DAP output events
#[derive(Clone, Trace, Finalize)]
struct DapLogger<W: Write + 'static> {
    /// TCP writer for sending DAP messages
    #[unsafe_ignore_trace]
    writer: Arc<Mutex<W>>,

    /// Sequence counter for DAP messages
    #[unsafe_ignore_trace]
    seq_counter: Arc<Mutex<i64>>,
}

impl<W: Write + 'static> DapLogger<W> {
    fn new(writer: Arc<Mutex<W>>, seq_counter: Arc<Mutex<i64>>) -> Self {
        Self {
            writer,
            seq_counter,
        }
    }

    fn send_output(&self, msg: String, category: &str) -> io::Result<()> {
        // Create an output event
        let seq = {
            let mut counter = self.seq_counter.lock().map_err(|e| {
                io::Error::other(format!("DapLogger seq_counter mutex poisoned: {e}"))
            })?;
            let current = *counter;
            *counter += 1;
            current
        };

        let output_event = Event {
            seq,
            event: "output".to_string(),
            body: Some(
                serde_json::to_value(OutputEventBody {
                    category: Some(category.to_string()),
                    output: msg + "\n",
                    group: None,
                    variables_reference: None,
                    source: None,
                    line: None,
                    column: None,
                    data: None,
                })
                .expect("Failed to serialize output event body"),
            ),
        };

        let output_message = ProtocolMessage::Event(output_event);

        // Send it immediately to the TCP stream
        let mut writer = self
            .writer
            .lock()
            .map_err(|e| io::Error::other(format!("DapLogger writer mutex poisoned: {e}")))?;
        send_message_internal(&output_message, &mut *writer)?;
        Ok(())
    }
}

impl<W: Write + 'static> Logger for DapLogger<W> {
    fn log(&self, msg: String, _state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
        self.send_output(msg, "stdout")
            .map_err(|e| js_error!("Failed to send log output: {}", e))?;
        Ok(())
    }

    fn info(&self, msg: String, _state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
        self.send_output(msg, "stdout")
            .map_err(|e| js_error!("Failed to send info output: {}", e))?;
        Ok(())
    }

    fn warn(&self, msg: String, _state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
        self.send_output(msg, "console")
            .map_err(|e| js_error!("Failed to send warn output: {}", e))?;
        Ok(())
    }

    fn error(&self, msg: String, _state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
        self.send_output(msg, "stderr")
            .map_err(|e| js_error!("Failed to send error output: {}", e))?;
        Ok(())
    }
}

/// Internal function to send a DAP message (used by logger)
fn send_message_internal<W: Write>(message: &ProtocolMessage, writer: &mut W) -> io::Result<()> {
    let json = serde_json::to_string(message).unwrap_or_else(|_| "{}".to_string());

    dbg_log!("[BOA-DAP] Output Event: {json}");

    write!(writer, "Content-Length: {}\r\n\r\n{}", json.len(), json)?;
    writer.flush()?;
    Ok(())
}

/// Handle a single TCP client connection using DAP protocol
#[allow(clippy::too_many_lines)]
fn handle_tcp_client(stream: std::net::TcpStream) -> io::Result<()> {
    use std::io::{BufRead, BufReader, Read};

    // Create a new debugger and session for this connection
    let debugger = Arc::new(Mutex::new(Debugger::new()));
    let session = Arc::new(Mutex::new(DebugSession::new(debugger.clone())));

    let mut reader = BufReader::new(stream.try_clone()?);
    let writer = Arc::new(Mutex::new(stream));

    let mut dap_server = DapServer::new(session.clone());

    loop {
        // Read the Content-Length header
        let mut header = String::new();
        match reader.read_line(&mut header) {
            Ok(0) => {
                dbg_log!("[BOA-DAP] Client disconnected");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                dbg_log!("[BOA-DAP] Error reading header: {e}");
                break;
            }
        }

        if header.trim().is_empty() {
            continue;
        }

        let content_length: usize = if let Some(len) = header
            .trim()
            .strip_prefix("Content-Length: ")
            .and_then(|s| s.parse().ok())
        {
            len
        } else {
            dbg_log!("[BOA-DAP] Invalid Content-Length header: {header}");
            continue;
        };

        // Read the empty line separator
        let mut empty = String::new();
        reader.read_line(&mut empty)?;

        // Read the message body
        let mut buffer = vec![0u8; content_length];
        reader.read_exact(&mut buffer)?;

        if let Ok(body_str) = String::from_utf8(buffer.clone()) {
            dbg_log!("[BOA-DAP] Request: {body_str}");
        }

        // Parse DAP message
        match serde_json::from_slice::<ProtocolMessage>(&buffer) {
            Ok(ProtocolMessage::Request(dap_request)) => {
                // Check if this is a terminated request - end the session
                if dap_request.command == "terminate" {
                    dbg_log!("[BOA-DAP] Terminate request received, ending session");

                    // Send success response
                    let response = ProtocolMessage::Response(Response {
                        seq: 0,
                        request_seq: dap_request.seq,
                        success: true,
                        command: dap_request.command,
                        message: None,
                        body: None,
                    });

                    let mut w = writer.lock().map_err(|e| {
                        io::Error::other(format!("Writer mutex poisoned in terminate handler: {e}"))
                    })?;
                    send_dap_message(&response, &mut *w)?;

                    // Break from the loop to end the session
                    break;
                }

                // Check if this is a launch request - we need to create Context here
                let responses = if dap_request.command == "launch" {
                    handle_launch_request_with_context(
                        dap_request,
                        &mut dap_server,
                        session.clone(),
                        writer.clone(),
                    )?
                } else if dap_request.command == "configurationDone" {
                    // After configurationDone, execute the program
                    handle_configuration_done_with_execution(
                        dap_request,
                        &mut dap_server,
                        session.clone(),
                        writer.clone(),
                    )?
                } else {
                    // Process all other requests normally through the server
                    dap_server.handle_request(&dap_request)
                };

                // Send all responses
                for response in responses {
                    let mut w = writer.lock().map_err(|e| {
                        io::Error::other(format!("Writer mutex poisoned sending responses: {e}"))
                    })?;
                    send_dap_message(&response, &mut *w)?;
                }
            }
            Err(e) => {
                dbg_log!("[BOA-DAP] Failed to parse request: {e}");
            }
            _ => {
                dbg_log!("[BOA-DAP] Unexpected message type (not a request)");
            }
        }
    }

    Ok(())
}

/// Send a DAP protocol message
fn send_dap_message<W: Write>(message: &ProtocolMessage, writer: &mut W) -> io::Result<()> {
    let json = serde_json::to_string(message).unwrap_or_else(|_| "{}".to_string());

    dbg_log!("[BOA-DAP] Response: {json}");

    // Write with a Content-Length header
    write!(writer, "Content-Length: {}\r\n\r\n{}", json.len(), json)?;
    writer.flush()?;
    Ok(())
}

/// Handle launch request and create Context setup function with runtimes
#[allow(clippy::needless_pass_by_value)]
fn handle_launch_request_with_context<W: Write + Send + 'static>(
    request: Request,
    _dap_server: &mut DapServer,
    session: Arc<Mutex<DebugSession>>,
    writer: Arc<Mutex<W>>,
) -> io::Result<Vec<ProtocolMessage>> {
    // Parse launch arguments
    let launch_args: LaunchRequestArguments = if let Some(args) = &request.arguments {
        serde_json::from_value(args.clone())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else {
        LaunchRequestArguments {
            no_debug: None,
            program: None,
            args: None,
            cwd: None,
            env: None,
            stop_on_entry: None,
        }
    };

    // Create a setup function that will register console and other runtimes
    // This function will be called in the eval thread after Context is created
    let writer_clone = writer.clone();
    let context_setup = Box::new(move |context: &mut Context| -> JsResult<()> {
        // Create DAP logger and register console with it
        let logger = DapLogger::new(writer_clone.clone(), Arc::new(Mutex::new(1)));

        // Register console with the DAP logger
        Console::register_with_logger(logger, context)?;

        Ok(())
    });

    // Create event handler callback for TCP mode
    // This will be called by the forwarder thread in session.rs for each event
    let writer_clone = writer.clone();
    let event_handler = Box::new(move |event: DebugEvent| {
        match event {
            DebugEvent::Shutdown => {
                dbg_log!("[BOA-DAP] Event handler received shutdown");
            }
            DebugEvent::Stopped {
                reason,
                description,
            } => {
                dbg_log!("[BOA-DAP] Event handler sending stopped event: {reason}");

                // Convert to DAP protocol message
                let dap_message = ProtocolMessage::Event(Event {
                    seq: 0,
                    event: "stopped".to_string(),
                    body: Some(
                        serde_json::to_value(StoppedEventBody {
                            reason,
                            description,
                            thread_id: Some(1),
                            preserve_focus_hint: None,
                            text: None,
                            all_threads_stopped: true,
                            hit_breakpoint_ids: None,
                        })
                        .expect("Failed to serialize stopped event body"),
                    ),
                });

                // Send it immediately to the TCP stream
                match writer_clone.lock() {
                    Ok(mut w) => {
                        if let Err(e) = send_message_internal(&dap_message, &mut *w) {
                            dbg_log!("[BOA-DAP] Failed to send event: {e}");
                        }
                    }
                    Err(e) => {
                        dbg_log!("[BOA-DAP] Writer mutex poisoned in Stopped event: {e}");
                    }
                }
            }
            DebugEvent::Terminated => {
                dbg_log!("[BOA-DAP] Event handler sending terminated event");

                // Send terminated event - tells VS Code the debuggee has exited
                let dap_message = ProtocolMessage::Event(Event {
                    seq: 0,
                    event: "terminated".to_string(),
                    body: None,
                });

                // Send it immediately to the TCP stream
                match writer_clone.lock() {
                    Ok(mut w) => {
                        if let Err(e) = send_message_internal(&dap_message, &mut *w) {
                            dbg_log!("[BOA-DAP] Failed to send terminated event: {e}");
                        }
                    }
                    Err(e) => {
                        dbg_log!("[BOA-DAP] Writer mutex poisoned in Terminated event: {e}");
                    }
                }
            }
        }
    });

    // Call handle_launch - it will spawn a forwarder thread and execute program
    // Forwarder thread is spawned BEFORE program execution to avoid missing events
    {
        let mut sess = session
            .lock()
            .map_err(|e| io::Error::other(format!("DebugSession mutex poisoned in launch: {e}")))?;
        sess.handle_launch(&launch_args, context_setup, event_handler)
            .map_err(|e| io::Error::other(format!("Failed to handle launch: {e}")))?;
    };

    // No execution result to include since execution happens asynchronously
    let body = None;

    // Return a success response directly (don't call dap_server.handle_request)
    let response = ProtocolMessage::Response(Response {
        seq: 0,
        request_seq: request.seq,
        success: true,
        command: request.command,
        message: None,
        body,
    });

    Ok(vec![response])
}

/// Handle configurationDone request and execute the program
#[allow(clippy::needless_pass_by_value)]
fn handle_configuration_done_with_execution<W: Write + Send + 'static>(
    request: Request,
    dap_server: &mut DapServer,
    session: Arc<Mutex<DebugSession>>,
    writer: Arc<Mutex<W>>,
) -> io::Result<Vec<ProtocolMessage>> {
    // First, let the DAP server handle configurationDone normally
    let responses = dap_server.handle_request(&request);

    // Get the program path from the session
    let program_path = {
        let sess = session.lock().map_err(|e| {
            io::Error::other(format!(
                "DebugSession mutex poisoned getting program path: {e}"
            ))
        })?;
        sess.get_program_path().map(ToString::to_string)
    };

    if let Some(path) = program_path {
        dbg_log!("[DAP-CLI] Executing program: {path}");

        // Read the JavaScript file
        match std::fs::read_to_string(&path) {
            Ok(source) => {
                // Execute the program in the evaluation thread
                let sess = session.lock().map_err(|e| {
                    io::Error::other(format!("DebugSession mutex poisoned during execution: {e}"))
                })?;
                match sess.execute(source) {
                    Ok(_result) => {
                        dbg_log!("[DAP-CLI] Program executed successfully");

                        // Send terminated event
                        let mut w = writer.lock().map_err(|e| {
                            io::Error::other(format!("Writer mutex poisoned after execution: {e}"))
                        })?;
                        let terminated_event = ProtocolMessage::Event(Event {
                            seq: 0,
                            event: "terminated".to_string(),
                            body: None,
                        });
                        if let Err(e) = send_dap_message(&terminated_event, &mut *w) {
                            dbg_log!("[DAP-CLI] Failed to send terminated event: {e}");
                        }
                    }
                    Err(e) => {
                        dbg_log!("[DAP-CLI] Execution error: {e:?}");

                        // Send output event with error
                        let mut w = writer.lock().map_err(|e| {
                            io::Error::other(format!(
                                "Writer mutex poisoned sending error output: {e}"
                            ))
                        })?;
                        let output_event = ProtocolMessage::Event(Event {
                            seq: 0,
                            event: "output".to_string(),
                            body: Some(serde_json::json!({
                                "category": "stderr",
                                "output": format!("Execution error: {e:?}\n")
                            })),
                        });
                        drop(send_dap_message(&output_event, &mut *w));

                        // Send terminated event
                        let terminated_event = ProtocolMessage::Event(Event {
                            seq: 0,
                            event: "terminated".to_string(),
                            body: None,
                        });
                        drop(send_dap_message(&terminated_event, &mut *w));
                    }
                }
            }
            Err(e) => {
                dbg_log!("[DAP-CLI] Failed to read file {path}: {e}");

                // Send output event with file read error
                let mut w = writer.lock().map_err(|e| {
                    io::Error::other(format!("Writer mutex poisoned sending file error: {e}"))
                })?;
                let output_event = ProtocolMessage::Event(Event {
                    seq: 0,
                    event: "output".to_string(),
                    body: Some(serde_json::json!({
                        "category": "stderr",
                        "output": format!("Failed to read file {path}: {e}\n")
                    })),
                });
                drop(send_dap_message(&output_event, &mut *w));

                // Send terminated event
                let terminated_event = ProtocolMessage::Event(Event {
                    seq: 0,
                    event: "terminated".to_string(),
                    body: None,
                });
                drop(send_dap_message(&terminated_event, &mut *w));
            }
        }
    } else {
        dbg_log!("[DAP-CLI] Configuration done (no program to execute)");
    }

    Ok(responses)
}
