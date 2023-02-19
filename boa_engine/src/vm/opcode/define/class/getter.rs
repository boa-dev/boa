use crate::{
    builtins::function::set_function_name,
    object::CONSTRUCTOR,
    property::PropertyDescriptor,
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context, JsString,
};

/// `DefineClassStaticGetterByName` implements the Opcode Operation for `Opcode::DefineClassStaticGetterByName`
///
/// Operation:
///  - Defines a class getter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticGetterByName;

impl Operation for DefineClassStaticGetterByName {
    const NAME: &'static str = "DefineClassStaticGetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticGetterByName";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let index = context.vm.read::<u32>();
        let function = context.vm.pop();
        let class = context.vm.pop();
        let class = class.as_object().expect("class must be object");
        let key = context
            .interner()
            .resolve_expect(context.vm.frame().code_block.names[index as usize].sym())
            .into_common::<JsString>(false)
            .into();
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, Some(JsString::from("get")), context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class.clone());
            function_mut.set_class_object(class.clone());
        }
        let set = ok_or_throw_completion!(class.__get_own_property__(&key, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        ok_or_throw_completion!(
            class.__define_own_property__(
                &key,
                PropertyDescriptor::builder()
                    .maybe_get(Some(function))
                    .maybe_set(set)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                context,
            ),
            context
        );
        CompletionType::Normal
    }
}

/// `DefineClassGetterByName` implements the Opcode Operation for `Opcode::DefineClassGetterByName`
///
/// Operation:
///  - Defines a class getter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassGetterByName;

impl Operation for DefineClassGetterByName {
    const NAME: &'static str = "DefineClassGetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassGetterByName";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let index = context.vm.read::<u32>();
        let function = context.vm.pop();
        let class_proto = context.vm.pop();
        let class_proto = class_proto.as_object().expect("class must be object");
        let key = context
            .interner()
            .resolve_expect(context.vm.frame().code_block.names[index as usize].sym())
            .into_common::<JsString>(false)
            .into();
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, Some(JsString::from("get")), context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class_proto.clone());
            let class = class_proto
                .get(CONSTRUCTOR, context)
                .expect("class prototype must have constructor")
                .as_object()
                .expect("class must be object")
                .clone();
            function_mut.set_class_object(class);
        }
        let set = ok_or_throw_completion!(class_proto.__get_own_property__(&key, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        ok_or_throw_completion!(
            class_proto.__define_own_property__(
                &key,
                PropertyDescriptor::builder()
                    .maybe_get(Some(function))
                    .maybe_set(set)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                context,
            ),
            context
        );
        CompletionType::Normal
    }
}

/// `DefineClassStaticGetterByValue` implements the Opcode Operation for `Opcode::DefineClassStaticGetterByValue`
///
/// Operation:
///  - Defines a class getter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticGetterByValue;

impl Operation for DefineClassStaticGetterByValue {
    const NAME: &'static str = "DefineClassStaticGetterByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticGetterByValue";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let function = context.vm.pop();
        let key = context.vm.pop();
        let class = context.vm.pop();
        let class = class.as_object().expect("class must be object");
        let key = key
            .to_property_key(context)
            .expect("property key must already be valid");
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, Some(JsString::from("get")), context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class.clone());
            function_mut.set_class_object(class.clone());
        }
        let set = ok_or_throw_completion!(class.__get_own_property__(&key, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        ok_or_throw_completion!(
            class.define_property_or_throw(
                key,
                PropertyDescriptor::builder()
                    .maybe_get(Some(function))
                    .maybe_set(set)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                context,
            ),
            context
        );
        CompletionType::Normal
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let function = context.vm.pop();
        let key = context.vm.pop();
        let class_proto = context.vm.pop();
        let class_proto = class_proto.as_object().expect("class must be object");
        let key = key
            .to_property_key(context)
            .expect("property key must already be valid");
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, Some(JsString::from("get")), context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class_proto.clone());
            let class = class_proto
                .get(CONSTRUCTOR, context)
                .expect("class prototype must have constructor")
                .as_object()
                .expect("class must be object")
                .clone();
            function_mut.set_class_object(class);
        }
        let set = ok_or_throw_completion!(class_proto.__get_own_property__(&key, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        ok_or_throw_completion!(
            class_proto.__define_own_property__(
                &key,
                PropertyDescriptor::builder()
                    .maybe_get(Some(function))
                    .maybe_set(set)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                context,
            ),
            context
        );
        CompletionType::Normal
    }
}
