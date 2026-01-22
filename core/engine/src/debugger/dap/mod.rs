//! Debug Adapter Protocol (DAP) implementation for Boa
//!
//! This module implements the Debug Adapter Protocol specification to enable
//! debugging Boa JavaScript code from IDEs like VS Code.
//!
//! # Architecture
//!
//! The DAP implementation consists of:
//! - Protocol types and messages (requests, responses, events)
//! - A DAP server that communicates via JSON-RPC
//! - Integration with Boa's debugger API
//! - Support for breakpoints, stepping, variable inspection
//!
//! # References
//!
//! - [DAP Specification](https://microsoft.github.io/debug-adapter-protocol/)
//! - [VS Code Debug Extension Guide](https://code.visualstudio.com/api/extension-guides/debugger-extension)

pub mod eval_context;
pub mod messages;
pub mod server;
pub mod session;

pub use eval_context::DebugEvent;
pub use messages::*;
pub use server::DapServer;
pub use session::DebugSession;

use serde::{Deserialize, Serialize};

/// DAP protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProtocolMessage {
    /// A request from the client to the debug adapter
    #[serde(rename = "request")]
    Request(Request),
    /// A response from the debug adapter to the client
    #[serde(rename = "response")]
    Response(Response),
    /// An event sent from the debug adapter to the client
    #[serde(rename = "event")]
    Event(Event),
}

/// DAP request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Sequence number of the message
    pub seq: i64,
    /// The command to execute
    pub command: String,
    /// Optional arguments for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

/// DAP response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Sequence number of the message
    pub seq: i64,
    /// Sequence number of the corresponding request
    pub request_seq: i64,
    /// Whether the request was successful
    pub success: bool,
    /// The command that this response is for
    pub command: String,
    /// Optional error message if success is false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Optional response body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

/// DAP event message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Sequence number of the message
    pub seq: i64,
    /// The type of event
    pub event: String,
    /// Optional event-specific data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

impl ProtocolMessage {
    /// Returns the sequence number of the message
    #[must_use]
    pub fn seq(&self) -> i64 {
        match self {
            Self::Request(r) => r.seq,
            Self::Response(r) => r.seq,
            Self::Event(e) => e.seq,
        }
    }
}
