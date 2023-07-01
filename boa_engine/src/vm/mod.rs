//! Boa's ECMAScript Virtual Machine
//!
//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        function::{arguments::Arguments, FunctionKind, ThisMode},
    },
    environments::{DeclarativeEnvironment, EnvironmentStack, FunctionSlots, ThisBindingStatus},
    realm::Realm,
    script::Script,
    vm::code_block::Readable,
    Context, JsError, JsNativeError, JsObject, JsResult, JsValue, Module,
};

use boa_gc::{custom_trace, Finalize, Gc, Trace};
use boa_profiler::Profiler;
use std::mem::size_of;

#[cfg(feature = "trace")]
use boa_interner::ToInternedString;
#[cfg(feature = "trace")]
use std::time::Instant;

mod call_frame;
mod code_block;
mod completion_record;
mod opcode;

mod runtime_limits;

#[cfg(feature = "flowgraph")]
pub mod flowgraph;

pub use runtime_limits::RuntimeLimits;

use self::opcode::ExecutionResult;
pub use {call_frame::CallFrame, code_block::CodeBlock, opcode::Opcode};

pub(crate) use {
    call_frame::GeneratorResumeKind,
    code_block::{
        create_function_object, create_function_object_fast, create_generator_function_object,
        CodeBlockFlags,
    },
    completion_record::CompletionRecord,
    opcode::BindingOpcode,
};

#[cfg(test)]
mod tests;

struct CallerState {
    realm: Realm,
    active_function: Option<JsObject>,
    environments: EnvironmentStack,
    stack: Vec<JsValue>,
    active_runnable: Option<ActiveRunnable>,
}

/// Virtual Machine.
#[derive(Debug)]
pub struct Vm {
    pub(crate) frames: Vec<CallFrame>,
    pub(crate) stack: Vec<JsValue>,
    pub(crate) err: Option<JsError>,
    pub(crate) environments: EnvironmentStack,
    #[cfg(feature = "trace")]
    pub(crate) trace: bool,
    pub(crate) runtime_limits: RuntimeLimits,
    pub(crate) active_function: Option<JsObject>,
    pub(crate) active_runnable: Option<ActiveRunnable>,
}

/// Active runnable in the current vm context.
#[derive(Debug, Clone, Finalize)]
pub(crate) enum ActiveRunnable {
    Script(Script),
    Module(Module),
}

unsafe impl Trace for ActiveRunnable {
    custom_trace!(this, {
        match this {
            Self::Script(script) => mark(script),
            Self::Module(module) => mark(module),
        }
    });
}

impl Vm {
    /// Creates a new virtual machine.
    pub(crate) fn new(global: Gc<DeclarativeEnvironment>) -> Self {
        Self {
            frames: Vec::with_capacity(16),
            stack: Vec::with_capacity(1024),
            environments: EnvironmentStack::new(global),
            err: None,
            #[cfg(feature = "trace")]
            trace: false,
            runtime_limits: RuntimeLimits::default(),
            active_function: None,
            active_runnable: None,
        }
    }

    /// Push a value on the stack.
    pub(crate) fn push<T>(&mut self, value: T)
    where
        T: Into<JsValue>,
    {
        self.stack.push(value.into());
    }

    /// Pop a value off the stack.
    ///
    /// # Panics
    ///
    /// If there is nothing to pop, then this will panic.
    #[track_caller]
    pub(crate) fn pop(&mut self) -> JsValue {
        self.stack.pop().expect("stack was empty")
    }

    #[track_caller]
    pub(crate) fn read<T: Readable>(&mut self) -> T {
        let value = self.frame().code_block.read::<T>(self.frame().pc as usize);
        self.frame_mut().pc += size_of::<T>() as u32;
        value
    }

    /// Retrieves the VM frame
    ///
    /// # Panics
    ///
    /// If there is no frame, then this will panic.
    #[track_caller]
    pub(crate) fn frame(&self) -> &CallFrame {
        self.frames.last().expect("no frame found")
    }

