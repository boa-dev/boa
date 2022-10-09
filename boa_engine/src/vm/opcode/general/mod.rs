use crate::{
    builtins::Number,
    value::Numeric,
    vm::{opcode::Operation, ShouldExit},
    Context, JsBigInt, JsResult, JsValue,
};
use std::ops::Neg as StdNeg;

pub(crate) mod jump;
pub(crate) mod logical;

pub(crate) use jump::*;
pub(crate) use logical::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Nop;

impl Operation for Nop {
    const NAME: &'static str = "Nop";
    const INSTRUCTION: &'static str = "INST - Nop";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
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
pub(crate) struct Inc;

impl Operation for Inc {
    const NAME: &'static str = "Inc";
    const INSTRUCTION: &'static str = "INST - Inc";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        match value.to_numeric(context)? {
            Numeric::Number(number) => context.vm.push(number + 1f64),
            Numeric::BigInt(bigint) => {
                context.vm.push(JsBigInt::add(&bigint, &JsBigInt::one()));
            }
        }
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct IncPost;

impl Operation for IncPost {
    const NAME: &'static str = "IncPost";
    const INSTRUCTION: &'static str = "INST - IncPost";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let value = value.to_numeric(context)?;
        context.vm.push(value.clone());
        match value {
            Numeric::Number(number) => context.vm.push(number + 1f64),
            Numeric::BigInt(bigint) => {
                context.vm.push(JsBigInt::add(&bigint, &JsBigInt::one()));
            }
        }
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Dec;

impl Operation for Dec {
    const NAME: &'static str = "Dec";
    const INSTRUCTION: &'static str = "INST - Dec";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        match value.to_numeric(context)? {
            Numeric::Number(number) => context.vm.push(number - 1f64),
            Numeric::BigInt(bigint) => {
                context.vm.push(JsBigInt::sub(&bigint, &JsBigInt::one()));
            }
        }
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DecPost;

impl Operation for DecPost {
    const NAME: &'static str = "DecPost";
    const INSTRUCTION: &'static str = "INST - DecPost";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let value = value.to_numeric(context)?;
        context.vm.push(value.clone());
        match value {
            Numeric::Number(number) => context.vm.push(number - 1f64),
            Numeric::BigInt(bigint) => {
                context.vm.push(JsBigInt::sub(&bigint, &JsBigInt::one()));
            }
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

macro_rules! implement_bin_ops {
    ($name:ident, $op:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub(crate) struct $name;

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);

            fn execute(context: &mut Context) -> JsResult<ShouldExit> {
                let rhs = context.vm.pop();
                let lhs = context.vm.pop();
                let value = lhs.$op(&rhs, context)?;
                context.vm.push(value);
                Ok(ShouldExit::False)
            }
        }
    };
}

implement_bin_ops!(Add, add);
implement_bin_ops!(Sub, sub);
implement_bin_ops!(Mul, mul);
implement_bin_ops!(Div, div);
implement_bin_ops!(Pow, pow);
implement_bin_ops!(Mod, rem);
implement_bin_ops!(BitAnd, bitand);
implement_bin_ops!(BitOr, bitor);
implement_bin_ops!(BitXor, bitxor);
implement_bin_ops!(ShiftLeft, shl);
implement_bin_ops!(ShiftRight, shr);
implement_bin_ops!(UnsignedShiftRight, ushr);
implement_bin_ops!(Eq, equals);
implement_bin_ops!(GreaterThan, gt);
implement_bin_ops!(GreaterThanOrEq, ge);
implement_bin_ops!(LessThan, lt);
implement_bin_ops!(LessThanOrEq, le);
