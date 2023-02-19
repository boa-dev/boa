use crate::{
    builtins::function::set_function_name,
    property::{PropertyDescriptor, PropertyKey},
    vm::{ok_or_throw_completion, opcode::Operation, throw_completion, CompletionType},
    Context, JsError, JsNativeError, JsString, JsValue,
};

/// `SetPropertyByName` implements the Opcode Operation for `Opcode::SetPropertyByName`
///
/// Operation:
///  - Sets a property by name of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertyByName;

impl Operation for SetPropertyByName {
    const NAME: &'static str = "SetPropertyByName";
    const INSTRUCTION: &'static str = "INST - SetPropertyByName";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let index = context.vm.read::<u32>();

        let value = context.vm.pop();
        let receiver = context.vm.pop();
        let object = context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            ok_or_throw_completion!(object.to_object(context), context)
        };

        let name = context.vm.frame().code_block.names[index as usize];
        let name: PropertyKey = context.interner().resolve_expect(name.sym()).utf16().into();

        //object.set(name, value.clone(), context.vm.frame().code.strict, context)?;
        let succeeded = ok_or_throw_completion!(
            object.__set__(name.clone(), value.clone(), receiver, context),
            context
        );
        if !succeeded && context.vm.frame().code_block.strict {
            throw_completion!(
                JsNativeError::typ()
                    .with_message(format!("cannot set non-writable property: {name}"))
                    .into(),
                JsError,
                context
            );
        }
        context.vm.stack.push(value);
        CompletionType::Normal
    }
}

/// `SetPropertyByValue` implements the Opcode Operation for `Opcode::SetPropertyByValue`
///
/// Operation:
///  - Sets a property by value of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertyByValue;

impl Operation for SetPropertyByValue {
    const NAME: &'static str = "SetPropertyByValue";
    const INSTRUCTION: &'static str = "INST - SetPropertyByValue";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let object = context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            ok_or_throw_completion!(object.to_object(context), context)
        };

        let key = ok_or_throw_completion!(key.to_property_key(context), context);
        ok_or_throw_completion!(
            object.set(
                key,
                value.clone(),
                context.vm.frame().code_block.strict,
                context,
            ),
            context
        );
        context.vm.stack.push(value);
        CompletionType::Normal
    }
}

/// `SetPropertyGetterByName` implements the Opcode Operation for `Opcode::SetPropertyGetterByName`
///
/// Operation:
///  - Sets a getter property by name of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertyGetterByName;

impl Operation for SetPropertyGetterByName {
    const NAME: &'static str = "SetPropertyGetterByName";
    const INSTRUCTION: &'static str = "INST - SetPropertyGetterByName";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let index = context.vm.read::<u32>();
        let value = context.vm.pop();
        let object = context.vm.pop();
        let object = ok_or_throw_completion!(object.to_object(context), context);
        let name = context.vm.frame().code_block.names[index as usize];
        let name = context
            .interner()
            .resolve_expect(name.sym())
            .into_common::<JsString>(false)
            .into();
        let set = ok_or_throw_completion!(object.__get_own_property__(&name, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        ok_or_throw_completion!(
            object.__define_own_property__(
                &name,
                PropertyDescriptor::builder()
                    .maybe_get(Some(value))
                    .maybe_set(set)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            ),
            context
        );
        CompletionType::Normal
    }
}

/// `SetPropertyGetterByValue` implements the Opcode Operation for `Opcode::SetPropertyGetterByValue`
///
/// Operation:
///  - Sets a getter property by value of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertyGetterByValue;

impl Operation for SetPropertyGetterByValue {
    const NAME: &'static str = "SetPropertyGetterByValue";
    const INSTRUCTION: &'static str = "INST - SetPropertyGetterByValue";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let object = context.vm.pop();
        let object = ok_or_throw_completion!(object.to_object(context), context);
        let name = ok_or_throw_completion!(key.to_property_key(context), context);
        let set = ok_or_throw_completion!(object.__get_own_property__(&name, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        ok_or_throw_completion!(
            object.__define_own_property__(
                &name,
                PropertyDescriptor::builder()
                    .maybe_get(Some(value))
                    .maybe_set(set)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            ),
            context
        );
        CompletionType::Normal
    }
}

/// `SetPropertySetterByName` implements the Opcode Operation for `Opcode::SetPropertySetterByName`
///
/// Operation:
///  - Sets a setter property by name of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertySetterByName;

impl Operation for SetPropertySetterByName {
    const NAME: &'static str = "SetPropertySetterByName";
    const INSTRUCTION: &'static str = "INST - SetPropertySetterByName";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let index = context.vm.read::<u32>();
        let value = context.vm.pop();
        let object = context.vm.pop();
        let object = ok_or_throw_completion!(object.to_object(context), context);
        let name = context.vm.frame().code_block.names[index as usize];
        let name = context
            .interner()
            .resolve_expect(name.sym())
            .into_common::<JsString>(false)
            .into();
        let get = ok_or_throw_completion!(object.__get_own_property__(&name, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();
        ok_or_throw_completion!(
            object.__define_own_property__(
                &name,
                PropertyDescriptor::builder()
                    .maybe_set(Some(value))
                    .maybe_get(get)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            ),
            context
        );
        CompletionType::Normal
    }
}

/// `SetPropertySetterByValue` implements the Opcode Operation for `Opcode::SetPropertySetterByValue`
///
/// Operation:
///  - Sets a setter property by value of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertySetterByValue;

impl Operation for SetPropertySetterByValue {
    const NAME: &'static str = "SetPropertySetterByValue";
    const INSTRUCTION: &'static str = "INST - SetPropertySetterByValue";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let object = context.vm.pop();
        let object = ok_or_throw_completion!(object.to_object(context), context);
        let name = ok_or_throw_completion!(key.to_property_key(context), context);
        let get = ok_or_throw_completion!(object.__get_own_property__(&name, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();
        ok_or_throw_completion!(
            object.__define_own_property__(
                &name,
                PropertyDescriptor::builder()
                    .maybe_set(Some(value))
                    .maybe_get(get)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            ),
            context
        );
        CompletionType::Normal
    }
}

/// `SetFunctionName` implements the Opcode Operation for `Opcode::SetFunctionName`
///
/// Operation:
///  - Sets the name of a function object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetFunctionName;

impl Operation for SetFunctionName {
    const NAME: &'static str = "SetFunctionName";
    const INSTRUCTION: &'static str = "INST - SetFunctionName";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let prefix = context.vm.read::<u8>();
        let function = context.vm.pop();
        let name = context.vm.pop();

        let name = match name {
            JsValue::String(name) => name.into(),
            JsValue::Symbol(name) => name.into(),
            _ => unreachable!(),
        };

        let prefix = match prefix {
            1 => Some(JsString::from("get")),
            2 => Some(JsString::from("set")),
            _ => None,
        };

        set_function_name(
            function.as_object().expect("function is not an object"),
            &name,
            prefix,
            context,
        );

        context.vm.stack.push(function);
        CompletionType::Normal
    }
}
