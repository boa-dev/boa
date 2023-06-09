use crate::{
    builtins::array::Array,
    object::IntegrityLevel,
    property::PropertyDescriptor,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};
use boa_macros::utf16;

/// `TemplateLookup` implements the Opcode Operation for `Opcode::TemplateLookup`
///
/// Operation:
///  - Lookup if a tagged template object is cached and skip the creation if it is.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TemplateLookup;

impl Operation for TemplateLookup {
    const NAME: &'static str = "TemplateLookup";
    const INSTRUCTION: &'static str = "INST - TemplateLookup";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let jump = context.vm.read::<u32>();
        let site = context.vm.read::<u64>();

        if let Some(template) = context.realm().lookup_template(site) {
            context.vm.push(template);
            context.vm.frame_mut().pc = jump;
        }

        Ok(CompletionType::Normal)
    }
}

/// `TemplateCreate` implements the Opcode Operation for `Opcode::TemplateCreate`
///
/// Operation:
///  - Create a new tagged template object and cache it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TemplateCreate;

impl Operation for TemplateCreate {
    const NAME: &'static str = "TemplateCreate";
    const INSTRUCTION: &'static str = "INST - TemplateCreate";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let count = raw_context.vm.read::<u32>();
        let site = raw_context.vm.read::<u64>();

        let template =
            Array::array_create(count.into(), None, context).expect("cannot fail per spec");
        let raw_obj =
            Array::array_create(count.into(), None, context).expect("cannot fail per spec");

        for index in (0..count).rev() {
            let raw_context = context.as_raw_context_mut();
            let raw_value = raw_context.vm.pop();
            let cooked_value = raw_context.vm.pop();
            template
                .define_property_or_throw(
                    index,
                    PropertyDescriptor::builder()
                        .value(cooked_value)
                        .writable(false)
                        .enumerable(true)
                        .configurable(false),
                    context,
                )
                .expect("should not fail on new array");
            raw_obj
                .define_property_or_throw(
                    index,
                    PropertyDescriptor::builder()
                        .value(raw_value)
                        .writable(false)
                        .enumerable(true)
                        .configurable(false),
                    context,
                )
                .expect("should not fail on new array");
        }

        raw_obj
            .set_integrity_level(IntegrityLevel::Frozen, context)
            .expect("should never fail per spec");
        template
            .define_property_or_throw(
                utf16!("raw"),
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

        context.as_raw_context_mut().vm.push(template);
        Ok(CompletionType::Normal)
    }
}
