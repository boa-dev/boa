use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsValue,
};

pub(crate) mod array;
pub(crate) mod class;
pub(crate) mod environment;
pub(crate) mod literal;
pub(crate) mod new_target;
pub(crate) mod numbers;
pub(crate) mod object;

pub(crate) use array::*;
pub(crate) use class::*;
pub(crate) use environment::*;
pub(crate) use literal::*;
pub(crate) use new_target::*;
pub(crate) use numbers::*;
pub(crate) use object::*;

macro_rules! implement_push_generics {
    ($name:ident, $push_value:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub(crate) struct $name;

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);

            fn execute(context: &mut Context) -> JsResult<ShouldExit> {
                context.vm.push($push_value);
                Ok(ShouldExit::False)
            }
        }
    };
}

implement_push_generics!(PushUndefined, JsValue::undefined());
implement_push_generics!(PushNull, JsValue::null());
implement_push_generics!(PushTrue, true);
implement_push_generics!(PushFalse, false);
implement_push_generics!(PushZero, 0);
implement_push_generics!(PushOne, 1);
implement_push_generics!(PushNaN, JsValue::nan());
implement_push_generics!(PushPositiveInfinity, JsValue::positive_infinity());
implement_push_generics!(PushNegativeInfinity, JsValue::negative_infinity());
