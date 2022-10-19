use crate::{
    property::PropertyDescriptor,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsString,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DefineOwnPropertyByName;

impl Operation for DefineOwnPropertyByName {
    const NAME: &'static str = "DefineOwnPropertyByName";
    const INSTRUCTION: &'static str = "INST - DefineOwnPropertyByName";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let object = context.vm.pop();
        let value = context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            object.to_object(context)?
        };
        let name = context.vm.frame().code.names[index as usize];
        let name = context
            .interner()
            .resolve_expect(name.sym())
            .into_common::<JsString>(false);
        object.__define_own_property__(
            name.into(),
            PropertyDescriptor::builder()
                .value(value)
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DefineOwnPropertyByValue;

impl Operation for DefineOwnPropertyByValue {
    const NAME: &'static str = "DefineOwnPropertyByValue";
    const INSTRUCTION: &'static str = "INST - DefineOwnPropertyByValue";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let object = context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            object.to_object(context)?
        };
        let key = key.to_property_key(context)?;
        object.__define_own_property__(
            key,
            PropertyDescriptor::builder()
                .value(value)
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}
