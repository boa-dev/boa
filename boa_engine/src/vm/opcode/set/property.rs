use crate::{
    property::{PropertyDescriptor, PropertyKey},
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SetPropertyByName;

impl Operation for SetPropertyByName {
    const NAME: &'static str = "SetPropertyByName";
    const INSTRUCTION: &'static str = "INST - SetPropertyByName";

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
        let name: PropertyKey = context.interner().resolve_expect(name).into();

        object.set(name, value, context.vm.frame().code.strict, context)?;
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SetPropertyByValue;

impl Operation for SetPropertyByValue {
    const NAME: &'static str = "SetPropertyByValue";
    const INSTRUCTION: &'static str = "INST - SetPropertyByValue";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let object = context.vm.pop();
        let key = context.vm.pop();
        let value = context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            object.to_object(context)?
        };

        let key = key.to_property_key(context)?;
        object.set(key, value, context.vm.frame().code.strict, context)?;
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SetPropertyGetterByName;

impl Operation for SetPropertyGetterByName {
    const NAME: &'static str = "SetPropertyGetterByName";
    const INSTRUCTION: &'static str = "INST - SetPropertyGetterByName";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let object = context.vm.pop();
        let value = context.vm.pop();
        let object = object.to_object(context)?;
        let name = context.vm.frame().code.names[index as usize];
        let name = context.interner().resolve_expect(name).into();
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
                .enumerable(true)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SetPropertyGetterByValue;

impl Operation for SetPropertyGetterByValue {
    const NAME: &'static str = "SetPropertyGetterByValue";
    const INSTRUCTION: &'static str = "INST - SetPropertyGetterByValue";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let object = context.vm.pop();
        let object = object.to_object(context)?;
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
                .enumerable(true)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SetPropertySetterByName;

impl Operation for SetPropertySetterByName {
    const NAME: &'static str = "SetPropertySetterByName";
    const INSTRUCTION: &'static str = "INST - SetPropertySetterByName";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let object = context.vm.pop();
        let value = context.vm.pop();
        let object = object.to_object(context)?;
        let name = context.vm.frame().code.names[index as usize];
        let name = context.interner().resolve_expect(name).into();
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
                .enumerable(true)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SetPropertySetterByValue;

impl Operation for SetPropertySetterByValue {
    const NAME: &'static str = "SetPropertySetterByValue";
    const INSTRUCTION: &'static str = "INST - SetPropertySetterByValue";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let object = context.vm.pop();
        let object = object.to_object(context)?;
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
                .enumerable(true)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(ShouldExit::False)
    }
}
