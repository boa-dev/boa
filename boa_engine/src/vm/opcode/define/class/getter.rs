use crate::{
    property::PropertyDescriptor,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsString,
};

/// `DefineClassGetterByName` implements the Opcode Operation for `Opcode::DefineClassGetterByName`
///
/// Operation:
///  - Defines a class getter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassGetterByName;

impl Operation for DefineClassGetterByName {
    const NAME: &'static str = "DefineClassGetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassGetterByName";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let value = context.vm.pop();
        let object = context.vm.pop();
        let object = object.to_object(context)?;
        value
            .as_object()
            .expect("method must be function object")
            .borrow_mut()
            .as_function_mut()
            .expect("method must be function object")
            .set_home_object(object.clone());
        let name = context.vm.frame().code.names[index as usize];
        let name = context
            .interner()
            .resolve_expect(name.sym())
            .into_common::<JsString>(false)
            .into();
        let set = object
            .__get_own_property__(&name, context)?
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        object.__define_own_property__(
            name,
            PropertyDescriptor::builder()
                .maybe_get(Some(value))
                .maybe_set(set)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}

/// `DefineClassGetterByValue` implements the Opcode Operation for `Opcode::DefineClassGetterByValue`
///
/// Operation:
///  - Defines a class getter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassGetterByValue;

impl Operation for DefineClassGetterByValue {
    const NAME: &'static str = "DefineClassGetterByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassGetterByValue";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let object = context.vm.pop();
        let object = object.to_object(context)?;
        value
            .as_object()
            .expect("method must be function object")
            .borrow_mut()
            .as_function_mut()
            .expect("method must be function object")
            .set_home_object(object.clone());
        let name = key.to_property_key(context)?;
        let set = object
            .__get_own_property__(&name, context)?
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        object.__define_own_property__(
            name,
            PropertyDescriptor::builder()
                .maybe_get(Some(value))
                .maybe_set(set)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}
