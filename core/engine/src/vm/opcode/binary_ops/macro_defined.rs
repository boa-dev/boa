use crate::{
    vm::opcode::{Operation, VaryingOperand},
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
            #[inline(always)]
            pub(crate) fn operation(
                (dst, lhs, rhs): (VaryingOperand, VaryingOperand, VaryingOperand),
                context: &mut Context,
            ) -> JsResult<()> {
                let lhs = context.vm.get_register(lhs.into()).clone();
                let rhs = context.vm.get_register(rhs.into()).clone();
                let value = lhs.$op(&rhs, context)?;
                context.vm.set_register(dst.into(), value.into());
                Ok(())
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
