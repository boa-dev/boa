use crate::{
    builtins::Number,
    value::Numeric,
    vm::{opcode::Operation, ShouldExit},
    Context, JsBigInt, JsResult, JsValue,
};
use std::ops::Neg as StdNeg;

pub(crate) mod bin_ops;
pub(crate) mod decrement;
pub(crate) mod increment;
pub(crate) mod jump;
pub(crate) mod logical;

pub(crate) use bin_ops::*;
pub(crate) use decrement::*;
pub(crate) use increment::*;
pub(crate) use jump::*;
pub(crate) use logical::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Nop;

impl Operation for Nop {
    const NAME: &'static str = "Nop";
    const INSTRUCTION: &'static str = "INST - Nop";

    fn execute(_context: &mut Context) -> JsResult<ShouldExit> {
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NotEq;

impl Operation for NotEq {
    const NAME: &'static str = "NotEq";
    const INSTRUCTION: &'static str = "INST - NotEq";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let rhs = context.vm.pop();
        let lhs = context.vm.pop();
        let value = !lhs.equals(&rhs, context)?;
        context.vm.push(value);
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StrictEq;

impl Operation for StrictEq {
    const NAME: &'static str = "StrictEq";
    const INSTRUCTION: &'static str = "INST - StrictEq";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let rhs = context.vm.pop();
        let lhs = context.vm.pop();
        context.vm.push(lhs.strict_equals(&rhs));
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StrictNotEq;

impl Operation for StrictNotEq {
    const NAME: &'static str = "StrictNotEq";
    const INSTRUCTION: &'static str = "INST - StrictNotEq";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let rhs = context.vm.pop();
        let lhs = context.vm.pop();
        context.vm.push(!lhs.strict_equals(&rhs));
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct In;

impl Operation for In {
    const NAME: &'static str = "In";
    const INSTRUCTION: &'static str = "INST - In";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let rhs = context.vm.pop();
        let lhs = context.vm.pop();

        if !rhs.is_object() {
            return context.throw_type_error(format!(
                "right-hand side of 'in' should be an object, got {}",
                rhs.type_of()
            ));
        }
        let key = lhs.to_property_key(context)?;
        let value = context.has_property(&rhs, &key)?;
        context.vm.push(value);
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct InstanceOf;

impl Operation for InstanceOf {
    const NAME: &'static str = "InstanceOf";
    const INSTRUCTION: &'static str = "INST - InstanceOf";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let target = context.vm.pop();
        let v = context.vm.pop();
        let value = v.instance_of(&target, context)?;

        context.vm.push(value);
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Void;

impl Operation for Void {
    const NAME: &'static str = "Void";
    const INSTRUCTION: &'static str = "INST - Void";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let _old = context.vm.pop();
        context.vm.push(JsValue::undefined());
        Ok(ShouldExit::False)
    }
}

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
