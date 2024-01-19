//! Boa's `Trace` module for the `Vm`.

use std::collections::VecDeque;
use std::fmt;

use super::{Constant, Vm, CallFrame};

// TODO: Build out further, maybe provide more element visiblity and events/outputs
/// The `Tracer` trait is a customizable trait that can be provided to `Boa`
/// for customizing output.
pub trait Tracer: fmt::Debug {
    /// Whether the current call frame should trace.
    fn should_trace(&self, frame: &CallFrame) -> TraceAction {
        if frame.code_block.name().to_std_string_escaped().as_str() == "<main>" {
            return TraceAction::BlockWithFullBytecode
        }
        TraceAction::Block
    }
    /// The output from tracing a `CodeBlock`'s bytecode.
    fn emit_bytecode_trace(&self, msg: &str);
    /// The output from entering a `CallFrame`.
    fn emit_call_frame_entrance_trace(&self, msg: &str);
    /// The trace output from an execution.
    fn emit_instruction_trace(&self, msg: &str);
    /// Trace output from exiting a `CallFrame`.
    fn emit_call_frame_exit_trace(&self, msg: &str);
}

/// `TraceAction` Determines the action that should occur
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum TraceAction {
    /// No trace
    None = 0,
    /// Traces the frames code block
    Block,
    /// Partial codeblock with bytecode
    BlockWithBytecode,
    /// Full trace with bytecode
    BlockWithFullBytecode,
}

#[derive(Debug)]
pub(crate) struct ActiveTracer;

impl Tracer for ActiveTracer {
    fn emit_bytecode_trace(&self, msg: &str) {
        println!("{msg}");
    }

    fn emit_call_frame_entrance_trace(&self, msg: &str) {
        println!("{msg}");
    }

    fn emit_instruction_trace(&self, msg: &str) {
        println!("{msg}");
    }

    fn emit_call_frame_exit_trace(&self, msg: &str) {
        println!("{msg}");
    }
}

impl Default for VmTrace {
    fn default() -> Self {
        Self {
            tracers: Vec::default(),
        }
    }
}

/// `VmTrace` is a boa spcific structure for running Boa's Virtual Machine trace.
///
/// It holds registered `Tracer` implementations and actions messages depending on
/// those implementations.
///
/// About the actions
///
/// After the Global callframe is initially provided. It searches
/// for all possible compiled output
pub struct VmTrace {
    tracers: Vec<Box<dyn Tracer>>,
}

// ==== Public API ====

impl VmTrace {
    /// Method for adding a compiled action on initialization.
    pub fn set_tracer(&mut self, tracer: Box<dyn Tracer>) {
        self.tracers.push(tracer);
    }

    /// Returns whether there is an active trace request.
    pub fn should_trace(&self, frame: &CallFrame) -> bool {
        self.trace_action(frame) != TraceAction::None
    }

    pub(crate) fn trace_action(&self, frame: &CallFrame) -> TraceAction {
        (&self.tracers).into_iter().fold(TraceAction::None, |a, b| a.max(b.should_trace(frame)))
    }

    /// Sets the current `Tracer` of `VmTrace`.
    pub fn activate_trace(&mut self) {
        self.tracers.push(Box::new(ActiveTracer));
    }
}

// ==== Trace Event/Action Methods ====

impl VmTrace {
    const COLUMN_WIDTH: usize = 26;
    const TIME_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH / 2;
    const OPCODE_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH;
    const OPERAND_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH;
    const NUMBER_OF_COLUMNS: usize = 4;

    /// Trace the current `CallFrame` according to current state
    pub(crate) fn trace_call_frame(&self, vm: &Vm) {
        let action = self.trace_action(vm.frame());
        match action {
            TraceAction::Block => {
                self.call_frame_header(vm);
            }
            TraceAction::BlockWithFullBytecode => {
                self.trace_compiled_bytecode(vm);
                self.call_frame_header(vm);
            }
            TraceAction::BlockWithBytecode => {
                self.trace_current_bytecode(vm);
                self.call_frame_header(vm);
            }
            TraceAction::None => {}
        }
    }

    /// Emits the current `CallFrame`'s header.
    pub(crate) fn call_frame_header(&self, vm: &Vm) {
        let msg = format!(
            " Call Frame -- {} ",
            vm.frame().code_block().name().to_std_string_escaped()
        );

        let frame_header = format!(
            "{msg:-^width$}",
            width = Self::COLUMN_WIDTH * Self::NUMBER_OF_COLUMNS - 10
        );

        for t in &self.tracers {
            t.emit_call_frame_entrance_trace(&frame_header);
        }

        if vm.frames.len() == 1 {
            let column_headers = format!(
                "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {:<OPERAND_COLUMN_WIDTH$} Stack\n",
                "Time",
                "Opcode",
                "Operands",
                TIME_COLUMN_WIDTH = Self::TIME_COLUMN_WIDTH,
                OPCODE_COLUMN_WIDTH = Self::OPCODE_COLUMN_WIDTH,
                OPERAND_COLUMN_WIDTH = Self::OPERAND_COLUMN_WIDTH,
            );

            for t in &self.tracers {
                t.emit_call_frame_entrance_trace(&column_headers);
            }
        }
    }

    /// Searches traces all of the current `CallFrame`'s available `CodeBlock`s.
    pub(crate) fn trace_compiled_bytecode(&self, vm: &Vm) {
        let mut queue = VecDeque::new();
        queue.push_back(vm.frame().code_block.clone());

        while let Some(active_block) = queue.pop_front() {
            for constant in &active_block.constants {
                if let Constant::Function(block) = constant {
                    queue.push_back(block.clone());
                }
            }

            for t in &self.tracers {
                t.emit_bytecode_trace(&active_block.to_string());
            }
            active_block.set_frame_traced(true);
        }
    }

    /// Searches and traces for only current frame's `CodeBlock`.
    pub(crate) fn trace_current_bytecode(&self, vm: &Vm) {
        for t in &self.tracers {
            t.emit_bytecode_trace(&vm.frame().code_block().to_string());
        }
        vm.frame().code_block().set_frame_traced(true);
    }

    /// Emits an exit message for the current `CallFrame`.
    pub(crate) fn trace_frame_end(&self, vm: &Vm, return_msg: &str) {
        if self.trace_action(vm.frame()) != TraceAction::None {
            let msg = format!(
                " Call Frame -- <Exiting {} via {return_msg}> ",
                vm.frame().code_block().name.to_std_string_escaped()
            );
            let frame_footer = format!(
                "{msg:-^width$}",
                width = Self::COLUMN_WIDTH * Self::NUMBER_OF_COLUMNS - 10
            );

            for t in &self.tracers {
                t.emit_call_frame_exit_trace(&frame_footer);
            }
        }
    }

    pub(crate) fn trace_instruction(
        &self,
        duration: u128,
        operand_kind: &str,
        opcode: &str,
        operands: &str,
        stack: &str,
    ) {
        let instruction_trace = format!(
            "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {stack}",
            format!("{}μs", duration),
            format!("{}{operand_kind}", opcode),
            TIME_COLUMN_WIDTH = Self::TIME_COLUMN_WIDTH,
            OPCODE_COLUMN_WIDTH = Self::OPCODE_COLUMN_WIDTH,
            OPERAND_COLUMN_WIDTH = Self::OPERAND_COLUMN_WIDTH,
        );

        for t in &self.tracers {
            t.emit_instruction_trace(&instruction_trace);
        }
    }
}

impl fmt::Debug for VmTrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.tracers)
    }
}
