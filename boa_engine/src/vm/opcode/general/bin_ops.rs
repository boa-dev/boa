use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

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
