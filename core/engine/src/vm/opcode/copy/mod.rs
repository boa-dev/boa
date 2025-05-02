use super::VaryingOperand;
use crate::{
    vm::{opcode::Operation, Registers},
    Context, JsResult,
};

/// `CopyDataProperties` implements the Opcode Operation for `Opcode::CopyDataProperties`
///
/// Operation:
///  - Copy all properties of one object to another object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CopyDataProperties;

impl CopyDataProperties {
    #[inline(always)]
    pub(super) fn operation(
        (object, source, keys): (VaryingOperand, VaryingOperand, Vec<VaryingOperand>),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let object = registers.get(object.into());
        let source = registers.get(source.into());
        let mut excluded_keys = Vec::with_capacity(keys.len());
        for key in keys {
            let key = registers.get(key.into());
            excluded_keys.push(
                key.to_property_key(context)
                    .expect("key must be property key"),
            );
        }
        let object = object.as_object().expect("not an object");
        object.copy_data_properties(source, excluded_keys, context)?;
        Ok(())
    }
}

impl Operation for CopyDataProperties {
    const NAME: &'static str = "CopyDataProperties";
    const INSTRUCTION: &'static str = "INST - CopyDataProperties";
    const COST: u8 = 6;
}
