use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `Jump` implements the Opcode Operation for `Opcode::Jump`
///
/// Operation:
///  - Unconditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Jump;

impl Operation for Jump {
    const NAME: &'static str = "Jump";
    const INSTRUCTION: &'static str = "INST - Jump";
    const COST: u8 = 1;

    fn execute(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        context.vm.frame_mut().pc = address;
        Ok(CompletionType::Normal)
    }
}

// `JumpIfTrue` implements the Opcode Operation for `Opcode::JumpIfTrue`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpIfTrue;

impl JumpIfTrue {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        value: u32,
        address: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        if value.to_boolean() {
            context.vm.frame_mut().pc = address;
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for JumpIfTrue {
    const NAME: &'static str = "JumpIfTrue";
    const INSTRUCTION: &'static str = "INST - JumpIfTrue";
    const COST: u8 = 1;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u8>().into();
        Self::operation(value, address, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u16>().into();
        Self::operation(value, address, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        Self::operation(value, address, registers, context)
    }
}

/// `JumpIfFalse` implements the Opcode Operation for `Opcode::JumpIfFalse`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpIfFalse;

impl JumpIfFalse {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        value: u32,
        address: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        if !value.to_boolean() {
            context.vm.frame_mut().pc = address;
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for JumpIfFalse {
    const NAME: &'static str = "JumpIfFalse";
    const INSTRUCTION: &'static str = "INST - JumpIfFalse";
    const COST: u8 = 1;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u8>().into();
        Self::operation(value, address, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u16>().into();
        Self::operation(value, address, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        Self::operation(value, address, registers, context)
    }
}

/// `JumpIfNotUndefined` implements the Opcode Operation for `Opcode::JumpIfNotUndefined`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpIfNotUndefined;

impl JumpIfNotUndefined {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        value: u32,
        address: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        if !value.is_undefined() {
            context.vm.frame_mut().pc = address;
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for JumpIfNotUndefined {
    const NAME: &'static str = "JumpIfNotUndefined";
    const INSTRUCTION: &'static str = "INST - JumpIfNotUndefined";
    const COST: u8 = 1;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u8>().into();
        Self::operation(value, address, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u16>().into();
        Self::operation(value, address, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        Self::operation(value, address, registers, context)
    }
}

/// `JumpIfNullOrUndefined` implements the Opcode Operation for `Opcode::JumpIfNullOrUndefined`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpIfNullOrUndefined;

impl JumpIfNullOrUndefined {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        value: u32,
        address: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        if value.is_null_or_undefined() {
            context.vm.frame_mut().pc = address;
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for JumpIfNullOrUndefined {
    const NAME: &'static str = "JumpIfNullOrUndefined";
    const INSTRUCTION: &'static str = "INST - JumpIfNullOrUndefined";
    const COST: u8 = 1;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u8>().into();
        Self::operation(value, address, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u16>().into();
        Self::operation(value, address, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        Self::operation(value, address, registers, context)
    }
}

/// `JumpTable` implements the Opcode Operation for `Opcode::JumpTable`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpTable;

impl Operation for JumpTable {
    const NAME: &'static str = "JumpTable";
    const INSTRUCTION: &'static str = "INST - JumpTable";
    const COST: u8 = 5;

    fn execute(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let default = context.vm.read::<u32>();
        let count = context.vm.read::<u32>();

        let value = context.vm.pop();
        if let Some(value) = value.as_i32() {
            let value = value as u32;
            let mut target = None;
            for i in 0..count {
                let address = context.vm.read::<u32>();
                if i + 1 == value {
                    target = Some(address);
                }
            }

            context.vm.frame_mut().pc = target.unwrap_or(default);

            return Ok(CompletionType::Normal);
        }

        unreachable!("expected positive integer, got {value:?}")
    }
}
