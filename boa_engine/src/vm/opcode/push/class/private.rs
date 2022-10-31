use crate::{
    object::PrivateElement,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `PushClassPrivateMethod` implements the Opcode Operation for `Opcode::PushClassPrivateMethod`
///
/// Operation:
///  - Push a private method to the class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassPrivateMethod;

impl Operation for PushClassPrivateMethod {
    const NAME: &'static str = "PushClassPrivateMethod";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateMethod";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.names[index as usize];
        let method = context.vm.pop();
        let method_object = method.as_callable().expect("method must be callable");
        let class = context.vm.pop();
        class
            .as_object()
            .expect("class must be function object")
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_private_method(name.sym(), PrivateElement::Method(method_object.clone()));
        Ok(ShouldExit::False)
    }
}

/// `PushClassPrivateGetter` implements the Opcode Operation for `Opcode::PushClassPrivateGetter`
///
/// Operation:
///  - Push a private getter to the class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassPrivateGetter;

impl Operation for PushClassPrivateGetter {
    const NAME: &'static str = "PushClassPrivateGetter";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateGetter";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.names[index as usize];
        let getter = context.vm.pop();
        let getter_object = getter.as_callable().expect("getter must be callable");
        let class = context.vm.pop();
        class
            .as_object()
            .expect("class must be function object")
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_private_method(
                name.sym(),
                PrivateElement::Accessor {
                    getter: Some(getter_object.clone()),
                    setter: None,
                },
            );
        Ok(ShouldExit::False)
    }
}

/// `PushClassPrivateSetter` implements the Opcode Operation for `Opcode::PushClassPrivateSetter`
///
/// Operation:
///  - Push a private setter to the class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassPrivateSetter;

impl Operation for PushClassPrivateSetter {
    const NAME: &'static str = "PushClassPrivateSetter";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateSetter";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.names[index as usize];
        let setter = context.vm.pop();
        let setter_object = setter.as_callable().expect("getter must be callable");
        let class = context.vm.pop();
        class
            .as_object()
            .expect("class must be function object")
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_private_method(
                name.sym(),
                PrivateElement::Accessor {
                    getter: None,
                    setter: Some(setter_object.clone()),
                },
            );
        Ok(ShouldExit::False)
    }
}