    /// Retrieves the VM frame mutably
    ///
    /// # Panics
    ///
    /// If there is no frame, then this will panic.
    #[track_caller]
    pub(crate) fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("no frame found")
    }

    // TODO: Rename `frame` to `frame_expect` and make this `frame`.
    pub(crate) fn frame_opt(&self) -> Option<&CallFrame> {
        self.frames.last()
    }

    pub(crate) fn push_frame(&mut self, frame: CallFrame) {
        self.frames.push(frame);
    }

    pub(crate) fn pop_frame(&mut self) -> Option<CallFrame> {
        self.frames.pop()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CompletionType {
    Normal,
    Return,
    Throw,
}

impl Context<'_> {
    fn execute_instruction(&mut self) -> JsResult<CompletionType> {
        let opcode: Opcode = {
            let _timer = Profiler::global().start_event("Opcode retrieval", "vm");

            let frame = self.vm.frame_mut();

            let pc = frame.pc;
            let opcode = Opcode::from(frame.code_block.bytecode[pc as usize]);
            frame.pc += 1;
            opcode
        };

        let _timer = Profiler::global().start_event(opcode.as_instruction_str(), "vm");

        opcode.execute(self)
    }

    fn execute_instruction2(&mut self) -> JsResult<ExecutionResult> {
        let opcode: Opcode = {
            let _timer = Profiler::global().start_event("Opcode retrieval", "vm");

            let frame = self.vm.frame_mut();

            let pc = frame.pc;
            let opcode = Opcode::from(frame.code_block.bytecode[pc as usize]);
            frame.pc += 1;
            opcode
        };

        let _timer = Profiler::global().start_event(opcode.as_instruction_str(), "vm");

        opcode.execute2(self)
    }

    pub(crate) fn run(&mut self) -> CompletionRecord {
        #[cfg(feature = "trace")]
        const COLUMN_WIDTH: usize = 26;
        #[cfg(feature = "trace")]
        const TIME_COLUMN_WIDTH: usize = COLUMN_WIDTH / 2;
        #[cfg(feature = "trace")]
        const OPCODE_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        #[cfg(feature = "trace")]
        const OPERAND_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        #[cfg(feature = "trace")]
        const NUMBER_OF_COLUMNS: usize = 4;

        let _timer = Profiler::global().start_event("run", "vm");

        #[cfg(feature = "trace")]
        if self.vm.trace {
            let msg = if self.vm.frames.last().is_some() {
                " Call Frame "
            } else {
                " VM Start "
            };

            println!(
                "{}\n",
                self.vm
                    .frame()
                    .code_block
                    .to_interned_string(self.interner())
            );
            println!(
                "{msg:-^width$}",
                width = COLUMN_WIDTH * NUMBER_OF_COLUMNS - 10
            );
            println!(
                "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {:<OPERAND_COLUMN_WIDTH$} Top Of Stack\n",
                "Time",
                "Opcode",
                "Operands",
            );
        }

        let current_stack_length = self.vm.stack.len();
        self.vm
            .frame_mut()
            .set_frame_pointer(current_stack_length as u32);

        // If the current executing function is an async function we have to resolve/reject it's promise at the end.
        // The relevant spec section is 3. in [AsyncBlockStart](https://tc39.es/ecma262/#sec-asyncblockstart).
        let promise_capability = self.vm.frame().promise_capability.clone();

        let execution_completion = loop {
            // 1. Exit the execution loop if program counter ever is equal to or exceeds the amount of instructions
            if self.vm.frame().code_block.bytecode.len() <= self.vm.frame().pc as usize {
                break CompletionType::Normal;
            }

            #[cfg(feature = "fuzz")]
            {
                if self.instructions_remaining == 0 {
                    let err = JsError::from_native(JsNativeError::no_instructions_remain());
                    self.vm.err = Some(err);
                    break CompletionType::Throw;
                }
                self.instructions_remaining -= 1;
            }

            // 1. Run the next instruction.
            #[cfg(feature = "trace")]
            let result = if self.vm.trace || self.vm.frame().code_block.traceable() {
                let mut pc = self.vm.frame().pc as usize;
                let opcode: Opcode = self
                    .vm
                    .frame()
                    .code_block
                    .read::<u8>(pc)
                    .try_into()
                    .expect("invalid opcode");
                let operands = self
                    .vm
                    .frame()
                    .code_block
                    .instruction_operands(&mut pc, self.interner());

                let instant = Instant::now();
                let result = self.execute_instruction();

                let duration = instant.elapsed();
                println!(
                    "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {}",
                    format!("{}μs", duration.as_micros()),
                    opcode.as_str(),
                    match self.vm.stack.last() {
                        Some(value) if value.is_callable() => "[function]".to_string(),
                        Some(value) if value.is_object() => "[object]".to_string(),
                        Some(value) => value.display().to_string(),
                        None => "<empty>".to_string(),
                    },
                );

                result
            } else {
                self.execute_instruction()
            };

            #[cfg(not(feature = "trace"))]
            let result = self.execute_instruction();

            // 2. Evaluate the result of executing the instruction.
            match result {
                Ok(CompletionType::Normal) => {}
                Ok(CompletionType::Return) => {
                    break CompletionType::Return;
                }
                Ok(CompletionType::Throw) => {
                    break CompletionType::Throw;
                }
                Err(err) => {
                    #[cfg(feature = "fuzz")]
                    {
                        if let Some(native_error) = err.as_native() {
                            // If we hit the execution step limit, bubble up the error to the
                            // (Rust) caller instead of trying to handle as an exception.
                            if native_error.is_no_instructions_remain() {
                                self.vm.err = Some(err);
                                break CompletionType::Throw;
                            }
                        }
                    }

                    if let Some(native_error) = err.as_native() {
                        // If we hit the execution step limit, bubble up the error to the
                        // (Rust) caller instead of trying to handle as an exception.
                        if native_error.is_runtime_limit() {
                            self.vm.err = Some(err);
                            break CompletionType::Throw;
                        }
                    }

                    self.vm.err = Some(err);

                    // If this frame has not evaluated the throw as an AbruptCompletion, then evaluate it
                    let evaluation = Opcode::Throw
                        .execute(self)
                        .expect("Opcode::Throw cannot return Err");

                    if evaluation == CompletionType::Normal {
                        continue;
                    }

                    break CompletionType::Throw;
                }
            }
        };

        // Early return immediately after loop.
        if self.vm.frame().r#yield {
            self.vm.frame_mut().r#yield = false;
            let result = self.vm.pop();
            return CompletionRecord::Return(result);
        }

        #[cfg(feature = "trace")]
        if self.vm.trace {
            println!("\nStack:");
            if self.vm.stack.is_empty() {
                println!("    <empty>");
            } else {
                for (i, value) in self.vm.stack.iter().enumerate() {
                    println!(
                        "{i:04}{:<width$} {}",
                        "",
                        if value.is_callable() {
                            "[function]".to_string()
                        } else if value.is_object() {
                            "[object]".to_string()
                        } else {
                            value.display().to_string()
                        },
                        width = COLUMN_WIDTH / 2 - 4,
                    );
                }
            }
            println!("\n");
        }

        // Determine the execution result
        let execution_result = if execution_completion == CompletionType::Throw {
            self.vm.frame_mut().abrupt_completion = None;
            self.vm.stack.truncate(self.vm.frame().fp as usize);
            JsValue::undefined()
        } else if execution_completion == CompletionType::Return {
            self.vm.frame_mut().abrupt_completion = None;
            let result = self.vm.pop();
            self.vm.stack.truncate(self.vm.frame().fp as usize);
            result
        } else if self.vm.stack.len() <= self.vm.frame().fp as usize {
            JsValue::undefined()
        } else {
            let result = self.vm.pop();
            self.vm.stack.truncate(self.vm.frame().fp as usize);
            result
        };

        if let Some(promise) = promise_capability {
            match execution_completion {
                CompletionType::Normal => {
                    promise
                        .resolve()
                        .call(&JsValue::undefined(), &[], self)
                        .expect("cannot fail per spec");
                }
                CompletionType::Return => {
                    promise
                        .resolve()
                        .call(&JsValue::undefined(), &[execution_result.clone()], self)
                        .expect("cannot fail per spec");
                }
                CompletionType::Throw => {
                    let err = self.vm.err.take().expect("Take must exist on a Throw");
                    promise
                        .reject()
                        .call(&JsValue::undefined(), &[err.to_opaque(self)], self)
                        .expect("cannot fail per spec");
                    self.vm.err = Some(err);
                }
            }
        } else if let Some(generator_object) = self.vm.frame().async_generator.clone() {
            // Step 3.e-g in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart)
            let mut generator_object_mut = generator_object.borrow_mut();
            let generator = generator_object_mut
                .as_async_generator_mut()
                .expect("must be async generator");

            generator.state = AsyncGeneratorState::Completed;
            generator.context = None;

            let next = generator
                .queue
                .pop_front()
                .expect("must have item in queue");
            drop(generator_object_mut);

            if execution_completion == CompletionType::Throw {
                AsyncGenerator::complete_step(
                    &next,
                    Err(self
                        .vm
                        .err
                        .take()
                        .expect("err must exist on a Completion::Throw")),
                    true,
                    None,
                    self,
                );
            } else {
                AsyncGenerator::complete_step(&next, Ok(execution_result), true, None, self);
            }
            AsyncGenerator::drain_queue(&generator_object, self);

            return CompletionRecord::Normal(JsValue::undefined());
        }

        // Any valid return statement is re-evaluated as a normal completion vs. return (yield).
        if execution_completion == CompletionType::Throw {
            return CompletionRecord::Throw(
                self.vm
                    .err
                    .take()
                    .expect("Err must exist for a CompletionType::Throw"),
            );
        }
        CompletionRecord::Normal(execution_result)
    }

    pub(crate) fn run2(&mut self) -> JsResult<JsValue> {
        #[cfg(feature = "trace")]
        const COLUMN_WIDTH: usize = 26;
        #[cfg(feature = "trace")]
        const TIME_COLUMN_WIDTH: usize = COLUMN_WIDTH / 2;
        #[cfg(feature = "trace")]
        const OPCODE_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        #[cfg(feature = "trace")]
        const OPERAND_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        #[cfg(feature = "trace")]
        const NUMBER_OF_COLUMNS: usize = 4;

        let _timer = Profiler::global().start_event("run2", "vm");

        #[cfg(feature = "trace")]
        if self.vm.trace {
            let msg = if self.vm.frames.last().is_some() {
                " Call Frame "
            } else {
                " VM Start "
            };

            println!(
                "{}\n",
                self.vm
                    .frame()
                    .code_block
                    .to_interned_string(self.interner())
            );
            println!(
                "{msg:-^width$}",
                width = COLUMN_WIDTH * NUMBER_OF_COLUMNS - 10
            );
            println!(
                "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {:<OPERAND_COLUMN_WIDTH$} Top Of Stack\n",
                "Time",
                "Opcode",
                "Operands",
            );
        }

        assert_eq!(
            self.vm.stack.len(),
            0,
            "run2 can only run top-level scripts."
        );

        let mut caller_stack: Vec<CallerState> = Vec::new();

        let result = loop {
            // Exit the execution loop if there aren't any more callers.
            let Some(frame) = self.vm.frame_opt() else {
                break self
                        .vm
                        .err
                        .take()
                        .map_or_else(|| Ok(self.vm.pop()), Err);
            };

            if frame.code_block.bytecode.len() <= frame.pc as usize {
                let push_undef = self.vm.stack.len() <= self.vm.frame().fp as usize;

                // TODO: cleanup this hack.
                if let Some(last) = caller_stack.pop() {
                    self.restore_caller(last);
                }

                if push_undef {
                    self.vm.push(JsValue::undefined());
                }

                self.vm.pop_frame();
                continue;
            }

            let result = if let Some(err) = self.vm.err.take() {
                Err(err)
            } else {
                #[cfg(feature = "trace")]
                if self.vm.trace || self.vm.frame().code_block.traceable() {
                    let mut pc = self.vm.frame().pc as usize;
                    let opcode: Opcode = self
                        .vm
                        .frame()
                        .code_block
                        .read::<u8>(pc)
                        .try_into()
                        .expect("invalid opcode");
                    let operands = self
                        .vm
                        .frame()
                        .code_block
                        .instruction_operands(&mut pc, self.interner());

                    let instant = Instant::now();
                    let result = self.execute_instruction2();

                    let duration = instant.elapsed();
                    println!(
                        "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {}",
                        format!("{}μs", duration.as_micros()),
                        opcode.as_str(),
                        match self.vm.stack.last() {
                            Some(value) if value.is_callable() => "[function]".to_string(),
                            Some(value) if value.is_object() => "[object]".to_string(),
                            Some(value) => value.display().to_string(),
                            None => "<empty>".to_string(),
                        },
                    );

                    result
                } else {
                    self.execute_instruction2()
                }

                #[cfg(not(feature = "trace"))]
                self.execute_instruction2()
            };

            let completion = match result {
                Ok(ExecutionResult::Call { f, this, args }) => {
                    prepare_call(f, this, args, &mut caller_stack, self);
                    continue;
                }
                Ok(ExecutionResult::Completion(completion)) => completion,
                Err(err) => {
                    self.vm.err = Some(err);

                    // If this frame has not evaluated the throw as an AbruptCompletion, then evaluate it
                    let evaluation = Opcode::Throw
                        .execute(self)
                        .expect("Opcode::Throw cannot return Err");

                    if evaluation == CompletionType::Normal {
                        continue;
                    }

                    CompletionType::Throw
                }
            };

            if completion == CompletionType::Throw {
                // TODO: cleanup this hack.
                if !caller_stack.is_empty() {
                    self.restore_caller(
                        caller_stack
                            .pop()
                            .expect("already checked that the stack is not empty"),
                    );
                }
                self.vm.pop_frame();
            } else if completion == CompletionType::Return {
                let result = self.vm.pop();
                // TODO: cleanup this hack.
                if let Some(last) = caller_stack.pop() {
                    self.restore_caller(last);
                }
                self.vm.push(result);
                self.vm.pop_frame();
            }
        };

        result
    }

    fn restore_caller(&mut self, state: CallerState) {
        self.vm.environments = state.environments;
        self.vm.stack = state.stack;
        self.vm.active_function = state.active_function;
        self.vm.active_runnable = state.active_runnable;
        self.enter_realm(state.realm);
    }
}

