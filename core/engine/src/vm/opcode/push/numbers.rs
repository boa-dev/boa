use crate::{
    Context,
    vm::opcode::{Operation, VaryingOperand},
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
            #[inline(always)]
            pub(crate) fn operation((dst, value): (VaryingOperand, $num_type),  context: &mut Context) {
                context.vm.set_register(dst.into(), i32::from(value).into());
            }
        }

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);
            const COST: u8 = 1;
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
            #[inline(always)]
            pub(crate) fn operation((dst, value): (VaryingOperand, $num_type),  context: &mut Context) {
                context.vm.set_register(dst.into(), value.into());
            }
        }

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);
            const COST: u8 = 1;
        }
    };
}

implement_push_numbers_with_conversion!(PushInt8, i8, "Push `i8` value on the stack");
implement_push_numbers_with_conversion!(PushInt16, i16, "Push `i16` value on the stack");

implement_push_numbers_no_conversion!(PushInt32, i32, "Push `i32` value on the stack");
implement_push_numbers_no_conversion!(PushFloat, f32, "Push `f64` value on the stack");
implement_push_numbers_no_conversion!(PushDouble, f64, "Push `f64` value on the stack");
