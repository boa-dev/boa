use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult
};

macro_rules! implement_push_numbers_with_conversion {
    ($name:ident, $num_type:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub(crate) struct $name;

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);

            fn execute(context: &mut Context) -> JsResult<ShouldExit> {
                let value = context.vm.read::<$num_type>();
                context.vm.push(i32::from(value));
                Ok(ShouldExit::False)
            }
        }
    }
}

macro_rules! implement_push_numbers_no_conversion {
    ($name:ident, $num_type:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub(crate) struct $name;

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);

            fn execute(context: &mut Context) -> JsResult<ShouldExit> {
                let value = context.vm.read::<$num_type>();
                context.vm.push(value);
                Ok(ShouldExit::False)
            }
        }
    }
}
implement_push_numbers_with_conversion! (
    PushInt8, i8
);

implement_push_numbers_with_conversion!(
    PushInt16, i16
);

implement_push_numbers_no_conversion!(
    PushInt32, i32
);

implement_push_numbers_no_conversion!(
    PushRational, f64
);
