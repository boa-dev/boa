use crate::{
    builtins::function::set_function_name,
    object::CONSTRUCTOR,
    property::PropertyDescriptor,
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context, JsString,
};

/// `DefineClassStaticSetterByName` implements the Opcode Operation for `Opcode::DefineClassStaticSetterByName`
///
/// Operation:
///  - Defines a class setter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticSetterByName;

impl Operation for DefineClassStaticSetterByName {
    const NAME: &'static str = "DefineClassStaticSetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticSetterByName";

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
            set_function_name(function_object, &key, Some(JsString::from("set")), context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class.clone());
            function_mut.set_class_object(class.clone());
        }
        let get = ok_or_throw_completion!(class.__get_own_property__(&key, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();
        ok_or_throw_completion!(
            class.__define_own_property__(
                &key,
                PropertyDescriptor::builder()
                    .maybe_set(Some(function))
                    .maybe_get(get)
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

/// `DefineClassSetterByName` implements the Opcode Operation for `Opcode::DefineClassSetterByName`
///
/// Operation:
///  - Defines a class setter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassSetterByName;

impl Operation for DefineClassSetterByName {
    const NAME: &'static str = "DefineClassSetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassSetterByName";

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
            set_function_name(function_object, &key, Some(JsString::from("set")), context);
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
        let get = ok_or_throw_completion!(class_proto.__get_own_property__(&key, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();
        ok_or_throw_completion!(
            class_proto.__define_own_property__(
                &key,
                PropertyDescriptor::builder()
                    .maybe_set(Some(function))
                    .maybe_get(get)
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

/// `DefineClassStaticSetterByValue` implements the Opcode Operation for `Opcode::DefineClassStaticSetterByValue`
///
/// Operation:
///  - Defines a class setter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticSetterByValue;

impl Operation for DefineClassStaticSetterByValue {
    const NAME: &'static str = "DefineClassStaticSetterByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticSetterByValue";

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
            set_function_name(function_object, &key, Some(JsString::from("set")), context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class.clone());
            function_mut.set_class_object(class.clone());
        }
        let get = ok_or_throw_completion!(class.__get_own_property__(&key, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();
        ok_or_throw_completion!(
            class.define_property_or_throw(
                key,
                PropertyDescriptor::builder()
                    .maybe_set(Some(function))
                    .maybe_get(get)
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

/// `DefineClassSetterByValue` implements the Opcode Operation for `Opcode::DefineClassSetterByValue`
///
/// Operation:
///  - Defines a class setter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassSetterByValue;

impl Operation for DefineClassSetterByValue {
    const NAME: &'static str = "DefineClassSetterByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassSetterByValue";

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
            set_function_name(function_object, &key, Some(JsString::from("set")), context);
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
        let get = ok_or_throw_completion!(class_proto.__get_own_property__(&key, context), context)
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();
        ok_or_throw_completion!(
            class_proto.__define_own_property__(
                &key,
                PropertyDescriptor::builder()
                    .maybe_set(Some(function))
                    .maybe_get(get)
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
