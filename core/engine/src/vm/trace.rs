use std::time::Duration;

/// The call frame name
///
/// This will have the name of the call frame provided or `Global` it's
/// the global call frame.
#[derive(Debug, Clone)]
pub enum CallFrameName {
    Global,
    Name(String),
}

/// A message that is emitted at the beginning of execution
#[derive(Debug, Clone)]
pub struct ExecutionStartMessage {
    pub call_frame_name: CallFrameName,
}

/// A message that emits details about a call frame
#[derive(Debug, Clone)]
pub struct CallFrameMessage {
    pub bytecode: String,
}

/// A message that emits instruction execution details about a call frame
#[derive(Debug, Clone)]
pub struct OpcodeExecutionMessage {
    pub opcode: &'static str,
    pub duration: Duration,
    pub operands: String,
    pub stack: String,
}

/// The various events that are emitted from Boa's virtual machine.
#[derive(Debug, Clone)]
pub enum VirtualMachineEvent {
    /// This event is the first event triggered.
    ///
    /// It emits information about the call frame.
    CallFrameTrace(CallFrameMessage),
    /// This event is triggered when the execution of a call frame is starting.
    ExecutionStart(ExecutionStartMessage),
    /// This event is triggered when executing an operation.
    ///
    /// It provides information about the opcode execution
    ExecutionTrace(OpcodeExecutionMessage),
    /// This event is triggered when a opcode that calls is reached.
    ///
    /// It signals that we about about to switch call frames.
    ExecutionCallEvent,
}

/// A trait to define a tracer that plugs into Boa's `Vm`
pub trait VirtualMachineTracer: std::fmt::Debug {
    /// Emits `VirtualMachineEvent`s from the virtual machine during execution
    fn emit_event(&self, _event: VirtualMachineEvent) {}
}

/// A default empty virtual machine tracer that drops events submitted to it.
#[derive(Debug, Clone, Copy)]
pub struct EmptyTracer;

impl VirtualMachineTracer for EmptyTracer {}

/// `StdoutTracer` is a `VirtualMachineTracer` implementation that prints the events
/// to stdout in a specific format.
#[derive(Debug, Clone, Copy)]
pub struct StdoutTracer;

impl StdoutTracer {
    const COLUMN_WIDTH: usize = 26;
    const TIME_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH / 2;
    const OPCODE_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH;
    const OPERAND_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH;
    const NUMBER_OF_COLUMNS: usize = 4;
}

#[allow(clippy::print_stdout)]
impl VirtualMachineTracer for StdoutTracer {
    fn emit_event(&self, event: VirtualMachineEvent) {
        match event {
            VirtualMachineEvent::ExecutionStart(start_message) => {
                let msg = match start_message.call_frame_name {
                    CallFrameName::Global => " VM Start ".to_string(),
                    CallFrameName::Name(name) => {
                        format!(" Call Frame -- {name} ")
                    }
                };

                println!(
                    "{msg:-^width$}",
                    width = Self::COLUMN_WIDTH * Self::NUMBER_OF_COLUMNS - 10
                );
                println!(
                    "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {:<OPERAND_COLUMN_WIDTH$} Stack\n",
                    "Time",
                    "Opcode",
                    "Operands",
                    TIME_COLUMN_WIDTH = Self::TIME_COLUMN_WIDTH,
                    OPCODE_COLUMN_WIDTH = Self::OPCODE_COLUMN_WIDTH,
                    OPERAND_COLUMN_WIDTH = Self::OPERAND_COLUMN_WIDTH,
                );
            }
            VirtualMachineEvent::ExecutionCallEvent => println!(),
            VirtualMachineEvent::CallFrameTrace(call_frame_message) => {
                println!("{}", call_frame_message.bytecode);
            }
            VirtualMachineEvent::ExecutionTrace(execution_message) => {
                let OpcodeExecutionMessage {
                    opcode,
                    duration,
                    operands,
                    stack,
                } = execution_message;

                println!(
                    "{:<TIME_COLUMN_WIDTH$} {opcode:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {stack}",
                    format!("{}μs", duration.as_micros()),
                    TIME_COLUMN_WIDTH = Self::TIME_COLUMN_WIDTH,
                    OPCODE_COLUMN_WIDTH = Self::OPCODE_COLUMN_WIDTH,
                    OPERAND_COLUMN_WIDTH = Self::OPERAND_COLUMN_WIDTH,
                );
            }
        }
    }
}
