//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits, plus an interpreter to execute those instructions

use crate::{
    environment::lexical_environment::VariableScope, exec::InterpreterState, BoaProfiler, Context,
    Result, Value,
};

pub(crate) mod compilation;
pub(crate) mod instructions;

pub use compilation::Compiler;
pub use instructions::Instruction;
use std::time::{Duration, Instant};

/// Virtual Machine.
#[derive(Debug)]
pub struct VM<'a> {
    ctx: &'a mut Context,
    idx: usize,
    instructions: Vec<Instruction>,
    pool: Vec<Value>,
    stack: Vec<Value>,
    stack_pointer: usize,
    profile: Profiler,
    is_trace: bool,
}
/// This profiler is used to output trace information when `--trace` is provided by the CLI or trace is set to `true` on the [`VM`] object
#[derive(Debug)]
struct Profiler {
    instant: Instant,
    prev_time: Duration,
    trace_string: String,
    start_flag: bool,
}

#[cfg(test)]
mod tests;

impl<'a> VM<'a> {
    pub fn new(compiler: Compiler, ctx: &'a mut Context) -> Self {
        let trace = ctx.trace;
        Self {
            ctx,
            idx: 0,
            instructions: compiler.instructions,
            pool: compiler.pool,
            stack: vec![],
            stack_pointer: 0,
            is_trace: trace,
            profile: Profiler {
                instant: Instant::now(),
                prev_time: Duration::from_secs(0),
                trace_string: String::new(), // Won't allocate if we don't use trace
                start_flag: false,
            },
        }
    }

    /// Push a value on the stack.
    #[inline]
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pop a value off the stack.
    ///
    /// # Panics
    ///
    /// If there is nothing to pop, then this will panic.
    #[inline]
    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    pub fn run(&mut self) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("runVM", "vm");
        self.idx = 0;

