use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
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
            fn operation(
                output: u32,
                lhs: u32,
                rhs: u32,
                operand_types: u8,
                context: &mut Context,
            ) -> JsResult<CompletionType> {
                let rp = context.vm.frame().rp;

                let lhs = context.vm.frame().read_value::<0>(operand_types, lhs, &context.vm);
                let rhs = context.vm.frame().read_value::<1>(operand_types, rhs, &context.vm);

                let value = lhs.$op(&rhs, context)?;

                context.vm.stack[(rp + output) as usize] = JsValue::from(value);
                Ok(CompletionType::Normal)
            }
        }

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);
            const COST: u8 = 2;

            fn execute(context: &mut Context) -> JsResult<CompletionType> {
                let operand_types = context.vm.read::<u8>();
                let output = context.vm.read::<u8>().into();
                let lhs = context.vm.read::<u8>().into();
                let rhs = context.vm.read::<u8>().into();
                Self::operation(output, lhs, rhs, operand_types, context)
            }

            fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
                let operand_types = context.vm.read::<u8>();
                let output = context.vm.read::<u16>().into();
                let lhs = context.vm.read::<u16>().into();
                let rhs = context.vm.read::<u16>().into();
                Self::operation(output, lhs, rhs, operand_types, context)
            }

            fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
                let operand_types = context.vm.read::<u8>();
                let output = context.vm.read::<u32>();
                let lhs = context.vm.read::<u32>();
                let rhs = context.vm.read::<u32>();
                Self::operation(output, lhs, rhs, operand_types, context)
            }
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
implement_bin_ops!(InstanceOf, instance_of, "Binary `<=` operator.");
