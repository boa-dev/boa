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

impl CopyDataProperties {
    fn operation(
        context: &mut Context<'_>,
        excluded_key_count: usize,
        excluded_key_count_computed: usize,
    ) -> JsResult<CompletionType> {
        let mut excluded_keys = Vec::with_capacity(excluded_key_count);
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
            let key = context.vm.pop();
            excluded_keys.push(
                key.to_property_key(context)
                    .expect("key must be property key"),
            );
        }
        object.copy_data_properties(&source, excluded_keys, context)?;
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for CopyDataProperties {
    const NAME: &'static str = "CopyDataProperties";
    const INSTRUCTION: &'static str = "INST - CopyDataProperties";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let excluded_key_count = context.vm.read::<u8>() as usize;
        let excluded_key_count_computed = context.vm.read::<u8>() as usize;
        Self::operation(context, excluded_key_count, excluded_key_count_computed)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let excluded_key_count = context.vm.read::<u16>() as usize;
        let excluded_key_count_computed = context.vm.read::<u16>() as usize;
        Self::operation(context, excluded_key_count, excluded_key_count_computed)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let excluded_key_count = context.vm.read::<u32>() as usize;
        let excluded_key_count_computed = context.vm.read::<u32>() as usize;
        Self::operation(context, excluded_key_count, excluded_key_count_computed)
    }
}
