use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `SetHomeObject` implements the Opcode Operation for `Opcode::SetHomeObject`
///
/// Operation:
///  - Set home object internal slot of a function object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetHomeObject;

impl Operation for SetHomeObject {
    const NAME: &'static str = "SetHomeObject";
    const INSTRUCTION: &'static str = "INST - SetHomeObject";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let function = context.vm.pop();
        let home = context.vm.pop();

        {
            let function_object = function.as_object().expect("must be object");
            let home_object = home.as_object().expect("must be object");
            let mut function_object_mut = function_object.borrow_mut();
            let function_mut = function_object_mut
                .as_function_mut()
                .expect("must be function object");
            function_mut.set_home_object(home_object.clone());
            function_mut.set_class_object(home_object.clone());
        }

        context.vm.push(home);
        context.vm.push(function);
        Ok(ShouldExit::False)
    }
}
