use crate::{
    builtins::function::set_function_name,
    object::CONSTRUCTOR,
    property::PropertyDescriptor,
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context, JsString,
};

/// `DefineClassStaticMethodByName` implements the Opcode Operation for `Opcode::DefineClassStaticMethodByName`
///
/// Operation:
///  - Defines a class method by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticMethodByName;

impl Operation for DefineClassStaticMethodByName {
    const NAME: &'static str = "DefineClassStaticMethodByName";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticMethodByName";

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
            set_function_name(function_object, &key, None, context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class.clone());
            function_mut.set_class_object(class.clone());
        }
        ok_or_throw_completion!(
            class.__define_own_property__(
                &key,
                PropertyDescriptor::builder()
                    .value(function)
                    .writable(true)
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

/// `DefineClassMethodByName` implements the Opcode Operation for `Opcode::DefineClassMethodByName`
///
/// Operation:
///  - Defines a class method by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassMethodByName;

impl Operation for DefineClassMethodByName {
    const NAME: &'static str = "DefineClassMethodByName";
    const INSTRUCTION: &'static str = "INST - DefineClassMethodByName";

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
            set_function_name(function_object, &key, None, context);
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
        ok_or_throw_completion!(
            class_proto.__define_own_property__(
                &key,
                PropertyDescriptor::builder()
                    .value(function)
                    .writable(true)
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

/// `DefineClassStaticMethodByValue` implements the Opcode Operation for `Opcode::DefineClassStaticMethodByValue`
///
/// Operation:
///  - Defines a class method by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticMethodByValue;

impl Operation for DefineClassStaticMethodByValue {
    const NAME: &'static str = "DefineClassStaticMethodByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticMethodByValue";

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
            set_function_name(function_object, &key, None, context);
            let mut function_object_mut = function_object.borrow_mut();
            let function_mut = function_object_mut
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class.clone());
            function_mut.set_class_object(class.clone());
        }
        ok_or_throw_completion!(
            class.define_property_or_throw(
                key,
                PropertyDescriptor::builder()
                    .value(function)
                    .writable(true)
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

/// `DefineClassMethodByValue` implements the Opcode Operation for `Opcode::DefineClassMethodByValue`
///
/// Operation:
///  - Defines a class method by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassMethodByValue;

impl Operation for DefineClassMethodByValue {
    const NAME: &'static str = "DefineClassMethodByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassMethodByValue";

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
            set_function_name(function_object, &key, None, context);
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
        ok_or_throw_completion!(
            class_proto.__define_own_property__(
                &key,
                PropertyDescriptor::builder()
                    .value(function)
                    .writable(true)
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
