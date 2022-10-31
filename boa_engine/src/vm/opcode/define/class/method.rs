use crate::{
    property::PropertyDescriptor,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsString,
};

/// `DefineClassMethodByName` implements the Opcode Operation for `Opcode::DefineClassMethodByName`
///
/// Operation:
///  - Defines a class method by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassMethodByName;

impl Operation for DefineClassMethodByName {
    const NAME: &'static str = "DefineClassMethodByName";
    const INSTRUCTION: &'static str = "INST - DefineClassMethodByName";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let value = context.vm.pop();
        let object = context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            object.to_object(context)?
        };
        value
            .as_object()
            .expect("method must be function object")
            .borrow_mut()
            .as_function_mut()
            .expect("method must be function object")
            .set_home_object(object.clone());
        let name = context.vm.frame().code.names[index as usize];
        let name = context.interner().resolve_expect(name.sym());
        object.__define_own_property__(
            name.into_common::<JsString>(false).into(),
            PropertyDescriptor::builder()
                .value(value)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}

/// `DefineClassMethodByValue` implements the Opcode Operation for `Opcode::DefineClassMethodByValue`
///
/// Operation:
///  - Defines a class method by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassMethodByValue;

impl Operation for DefineClassMethodByValue {
    const NAME: &'static str = "DefineClassMethodByName";
    const INSTRUCTION: &'static str = "INST - DefineClassMethodByName";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let object = context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            object.to_object(context)?
        };
        value
            .as_object()
            .expect("method must be function object")
            .borrow_mut()
            .as_function_mut()
            .expect("method must be function object")
            .set_home_object(object.clone());
        let key = key.to_property_key(context)?;
        object.__define_own_property__(
            key,
            PropertyDescriptor::builder()
                .value(value)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}
