use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsString,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DeletePropertyByName;

impl Operation for DeletePropertyByName {
    const NAME: &'static str = "DeletePropertyByName";
    const INSTRUCTION: &'static str = "INST - DeletePropertyByName";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let key = context.vm.frame().code.names[index as usize];
        let key = context
            .interner()
            .resolve_expect(key)
            .into_common::<JsString>(false)
            .into();
        let object = context.vm.pop();
        let result = object.to_object(context)?.__delete__(&key, context)?;
        if !result && context.vm.frame().code.strict {
            return Err(context.construct_type_error("Cannot delete property"));
        }
        context.vm.push(result);
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DeletePropertyByValue;

impl Operation for DeletePropertyByValue {
    const NAME: &'static str = "DeletePropertyByValue";
    const INSTRUCTION: &'static str = "INST - DeletePropertyByValue";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let object = context.vm.pop();
        let key = context.vm.pop();
        let result = object
            .to_object(context)?
            .__delete__(&key.to_property_key(context)?, context)?;
        if !result && context.vm.frame().code.strict {
            return Err(context.construct_type_error("Cannot delete property"));
        }
        context.vm.push(result);
        Ok(ShouldExit::False)
    }
}
