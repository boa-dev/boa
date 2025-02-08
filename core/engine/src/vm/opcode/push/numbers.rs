use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
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

        impl $name {
            fn operation(dst: u32, value: $num_type, registers: &mut Registers) -> JsResult<CompletionType> {
                registers.set(dst, i32::from(value).into());
                Ok(CompletionType::Normal)
            }
        }

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);
            const COST: u8 = 1;

            fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
                let dst = context.vm.read::<u8>().into();
                let value = context.vm.read::<$num_type>();
                Self::operation(dst, value, registers)
            }

            fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
                let dst = context.vm.read::<u16>().into();
                let value = context.vm.read::<$num_type>();
                Self::operation(dst, value, registers)
            }

            fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
                let dst = context.vm.read::<u32>().into();
                let value = context.vm.read::<$num_type>();
                Self::operation(dst, value, registers)
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

        impl $name {
            fn operation(dst: u32, value: $num_type, registers: &mut Registers) -> JsResult<CompletionType> {
                registers.set(dst, value.into());
                Ok(CompletionType::Normal)
            }
        }

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);
            const COST: u8 = 1;

            fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
                let dst = context.vm.read::<u8>().into();
                let value = context.vm.read::<$num_type>();
                Self::operation(dst, value, registers)
            }

            fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
                let dst = context.vm.read::<u16>().into();
                let value = context.vm.read::<$num_type>();
                Self::operation(dst, value, registers)
            }

            fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
                let dst = context.vm.read::<u32>().into();
                let value = context.vm.read::<$num_type>();
                Self::operation(dst, value, registers)
            }
        }
    };
}

implement_push_numbers_with_conversion!(PushInt8, i8, "Push `i8` value on the stack");
implement_push_numbers_with_conversion!(PushInt16, i16, "Push `i16` value on the stack");

implement_push_numbers_no_conversion!(PushInt32, i32, "Push `i32` value on the stack");
implement_push_numbers_no_conversion!(PushFloat, f32, "Push `f64` value on the stack");
implement_push_numbers_no_conversion!(PushDouble, f64, "Push `f64` value on the stack");
