use crate::{
    error::JsNativeError,
    object::PrivateElement,
    property::PropertyDescriptor,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `AssignPrivateField` implements the Opcode Operation for `Opcode::AssignPrivateField`
///
/// Operation:
///  - Assign the value of a private property of an object by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AssignPrivateField;

impl Operation for AssignPrivateField {
    const NAME: &'static str = "AssignPrivateField";
    const INSTRUCTION: &'static str = "INST - AssignPrivateField";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.private_names[index as usize];
        let value = context.vm.pop();
        let object = context.vm.pop();
        if let Some(object) = object.as_object() {
            let mut object_borrow_mut = object.borrow_mut();
            match object_borrow_mut.get_private_element(name) {
                Some(PrivateElement::Field(_)) => {
                    if !object_borrow_mut
                        .assign_private_element(name, PrivateElement::Field(value.clone()))
                    {
                        return Err(JsNativeError::typ()
                            .with_message("cannot assign to private field")
                            .into());
                    }
                }
                Some(PrivateElement::Method(_)) => {
                    return Err(JsNativeError::typ()
                        .with_message("private method is not writable")
                        .into());
                }
                Some(PrivateElement::Accessor {
                    setter: Some(setter),
                    ..
                }) => {
                    let setter = setter.clone();
                    drop(object_borrow_mut);
                    setter.call(&object.clone().into(), &[value.clone()], context)?;
                }
                None => {
                    return Err(JsNativeError::typ()
                        .with_message("private field not defined")
                        .into());
                }
                _ => {
                    return Err(JsNativeError::typ()
                        .with_message("private field defined without a setter")
                        .into());
                }
            }
        } else {
            return Err(JsNativeError::typ()
                .with_message("cannot set private property on non-object")
                .into());
        }
        context.vm.push(value);
        Ok(ShouldExit::False)
    }
}

/// `SetPrivateField` implements the Opcode Operation for `Opcode::SetPrivateField`
///
/// Operation:
///  - Set a private property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateField;

impl Operation for SetPrivateField {
    const NAME: &'static str = "SetPrivateValue";
    const INSTRUCTION: &'static str = "INST - SetPrivateValue";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.private_names[index as usize];
        let value = context.vm.pop();
        let object = context.vm.pop();
        if let Some(object) = object.as_object() {
            let mut object_borrow_mut = object.borrow_mut();
            if let Some(PrivateElement::Accessor {
                getter: _,
                setter: Some(setter),
            }) = object_borrow_mut.get_private_element(name)
            {
                let setter = setter.clone();
                drop(object_borrow_mut);
                setter.call(&object.clone().into(), &[value], context)?;
            } else {
                object_borrow_mut.set_private_element(name, PrivateElement::Field(value));
            }
        } else {
            return Err(JsNativeError::typ()
                .with_message("cannot set private property on non-object")
                .into());
        }
        Ok(ShouldExit::False)
    }
}

/// `SetPrivateMethod` implements the Opcode Operation for `Opcode::SetPrivateMethod`
///
/// Operation:
///  - Set a private method of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateMethod;

impl Operation for SetPrivateMethod {
    const NAME: &'static str = "SetPrivateMethod";
    const INSTRUCTION: &'static str = "INST - SetPrivateMethod";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.private_names[index as usize];
        let value = context.vm.pop();
        let value = value.as_callable().expect("method must be callable");

        let name_string = format!("#{}", context.interner().resolve_expect(name.description()));
        let desc = PropertyDescriptor::builder()
            .value(name_string)
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build();
        value
            .__define_own_property__("name".into(), desc, context)
            .expect("failed to set name property on private method");

        let object = context.vm.pop();
        if let Some(object) = object.as_object() {
            object
                .borrow_mut()
                .set_private_element(name, PrivateElement::Method(value.clone()));
            let mut value_mut = value.borrow_mut();
            let function = value_mut
                .as_function_mut()
                .expect("method must be a function");
            function.set_class_object(object.clone());
        } else {
            return Err(JsNativeError::typ()
                .with_message("cannot set private setter on non-object")
                .into());
        }
        Ok(ShouldExit::False)
    }
}

/// `SetPrivateSetter` implements the Opcode Operation for `Opcode::SetPrivateSetter`
///
/// Operation:
///  - Set a private setter property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateSetter;

impl Operation for SetPrivateSetter {
    const NAME: &'static str = "SetPrivateSetter";
    const INSTRUCTION: &'static str = "INST - SetPrivateSetter";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.private_names[index as usize];
        let value = context.vm.pop();
        let value = value.as_callable().expect("setter must be callable");
        let object = context.vm.pop();
        if let Some(object) = object.as_object() {
            object
                .borrow_mut()
                .set_private_element_setter(name, value.clone());
            let mut value_mut = value.borrow_mut();
            let function = value_mut
                .as_function_mut()
                .expect("method must be a function");
            function.set_class_object(object.clone());
        } else {
            return Err(JsNativeError::typ()
                .with_message("cannot set private setter on non-object")
                .into());
        }
        Ok(ShouldExit::False)
    }
}

/// `SetPrivateGetter` implements the Opcode Operation for `Opcode::SetPrivateGetter`
///
/// Operation:
///  - Set a private getter property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateGetter;

impl Operation for SetPrivateGetter {
    const NAME: &'static str = "SetPrivateGetter";
    const INSTRUCTION: &'static str = "INST - SetPrivateGetter";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.private_names[index as usize];
        let value = context.vm.pop();
        let value = value.as_callable().expect("getter must be callable");
        let object = context.vm.pop();
        if let Some(object) = object.as_object() {
            object
                .borrow_mut()
                .set_private_element_getter(name, value.clone());
            let mut value_mut = value.borrow_mut();
            let function = value_mut
                .as_function_mut()
                .expect("method must be a function");
            function.set_class_object(object.clone());
        } else {
            return Err(JsNativeError::typ()
                .with_message("cannot set private getter on non-object")
                .into());
        }
        Ok(ShouldExit::False)
    }
}
