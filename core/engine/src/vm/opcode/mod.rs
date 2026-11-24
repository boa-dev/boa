#![allow(clippy::inline_always)]
#![allow(clippy::doc_markdown)]
//! Boa's VM operation implementations and interpreter dispatch.
//!
//! The bytecompiler lowers ECMAScript runtime semantics into these opcode families, which are then
//! executed by the virtual machine. Most opcode submodules mirror a part of the specification,
//! such as function calls, environment records, iteration, property access, or class evaluation.
//! See the general runtime semantics overview in the [specification][spec].
//!
//! When the specification marks an abstract operation as infallible with `!`, Boa treats the
//! corresponding condition as an internal bytecode invariant. In those cases, malformed
//! bytecode is treated as an engine bug and may trip an assertion instead of returning a
//! recoverable JavaScript exception.
//!
//! [spec]: https://tc39.es/ecma262/#sec-runtime-semantics-evaluation
use super::bytecode::{Address, Argument, IndexOperand, RegisterOperand, for_each_opcode};
use crate::{
    Context,
    vm::{completion_record::CompletionRecord, completion_record::IntoCompletionRecord},
};
use std::ops::ControlFlow;
use thin_vec::ThinVec;

// Operation modules
mod arguments;
mod r#await;
mod binary_ops;
mod call;
mod concat;
mod control_flow;
mod copy;
mod define;
mod delete;
mod environment;
mod function;
mod generator;
mod get;
mod iteration;
mod meta;
mod new;
mod nop;
mod object;
mod pop;
mod push;
mod rest_parameter;
mod set;
mod switch;
mod templates;
mod to;
mod unary_ops;
mod value;

// Operation structs
#[doc(inline)]
pub(crate) use arguments::*;
#[doc(inline)]
pub(crate) use r#await::*;
#[doc(inline)]
pub(crate) use binary_ops::*;
#[doc(inline)]
pub(crate) use call::*;
#[doc(inline)]
pub(crate) use concat::*;
#[doc(inline)]
pub(crate) use control_flow::*;
#[doc(inline)]
pub(crate) use copy::*;
#[doc(inline)]
pub(crate) use define::*;
#[doc(inline)]
pub(crate) use delete::*;
#[doc(inline)]
pub(crate) use environment::*;
#[doc(inline)]
pub(crate) use function::*;
#[doc(inline)]
pub(crate) use generator::*;
#[doc(inline)]
pub(crate) use get::*;
#[doc(inline)]
pub(crate) use iteration::*;
#[doc(inline)]
pub(crate) use meta::*;
#[doc(inline)]
pub(crate) use new::*;
#[doc(inline)]
pub(crate) use nop::*;
#[doc(inline)]
pub(crate) use object::*;
#[doc(inline)]
pub(crate) use pop::*;
#[doc(inline)]
pub(crate) use push::*;
#[doc(inline)]
pub(crate) use rest_parameter::*;
#[doc(inline)]
pub(crate) use set::*;
#[doc(inline)]
pub(crate) use switch::*;
#[doc(inline)]
pub(crate) use templates::*;
#[doc(inline)]
pub(crate) use to::*;
#[doc(inline)]
pub(crate) use unary_ops::*;
#[doc(inline)]
pub(crate) use value::*;

/// Metadata used by interpreter operations.
///
/// This trait should be implemented for a struct that corresponds with
/// any arm of the `OpCode` enum.
pub(crate) trait Operation {
    #[allow(unused)] // TODO: need to double check usage.
    const INSTRUCTION: &'static str;
    const COST: u8;
}

macro_rules! generate_handlers {
    (
        $(
            $(#[$comment:ident $($args:tt)*])*
            $Variant:ident $({
                $(
                    $(#[$fieldinner:ident $($fieldargs:tt)*])*
                    $FieldName:ident : $FieldType:ty
                ),*
                $(,)?
            })? $(=> $mapping:ident)?
        ),*
        $(,)?
    ) => {
        type OpcodeHandler = fn(&mut Context, usize) -> ControlFlow<CompletionRecord>;

        pub(crate) const OPCODE_HANDLERS: [OpcodeHandler; 256] = {
            [
                $(
                    pastey::paste! { [<handle_ $Variant:snake>] },
                )*
            ]
        };

        type OpcodeHandlerBudget = fn(&mut Context, usize, &mut u32) -> ControlFlow<CompletionRecord>;

        pub(crate) const OPCODE_HANDLERS_BUDGET: [OpcodeHandlerBudget; 256] = {
            [
                $(
                    pastey::paste! { [<handle_ $Variant:snake _budget>] },
                )*
            ]
        };

        $(
            pastey::paste! {
                #[inline(always)]
                #[allow(unused_parens)]
                fn [<handle_ $Variant:snake>](context: &mut Context, pc: usize) -> ControlFlow<CompletionRecord> {
                    let bytes = &context.vm.frame().code_block.bytecode.bytes;
                    let (args, next_pc) = <($($($FieldType),*)?)>::decode(bytes, pc + 1);
                    context.vm.frame_mut().pc = next_pc as u32;
                    let result = $Variant::operation(args, context);
                    IntoCompletionRecord::into_completion_record(result, context)
                }
            }
        )*

        $(
            pastey::paste! {
                #[inline(always)]
                #[allow(unused_parens)]
                fn [<handle_ $Variant:snake _budget>](context: &mut Context, pc: usize, budget: &mut u32) -> ControlFlow<CompletionRecord> {
                    *budget = budget.saturating_sub(u32::from($Variant::COST));
                    let bytes = &context.vm.frame().code_block.bytecode.bytes;
                    let (args, next_pc) = <($($($FieldType),*)?)>::decode(bytes, pc + 1);
                    context.vm.frame_mut().pc = next_pc as u32;
                    let result = $Variant::operation(args, context);
                    IntoCompletionRecord::into_completion_record(result, context)
                }
            }
        )*

        $(
            $(
                struct $Variant {}
                impl $Variant {
                    #[allow(unused_parens)]
                    #[allow(unused_variables)]
                    #[inline(always)]
                    fn operation(args: (), context: &mut Context) {
                        $mapping::operation(args, context)
                    }
                }

                impl Operation for $Variant {
                    const INSTRUCTION: &'static str = "INST - Reserved";
                    const COST: u8 = 0;
                }
            )?
        )*
    };
}

for_each_opcode!(generate_handlers);
