use crate::{
    property::PropertyDescriptor,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsString,
};

/// `DefineClassSetterByName` implements the Opcode Operation for `Opcode::DefineClassSetterByName`
///
/// Operation:
///  - Defines a class setter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassSetterByName;

impl Operation for DefineClassSetterByName {
    const NAME: &'static str = "DefineClassSetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassSetterByName";

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
        let get = object
            .__get_own_property__(&name, context)?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();
        object.__define_own_property__(
            name,
            PropertyDescriptor::builder()
                .maybe_set(Some(value))
                .maybe_get(get)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}

/// `DefineClassSetterByValue` implements the Opcode Operation for `Opcode::DefineClassSetterByValue`
///
/// Operation:
///  - Defines a class setter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassSetterByValue;

impl Operation for DefineClassSetterByValue {
    const NAME: &'static str = "DefineClassSetterByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassSetterByValue";

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
        let get = object
            .__get_own_property__(&name, context)?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();
        object.__define_own_property__(
            name,
            PropertyDescriptor::builder()
                .maybe_set(Some(value))
                .maybe_get(get)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}
