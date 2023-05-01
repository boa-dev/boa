use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `CopyDataProperties` implements the Opcode Operation for `Opcode::CopyDataProperties`
///
/// Operation:
///  - Copy all properties of one object to another object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CopyDataProperties;

impl Operation for CopyDataProperties {
    const NAME: &'static str = "CopyDataProperties";
    const INSTRUCTION: &'static str = "INST - CopyDataProperties";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let excluded_key_count = context.vm.read::<u32>();
        let excluded_key_count_computed = context.vm.read::<u32>();
        let mut excluded_keys = Vec::with_capacity(excluded_key_count as usize);
        for _ in 0..excluded_key_count {
            let key = context.vm.pop();
            excluded_keys.push(
                key.to_property_key(context)
                    .expect("key must be property key"),
            );
        }
        let value = context.vm.pop();
        let object = value.as_object().expect("not an object");
        let source = context.vm.pop();
        for _ in 0..excluded_key_count_computed {
            let key = context
                .vm
                .frame_mut()
                .keys
                .pop()
                .expect("property key should have been pushed");
            excluded_keys.push(key);
        }
        object.copy_data_properties(&source, excluded_keys, context)?;
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}
