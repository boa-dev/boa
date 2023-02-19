use crate::{
    builtins::Number,
    value::Numeric,
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context, JsBigInt,
};
use std::ops::Neg as StdNeg;

pub(crate) mod decrement;
pub(crate) mod increment;
pub(crate) mod logical;
pub(crate) mod void;

pub(crate) use decrement::*;
pub(crate) use increment::*;
pub(crate) use logical::*;
pub(crate) use void::*;

/// `TypeOf` implements the Opcode Operation for `Opcode::TypeOf`
///
/// Operation:
///  - Unary `typeof` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TypeOf;

impl Operation for TypeOf {
    const NAME: &'static str = "TypeOf";
    const INSTRUCTION: &'static str = "INST - TypeOf";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        context.vm.push(value.type_of());
        CompletionType::Normal
    }
}

/// `Pos` implements the Opcode Operation for `Opcode::Pos`
///
/// Operation:
///  - Unary `+` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Pos;

impl Operation for Pos {
    const NAME: &'static str = "Pos";
    const INSTRUCTION: &'static str = "INST - Pos";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        let value = ok_or_throw_completion!(value.to_number(context), context);
        context.vm.push(value);
        CompletionType::Normal
    }
}

/// `Neg` implements the Opcode Operation for `Opcode::Neg`
///
/// Operation:
///  - Unary `-` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Neg;

impl Operation for Neg {
    const NAME: &'static str = "Neg";
    const INSTRUCTION: &'static str = "INST - Neg";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        match ok_or_throw_completion!(value.to_numeric(context), context) {
            Numeric::Number(number) => context.vm.push(number.neg()),
            Numeric::BigInt(bigint) => context.vm.push(JsBigInt::neg(&bigint)),
        }
        CompletionType::Normal
    }
}

/// `BitNot` implements the Opcode Operation for `Opcode::BitNot`
///
/// Operation:
///  - Unary bitwise `~` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BitNot;

impl Operation for BitNot {
    const NAME: &'static str = "BitNot";
    const INSTRUCTION: &'static str = "INST - BitNot";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        match ok_or_throw_completion!(value.to_numeric(context), context) {
            Numeric::Number(number) => context.vm.push(Number::not(number)),
            Numeric::BigInt(bigint) => context.vm.push(JsBigInt::not(&bigint)),
        }
        CompletionType::Normal
    }
}
