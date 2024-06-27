use crate::{
    vm::{opcode::Operation, CompletionType, InstructionOperand},
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
                lhs: InstructionOperand,
                rhs: InstructionOperand,
                context: &mut Context,
            ) -> JsResult<CompletionType> {
                let rp = context.vm.frame().rp;

                let lhs = lhs.to_value(&context.vm);
                let rhs = rhs.to_value(&context.vm);

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
                let output = u32::from(context.vm.read::<u8>());
                let lhs = InstructionOperand::from(context.vm.read::<u8>());
                let rhs = InstructionOperand::from(context.vm.read::<u8>());
                Self::operation(output, lhs, rhs, context)
            }

            fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
                let output = u32::from(context.vm.read::<u16>());
                let lhs = InstructionOperand::from(context.vm.read::<u16>());
                let rhs = InstructionOperand::from(context.vm.read::<u16>());
                Self::operation(output, lhs, rhs, context)
            }

            fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
                let output = context.vm.read::<u32>();
                let lhs = InstructionOperand::from(context.vm.read::<u32>());
                let rhs = InstructionOperand::from(context.vm.read::<u32>());
                Self::operation(output, lhs, rhs, context)
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
