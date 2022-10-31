use crate::{
    error::JsNativeError,
    object::PrivateElement,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `GetPrivateField` implements the Opcode Operation for `Opcode::GetPrivateField`
///
/// Operation:
///  - Get a private property by name from an object an push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPrivateField;

impl Operation for GetPrivateField {
    const NAME: &'static str = "GetPrivateField";
    const INSTRUCTION: &'static str = "INST - GetPrivateField";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.names[index as usize];
        let value = context.vm.pop();
        if let Some(object) = value.as_object() {
            let object_borrow_mut = object.borrow();
            if let Some(element) = object_borrow_mut.get_private_element(name.sym()) {
                match element {
                    PrivateElement::Field(value) => context.vm.push(value),
                    PrivateElement::Method(method) => context.vm.push(method.clone()),
                    PrivateElement::Accessor {
                        getter: Some(getter),
                        setter: _,
                    } => {
                        let value = getter.call(&value, &[], context)?;
                        context.vm.push(value);
                    }
                    PrivateElement::Accessor { .. } => {
                        return Err(JsNativeError::typ()
                            .with_message("private property was defined without a getter")
                            .into());
                    }
                }
            } else {
                return Err(JsNativeError::typ()
                    .with_message("private property does not exist")
                    .into());
            }
        } else {
            return Err(JsNativeError::typ()
                .with_message("cannot read private property from non-object")
                .into());
        }
        Ok(ShouldExit::False)
    }
}
