use super::VaryingOperand;
use crate::{
    Context, builtins::array::Array, js_string, object::IntegrityLevel,
    property::PropertyDescriptor, vm::opcode::Operation,
};
use thin_vec::ThinVec;

/// `TemplateLookup` implements the Opcode Operation for `Opcode::TemplateLookup`
///
/// Operation:
///  - Lookup if a tagged template object is cached and skip the creation if it is.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TemplateLookup;

impl TemplateLookup {
    #[inline(always)]
    pub(super) fn operation((jump, site, dst): (u32, u64, VaryingOperand), context: &mut Context) {
        if let Some(template) = context.realm().lookup_template(site) {
            context.vm.set_register(dst.into(), template.into());
            context.vm.frame_mut().pc = jump;
        }
    }
}

impl Operation for TemplateLookup {
    const NAME: &'static str = "TemplateLookup";
    const INSTRUCTION: &'static str = "INST - TemplateLookup";
    const COST: u8 = 3;
}

/// `TemplateCreate` implements the Opcode Operation for `Opcode::TemplateCreate`
///
/// Operation:
///  - Create a new tagged template object and cache it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TemplateCreate;

impl TemplateCreate {
    #[inline(always)]
    pub(super) fn operation(
        (site, dst, values): (u64, VaryingOperand, ThinVec<u32>),
        context: &mut Context,
    ) {
        let count = values.len() / 2;
        let template = Array::array_create(count, None, context).expect("cannot fail per spec");
        let raw_obj = Array::array_create(count, None, context).expect("cannot fail per spec");

        let mut index = 0;
        let mut cooked = true;
        for value in values {
            if cooked {
                let cooked_value = context.vm.get_register(value as usize);
                template
                    .define_property_or_throw(
                        index,
                        PropertyDescriptor::builder()
                            .value(cooked_value.clone())
                            .writable(false)
                            .enumerable(true)
                            .configurable(false),
                        context,
                    )
                    .expect("should not fail on new array");
            } else {
                let raw_value = context.vm.get_register(value as usize);
                raw_obj
                    .define_property_or_throw(
                        index,
                        PropertyDescriptor::builder()
                            .value(raw_value.clone())
                            .writable(false)
                            .enumerable(true)
                            .configurable(false),
                        context,
                    )
                    .expect("should not fail on new array");
                index += 1;
            }

            cooked = !cooked;
        }

        raw_obj
            .set_integrity_level(IntegrityLevel::Frozen, context)
            .expect("should never fail per spec");
        template
            .define_property_or_throw(
                js_string!("raw"),
                PropertyDescriptor::builder()
                    .value(raw_obj)
                    .writable(false)
                    .enumerable(false)
                    .configurable(false),
                context,
            )
            .expect("should never fail per spec");
        template
            .set_integrity_level(IntegrityLevel::Frozen, context)
            .expect("should never fail per spec");

        context.realm().push_template(site, template.clone());

        context.vm.set_register(dst.into(), template.into());
    }
}

impl Operation for TemplateCreate {
    const NAME: &'static str = "TemplateCreate";
    const INSTRUCTION: &'static str = "INST - TemplateCreate";
    const COST: u8 = 6;
}
