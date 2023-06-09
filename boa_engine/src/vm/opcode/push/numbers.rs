use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

macro_rules! implement_push_numbers_with_conversion {
    ($name:ident, $num_type:ty, $doc_string:literal) => {
        #[doc= concat!("`", stringify!($name), "` implements the OpCode Operation for `Opcode::", stringify!($name), "`\n")]
        #[doc= "\n"]
        #[doc="Operation:\n"]
        #[doc= concat!(" - ", $doc_string)]
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct $name;

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);

            fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
                let context = context.as_raw_context_mut();
                let value = context.vm.read::<$num_type>();
                context.vm.push(i32::from(value));
                Ok(CompletionType::Normal)
            }
        }
    };
}

macro_rules! implement_push_numbers_no_conversion {
    ($name:ident, $num_type:ty, $doc_string:literal) => {
        #[doc= concat!("`", stringify!($name), "` implements the OpCode Operation for `Opcode::", stringify!($name), "`\n")]
        #[doc= "\n"]
        #[doc="Operation:\n"]
        #[doc= concat!(" - ", $doc_string)]
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct $name;

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);

            fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
                let context = context.as_raw_context_mut();
                let value = context.vm.read::<$num_type>();
                context.vm.push(value);
                Ok(CompletionType::Normal)
            }
        }
    };
}

implement_push_numbers_with_conversion!(PushInt8, i8, "Push `i8` value on the stack");
implement_push_numbers_with_conversion!(PushInt16, i16, "Push `i16` value on the stack");

implement_push_numbers_no_conversion!(PushInt32, i32, "Push `i32` value on the stack");
implement_push_numbers_no_conversion!(PushRational, f64, "Push `f64` value on the stack");
