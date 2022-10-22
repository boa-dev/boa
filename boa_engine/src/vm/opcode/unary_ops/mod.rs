use crate::{
    builtins::Number,
    value::Numeric,
    vm::{opcode::Operation, ShouldExit},
    Context, JsBigInt, JsResult,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TypeOf;

impl Operation for TypeOf {
    const NAME: &'static str = "TypeOf";
    const INSTRUCTION: &'static str = "INST - TypeOf";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        context.vm.push(value.type_of());
        Ok(ShouldExit::False)
    }
}

/// `Pos` implements the Opcode Operation for `Opcode::Pos`
///
/// Operation:
///  - Unary `+` operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Pos;

impl Operation for Pos {
    const NAME: &'static str = "Pos";
    const INSTRUCTION: &'static str = "INST - Pos";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let value = value.to_number(context)?;
        context.vm.push(value);
        Ok(ShouldExit::False)
    }
}

/// `Neg` implements the Opcode Operation for `Opcode::Neg`
///
/// Operation:
///  - Unary `-` operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Neg;

impl Operation for Neg {
    const NAME: &'static str = "Neg";
    const INSTRUCTION: &'static str = "INST - Neg";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        match value.to_numeric(context)? {
            Numeric::Number(number) => context.vm.push(number.neg()),
            Numeric::BigInt(bigint) => context.vm.push(JsBigInt::neg(&bigint)),
        }
        Ok(ShouldExit::False)
    }
}

/// `BitNot` implements the Opcode Operation for `Opcode::BitNot`
///
/// Operation:
///  - Unary bitwise `~` operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BitNot;

impl Operation for BitNot {
    const NAME: &'static str = "BitNot";
    const INSTRUCTION: &'static str = "INST - BitNot";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        match value.to_numeric(context)? {
            Numeric::Number(number) => context.vm.push(Number::not(number)),
            Numeric::BigInt(bigint) => context.vm.push(JsBigInt::not(&bigint)),
        }
        Ok(ShouldExit::False)
    }
}