        while self.idx < self.instructions.len() {
            if self.is_trace {
                self.trace_print(false);
            };

            let _timer =
                BoaProfiler::global().start_event(&self.instructions[self.idx].to_string(), "vm");

            macro_rules! bin_op {
                ($op:ident) => {{
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.$op(&r, self.ctx)?;
                    Some(val.into())
                }};
            }
            let result = match self.instructions[self.idx] {
                Instruction::Undefined => Some(Value::undefined()),
                Instruction::Null => Some(Value::null()),
                Instruction::True => Some(Value::boolean(true)),
                Instruction::False => Some(Value::boolean(false)),
                Instruction::Zero => Some(Value::integer(0)),
                Instruction::One => Some(Value::integer(1)),
                Instruction::Int32(i) => Some(Value::integer(i)),
                Instruction::Rational(r) => Some(Value::rational(r)),
                Instruction::String(index) => Some(self.pool[index].clone()),
                Instruction::BigInt(index) => Some(self.pool[index].clone()),
                Instruction::Add => {
                    bin_op!(add)
                }
                Instruction::Sub => {
                    bin_op!(sub)
                }
                Instruction::Mul => {
                    bin_op!(mul)
                }
                Instruction::Div => {
                    bin_op!(div)
                }
                Instruction::Pow => {
                    bin_op!(pow)
                }
                Instruction::Mod => {
                    bin_op!(rem)
                }
                Instruction::BitAnd => {
                    bin_op!(bitand)
                }
                Instruction::BitOr => {
                    bin_op!(bitor)
                }
                Instruction::BitXor => {
                    bin_op!(bitxor)
                }
                Instruction::Shl => {
                    bin_op!(shl)
                }
                Instruction::Shr => {
                    bin_op!(shr)
                }
                Instruction::UShr => {
                    bin_op!(ushr)
                }
                Instruction::Eq => {
                    let r = self.pop();
                    let l = self.pop();
                    Some((l.equals(&r, self.ctx)?).into())
                }
                Instruction::NotEq => {
                    let r = self.pop();
                    let l = self.pop();
                    Some((!l.equals(&r, self.ctx)?).into())
                }
                Instruction::StrictEq => {
                    let r = self.pop();
                    let l = self.pop();
                    Some((l.strict_equals(&r)).into())
                }
                Instruction::StrictNotEq => {
                    let r = self.pop();
                    let l = self.pop();
                    Some((!l.strict_equals(&r)).into())
                }
                Instruction::Gt => {
                    bin_op!(gt)
                }
                Instruction::Ge => {
                    bin_op!(ge)
                }
                Instruction::Lt => {
                    bin_op!(lt)
                }
                Instruction::Le => {
                    bin_op!(le)
                }
                Instruction::In => {
                    let r = self.pop();
                    let l = self.pop();

                    if !r.is_object() {
                        return self.ctx.throw_type_error(format!(
                            "right-hand side of 'in' should be an object, got {}",
                            r.get_type().as_str()
                        ));
                    }
                    let key = l.to_property_key(self.ctx)?;
                    Some(self.ctx.has_property(&r, &key).into())
                }
                Instruction::InstanceOf => {
                    let r = self.pop();
                    let _l = self.pop();
                    if !r.is_object() {
                        return self.ctx.throw_type_error(format!(
                            "right-hand side of 'instanceof' should be an object, got {}",
                            r.get_type().as_str()
                        ));
                    }

                    // spec: https://tc39.es/ecma262/#sec-instanceofoperator
                    todo!("instanceof operator")
                }
                Instruction::Void => {
                    let _value = self.pop();
                    Some(Value::undefined())
                }
                Instruction::TypeOf => {
                    let value = self.pop();
                    Some(value.get_type().as_str().into())
                }
                Instruction::Pos => {
                    let value = self.pop();
                    let value = value.to_number(self.ctx)?;
                    Some(value.into())
                }
                Instruction::Neg => {
                    let value = self.pop();
                    Some(Value::from(!value.to_boolean()))
                }
                Instruction::Not => {
                    let value = self.pop();
                    Some((!value.to_boolean()).into())
                }
                Instruction::BitNot => {
                    let target = self.pop();
                    let num = target.to_number(self.ctx)?;
                    let value = if num.is_nan() {
                        -1
                    } else {
                        // TODO: this is not spec compliant.
                        !(num as i32)
                    };
                    Some(value.into())
                }
                Instruction::DefVar(name_index) => {
                    let name: String = self.pool[name_index].to_string(self.ctx)?.to_string();

                    self.ctx.create_mutable_binding(
                        name.to_string(),
                        false,
                        VariableScope::Function,
                    )?;

                    None
                }
                Instruction::DefLet(name_index) => {
                    let name = self.pool[name_index].to_string(self.ctx)?;

                    self.ctx.create_mutable_binding(
                        name.to_string(),
                        false,
                        VariableScope::Block,
                    )?;

                    None
                }
                Instruction::DefConst(name_index) => {
                    let name = self.pool[name_index].to_string(self.ctx)?;

                    self.ctx.create_immutable_binding(
                        name.to_string(),
                        false,
                        VariableScope::Block,
                    )?;

                    None
                }
                Instruction::InitLexical(name_index) => {
                    let name = self.pool[name_index].to_string(self.ctx)?;
                    let value = self.pop();
                    self.ctx.initialize_binding(&name, value.clone())?;

                    None
                }
                // Find a binding on the environment chain and push its value.
                Instruction::GetName(ref name) => match self.ctx.get_binding_value(&name) {
                    Ok(val) => Some(val),
                    Err(val) => {
                        self.ctx
                            .executor()
                            .set_current_state(InterpreterState::Error);
                        Some(val)
                    }
                },
                // Create a new object and push to the stack
                Instruction::NewObject => Some(Value::new_object(self.ctx)),
            };

            if let Some(value) = result {
                self.push(value);
            }

            self.idx += 1;

            if matches!(
                self.ctx.executor().get_current_state(),
                &InterpreterState::Error,
            ) {
                break;
            }
        }

        if self.is_trace {
            self.trace_print(true);
        };
        let res = self.pop();
        Ok(res)
    }

    pub fn trace_print(&mut self, end: bool) {
        let duration = self.profile.instant.elapsed() - self.profile.prev_time;
        if self.profile.start_flag {
            if self.is_trace {
                println!(
                    "{0: <10} {1}",
                    format!("{}μs", duration.as_micros()),
                    self.profile.trace_string
                );
            }
        } else {
            println!("VM start up time: {}μs", duration.as_micros());
            println!(
                "{0: <10} {1: <20} {2: <10}",
                "Time", "Instr", "Top Of Stack"
            );
            println!();
        }

        self.profile.start_flag = true;

        if self.is_trace {
            self.profile.trace_string = format!(
                "{0:<20}  {1}",
                format!(
                    "{:<20}",
                    self.instructions[if end { self.idx - 1 } else { self.idx }]
                ),
                match self.stack.last() {
                    None => "<empty>".to_string(),
                    Some(val) => format!("{}\t{:p}", val.display(), val),
                }
            );
        }

        if end {
            println!();
            println!("Pool");
            for (i, val) in self.pool.iter().enumerate() {
                println!("{:<10} {:<10} {:p}", i, val.display(), val);
            }

            println!();
            println!("Stack");
            for (i, val) in self.stack.iter().enumerate() {
                println!("{:<10} {:<10} {:p}", i, val.display(), val);
            }
            println!();
        }

        self.profile.prev_time = self.profile.instant.elapsed();
    }
}
