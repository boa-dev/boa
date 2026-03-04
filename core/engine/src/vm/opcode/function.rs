use crate::{
    Context, JsExpect, JsResult, JsValue,
    builtins::function::OrdinaryFunction,
    vm::opcode::{Operation, VaryingOperand},
};

/// `SetHomeObject` implements the Opcode Operation for `Opcode::SetHomeObject`
///
/// Operation:
///  - Set home object internal slot of a function object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetHomeObject;

impl SetHomeObject {
    #[inline(always)]
    pub(crate) fn operation(
        (function, home): (VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let function = context.vm.get_register(function.into());
        let home = context.vm.get_register(home.into());

        function
            .as_object()
            .js_expect("must be object")?
            .downcast_mut::<OrdinaryFunction>()
            .js_expect("must be function object")?
            .set_home_object(home.as_object().js_expect("must be object")?.clone());

        Ok(())
    }
}

impl Operation for SetHomeObject {
    const NAME: &'static str = "SetHomeObject";
    const INSTRUCTION: &'static str = "INST - SetHomeObject";
    const COST: u8 = 4;
}

/// `GetHomeObject` implements the Opcode Operation for `Opcode::GetHomeObject`
///
/// Operation:
///  - Get the home object internal slot of a function object (null if not set).
///
/// Registers (in):
///  - `function`: `JsObject<OrdinaryFunction>`
///
/// Registers (out):
///  - `home`: `JsObject` or `null` if the home object is not set.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetHomeObject;

impl GetHomeObject {
    #[inline(always)]
    pub(crate) fn operation(function: VaryingOperand, context: &mut Context) -> JsResult<()> {
        let function_v = context.vm.get_register(function.into());

        let home_object = function_v
            .as_object()
            .js_expect("must be object")?
            .downcast_ref::<OrdinaryFunction>()
            .js_expect("must be function object")?
            .get_home_object()
            .map_or_else(JsValue::null, |o| o.clone().into());

        context.vm.set_register(function.into(), home_object);
        Ok(())
    }
}

impl Operation for GetHomeObject {
    const NAME: &'static str = "GetHomeObject";
    const INSTRUCTION: &'static str = "INST - GetHomeObject";
    const COST: u8 = 4;
}
