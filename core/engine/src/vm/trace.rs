use std::time::Duration;

use super::{Vm, operands::OperandsShape};

use crate::JsValue;

/// A stack group represents a group of stack values based on the call frame.
struct StackGroup {
    value: String,
    count: usize,
    frame_pointer: Option<usize>,
}

impl StackGroup {
    const fn new(value: String, count: usize, fp: Option<usize>) -> Self {
        Self {
            value,
            count,
            frame_pointer: fp,
        }
    }
}

/// Information about the current call frame.
#[derive(Debug, Clone, Copy)]
pub struct CallFrameInfo {
    /// The amount of call frames on the frame stack
    pub frame_count: usize,
    /// The current frame pointer
    pub frame_pointer: usize,
}

/// Display options for the current stack trace
#[derive(Debug, Clone, Copy)]
pub struct VmDisplayOptions {
    max_stack_width: usize,
    max_value_len: usize,
}

/// A trace of the current stack at a specific moment
#[derive(Debug, Clone)]
pub struct VmStackTrace {
    /// A clone of the full stack
    pub stack: Vec<JsValue>,
    /// Call frame information
    pub call_frame_info: CallFrameInfo,
    /// Display options for the stack trace
    pub display_options: VmDisplayOptions,
}

impl VmStackTrace {
    const DEFAULT_MAX_VALUE_LEN: usize = 18;
    const DEFAULT_MAX_STACK_WIDTH: usize = 68;

    /// Creates a new stack trace from the current Vm.
    pub(crate) fn new(vm: &Vm) -> Self {
        let display_options = VmDisplayOptions {
            max_stack_width: Self::DEFAULT_MAX_STACK_WIDTH,
            max_value_len: Self::DEFAULT_MAX_VALUE_LEN,
        };

        let call_frame_info = CallFrameInfo {
            frame_count: vm.frames.len(),
            frame_pointer: vm.frame().fp as usize,
        };

        Self {
            stack: vm.stack.stack.clone(),
            display_options,
            call_frame_info,
        }
    }

    fn group(&self) -> (Vec<StackGroup>, bool) {
        let mut force_truncation = false;
        let mut stack_groups: Vec<StackGroup> = Vec::default();
        // Lazily group values to avoid eagerly evaluating `raw_value` for the entire stack.
        for (idx, v) in self.stack.iter().enumerate().rev() {
            let is_frame = self.call_frame_info.frame_pointer == idx;
            let raw = raw_value(v);
            if !is_frame
                && let Some(last_group) = stack_groups.last_mut()
                && last_group.value == raw
                && last_group.frame_pointer.is_none()
            {
                last_group.count += 1;
            } else {
                let marker = if is_frame {
                    Some(self.call_frame_info.frame_count)
                } else {
                    None
                };
                stack_groups.push(StackGroup::new(raw, 1, marker));
                // If groups is large enough to mathematically guarantee overflowing the display width,
                // we can stop evaluating to save instruction budget / time.
                if stack_groups.len() > Self::DEFAULT_MAX_STACK_WIDTH / 2 {
                    force_truncation = true;
                    break;
                }
            }
        }
        (stack_groups, force_truncation)
    }
}

impl std::fmt::Display for VmStackTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.stack.is_empty() {
            f.write_str("[ <empty> ]")?;
            return Ok(());
        }

        let (groups, mut truncated) = self.group();

        let mut stack_string = String::from("[ ");

        let suffix = format!(".. ({} total) ]", self.call_frame_info.frame_count);

        for (
            i,
            StackGroup {
                value,
                count,
                frame_pointer,
            },
        ) in groups.iter().enumerate()
        {
            let displayable_value = truncate_to_len(value, self.display_options.max_value_len);
            let part = if *count > 1 {
                format!("{displayable_value} (x{count})")
            } else {
                displayable_value
            };
            let separator = if let Some(fc) = frame_pointer {
                format!(" |{fc}|")
            } else if i + 1 < groups.len() {
                ",".to_string()
            } else {
                String::new()
            };
            let addition = format!("{part}{separator} ");
            if stack_string.len() + addition.len() + suffix.len()
                > self.display_options.max_stack_width
            {
                truncated = true;
                break;
            }
            stack_string.push_str(&addition);
        }

        if truncated {
            stack_string.push_str(&suffix);
        } else {
            stack_string.push(']');
        }

        f.write_str(&stack_string)
    }
}

fn raw_value(value: &JsValue) -> String {
    match value {
        v if v.is_callable() => "func".to_string(),
        v if v.is_object() => "obj".to_string(),
        v if v.is_undefined() => "und".to_string(),
        v if v.is_null() => "null".to_string(),
        v => v.display().to_string(),
    }
}

fn truncate_to_len(val: &str, max_len: usize) -> String {
    if val.len() <= max_len {
        return val.to_string();
    }
    let mut end = max_len - 2;
    while !val.is_char_boundary(end) && end > 0 {
        end -= 1;
    }
    format!("{}..", &val[..end])
}
/// The call frame name
///
/// This will have the name of the call frame provided or `Global` it's
/// the global call frame.
#[derive(Debug, Clone)]
pub enum CallFrameName {
    /// The global call frame
    Global,
    /// The name of the current call frame.
    Name(String),
}

/// A message that is emitted at the beginning of execution
#[derive(Debug, Clone)]
pub struct ExecutionStartMessage {
    /// The call frame name for the current execution start
    pub call_frame_name: CallFrameName,
}

/// A message that emits details about a call frame
#[derive(Debug, Clone)]
pub struct CallFrameMessage {
    /// The displayable bytecode for the current call frame.
    pub bytecode: String,
}

/// A message that emits instruction execution details about a call frame
#[derive(Debug, Clone)]
pub struct OpcodeExecutionMessage {
    /// The current opcode being executed
    pub opcode: &'static str,
    /// The duration taken for the current opcode execution
    pub duration: Duration,
    /// The operands for the opcode
    pub operands: OperandsShape,
    /// A stack trace for the current execution
    pub stack_trace: VmStackTrace,
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
                    stack_trace,
                } = execution_message;

                println!(
                    "{:<TIME_COLUMN_WIDTH$} {opcode:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {stack_trace}",
                    format!("{}μs", duration.as_micros()),
                    TIME_COLUMN_WIDTH = Self::TIME_COLUMN_WIDTH,
                    OPCODE_COLUMN_WIDTH = Self::OPCODE_COLUMN_WIDTH,
                    OPERAND_COLUMN_WIDTH = Self::OPERAND_COLUMN_WIDTH,
                );
            }
        }
    }
}
