use crate::{
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionType, Registers,
    },
    Context, JsResult,
};

macro_rules! implement_bin_ops {
    ($name:ident, $op:ident, $doc_string:literal) => {
        #[doc= concat!("`", stringify!($name), "` implements the OpCode Operation for `Opcode::", stringify!($name), "`\n")]
        #[doc= "\n"]
        #[doc="Operation:\n"]
        #[doc= concat!(" - ", $doc_string)]
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct $name;

        impl $name {
            #[allow(clippy::needless_pass_by_value)]
            #[inline(always)]
    pub(crate) fn operation(
                (dst, lhs, rhs): (VaryingOperand, VaryingOperand, VaryingOperand),
                registers: &mut Registers,
                context: &mut Context,
            ) -> JsResult<CompletionType> {
                let lhs = registers.get(lhs.into());
                let rhs = registers.get(rhs.into());
                let value = lhs.$op(&rhs, context)?;
                registers.set(dst.into(), value.into());
                Ok(CompletionType::Normal)
            }
        }

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);
            const COST: u8 = 2;
        }
    };
}

implement_bin_ops!(Add, add, "Binary `+` operator.");
implement_bin_ops!(Sub, sub, "Binary `-` operator.");
implement_bin_ops!(Mul, mul, "Binary `*` operator.");
implement_bin_ops!(Div, div, "Binary `/` operator.");
implement_bin_ops!(Pow, pow, "Binary `**` operator.");
implement_bin_ops!(Mod, rem, "Binary `%` operator.");
implement_bin_ops!(BitAnd, bitand, "Binary `&` operator.");
implement_bin_ops!(BitOr, bitor, "Binary `|` operator.");
implement_bin_ops!(BitXor, bitxor, "Binary `^` operator.");
implement_bin_ops!(ShiftLeft, shl, "Binary `<<` operator.");
implement_bin_ops!(ShiftRight, shr, "Binary `>>` operator.");
implement_bin_ops!(UnsignedShiftRight, ushr, "Binary `>>>` operator.");
implement_bin_ops!(Eq, equals, "Binary `==` operator.");
implement_bin_ops!(GreaterThan, gt, "Binary `>` operator.");
implement_bin_ops!(GreaterThanOrEq, ge, "Binary `>=` operator.");
implement_bin_ops!(LessThan, lt, "Binary `<` operator.");
implement_bin_ops!(LessThanOrEq, le, "Binary `<=` operator.");
implement_bin_ops!(InstanceOf, instance_of, "Binary `instanceof` operator.");