fn prepare_call(
    f: JsObject,
    this: JsValue,
    args: Vec<JsValue>,
    caller_stack: &mut Vec<CallerState>,
    context: &mut Context<'_>,
) {
    let object = f.borrow();
    let function_object = object.as_function().expect("not a function");
    let realm = function_object.realm().clone();

    let old_realm = context.enter_realm(realm);

    let old_active_function = context.vm.active_function.replace(f.clone());

    let (code, mut environments, class_object, mut script_or_module) = match function_object.kind()
    {
        FunctionKind::Ordinary {
            code,
            environments,
            class_object,
            script_or_module,
            ..
        } => {
            let code = code.clone();
            if code.is_class_constructor() {
                context.vm.err = Some(
                    JsNativeError::typ()
                        .with_message("class constructor cannot be invoked without 'new'")
                        .with_realm(context.realm().clone())
                        .into(),
                );
                return;
            }
            (
                code,
                environments.clone(),
                class_object.clone(),
                script_or_module.clone(),
            )
        }
        _ => {
            drop(object);
            match f.call_internal(&this, &args, context) {
                Ok(val) => context.vm.push(val),
                Err(err) => {
                    context.vm.err = Some(err);
                }
            }
            context.enter_realm(old_realm);
            context.vm.active_function = old_active_function;
            return;
        }
    };

    drop(object);

    std::mem::swap(&mut environments, &mut context.vm.environments);

    let lexical_this_mode = code.this_mode == ThisMode::Lexical;

    let this = if lexical_this_mode {
        ThisBindingStatus::Lexical
    } else if code.strict() {
        ThisBindingStatus::Initialized(this.clone())
    } else if this.is_null_or_undefined() {
        ThisBindingStatus::Initialized(context.realm().global_this().clone().into())
    } else {
        ThisBindingStatus::Initialized(
            this.to_object(context)
                .expect("conversion cannot fail")
                .into(),
        )
    };

    let mut last_env = code.compile_environments.len() - 1;

    if let Some(class_object) = class_object {
        let index = context
            .vm
            .environments
            .push_lexical(code.compile_environments[last_env].clone());
        context
            .vm
            .environments
            .put_lexical_value(index, 0, class_object.into());
        last_env -= 1;
    }

    if code.has_binding_identifier() {
        let index = context
            .vm
            .environments
            .push_lexical(code.compile_environments[last_env].clone());
        context
            .vm
            .environments
            .put_lexical_value(index, 0, f.clone().into());
        last_env -= 1;
    }

    context.vm.environments.push_function(
        code.compile_environments[last_env].clone(),
        FunctionSlots::new(this, f.clone(), None),
    );

    if code.has_parameters_env_bindings() {
        last_env -= 1;
        context
            .vm
            .environments
            .push_lexical(code.compile_environments[last_env].clone());
    }

    // Taken from: `FunctionDeclarationInstantiation` abstract function.
    //
    // Spec: https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
    //
    // 22. If argumentsObjectNeeded is true, then
    if code.needs_arguments_object() {
        // a. If strict is true or simpleParameterList is false, then
        //     i. Let ao be CreateUnmappedArgumentsObject(argumentsList).
        // b. Else,
        //     i. NOTE: A mapped argument object is only provided for non-strict functions
        //              that don't have a rest parameter, any parameter
        //              default value initializers, or any destructured parameters.
        //     ii. Let ao be CreateMappedArgumentsObject(func, formals, argumentsList, env).
        let arguments_obj = if code.strict() || !code.params.is_simple() {
            Arguments::create_unmapped_arguments_object(&args, context)
        } else {
            let env = context.vm.environments.current();
            Arguments::create_mapped_arguments_object(
                &f,
                &code.params,
                &args,
                env.declarative_expect(),
                context,
            )
        };
        let env_index = context.vm.environments.len() as u32 - 1;
        context
            .vm
            .environments
            .put_lexical_value(env_index, 0, arguments_obj.into());
    }

    let argument_count = args.len();

    // Push function arguments to the stack.
    let mut args = if code.params.as_ref().len() > args.len() {
        let mut v = args.to_vec();
        v.extend(vec![
            JsValue::Undefined;
            code.params.as_ref().len() - args.len()
        ]);
        v
    } else {
        args.to_vec()
    };
    args.reverse();
    let mut stack = args;

    std::mem::swap(&mut context.vm.stack, &mut stack);

    let frame = CallFrame::new(code).with_argument_count(argument_count as u32);

    std::mem::swap(&mut context.vm.active_runnable, &mut script_or_module);

    context.vm.push_frame(frame);

    caller_stack.push(CallerState {
        realm: old_realm,
        active_function: old_active_function,
        environments,
        stack,
        active_runnable: script_or_module,
    });
}
