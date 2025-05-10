use super::VaryingOperand;
use crate::{
    builtins::Number,
    value::Numeric,
    vm::{opcode::Operation, Registers},
    Context, JsBigInt, JsResult,
};
use std::ops::Neg as StdNeg;

pub(crate) mod decrement;
pub(crate) mod increment;
pub(crate) mod logical;

pub(crate) use decrement::*;
pub(crate) use increment::*;
pub(crate) use logical::*;

/// `TypeOf` implements the Opcode Operation for `Opcode::TypeOf`
///
/// Operation:
///  - Unary `typeof` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TypeOf;

impl TypeOf {
    #[inline(always)]
    pub(super) fn operation(value: VaryingOperand, registers: &mut Registers, _: &mut Context) {
        registers.set(
            value.into(),
            registers.get(value.into()).js_type_of().into(),
        );
    }
}

impl Operation for TypeOf {
    const NAME: &'static str = "TypeOf";
    const INSTRUCTION: &'static str = "INST - TypeOf";
    const COST: u8 = 1;
}

/// `Pos` implements the Opcode Operation for `Opcode::Pos`
///
/// Operation:
///  - Unary `+` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Pos;

impl Pos {
    #[inline(always)]
    pub(super) fn operation(
        value: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        registers.set(
            value.into(),
            registers.get(value.into()).to_number(context)?.into(),
        );
        Ok(())
    }
}

impl Operation for Pos {
    const NAME: &'static str = "Pos";
    const INSTRUCTION: &'static str = "INST - Pos";
    const COST: u8 = 3;
}

/// `Neg` implements the Opcode Operation for `Opcode::Neg`
///
/// Operation:
///  - Unary `-` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Neg;

impl Neg {
    #[inline(always)]
    pub(super) fn operation(
        value: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        match registers.get(value.into()).to_numeric(context)? {
            Numeric::Number(number) => registers.set(value.into(), number.neg().into()),
            Numeric::BigInt(bigint) => registers.set(value.into(), JsBigInt::neg(&bigint).into()),
        }
        Ok(())
    }
}

impl Operation for Neg {
    const NAME: &'static str = "Neg";
    const INSTRUCTION: &'static str = "INST - Neg";
    const COST: u8 = 3;
}

/// `BitNot` implements the Opcode Operation for `Opcode::BitNot`
///
/// Operation:
///  - Unary bitwise `~` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BitNot;

impl BitNot {
    #[inline(always)]
    pub(super) fn operation(
        value: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        match registers.get(value.into()).to_numeric(context)? {
            Numeric::Number(number) => registers.set(value.into(), Number::not(number).into()),
            Numeric::BigInt(bigint) => registers.set(value.into(), JsBigInt::not(&bigint).into()),
        }
        Ok(())
    }
}

impl Operation for BitNot {
    const NAME: &'static str = "BitNot";
    const INSTRUCTION: &'static str = "INST - BitNot";
    const COST: u8 = 3;
}
