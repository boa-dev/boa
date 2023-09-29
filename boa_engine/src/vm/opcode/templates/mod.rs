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

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
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

impl TemplateCreate {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, count: u32, site: u64) -> JsResult<CompletionType> {
        let template =
            Array::array_create(count.into(), None, context).expect("cannot fail per spec");
        let raw_obj =
            Array::array_create(count.into(), None, context).expect("cannot fail per spec");

        for index in (0..count).rev() {
            let raw_value = context.vm.pop();
            let cooked_value = context.vm.pop();
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

        context.vm.push(template);
        Ok(CompletionType::Normal)
    }
}

impl Operation for TemplateCreate {
    const NAME: &'static str = "TemplateCreate";
    const INSTRUCTION: &'static str = "INST - TemplateCreate";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let count = u32::from(context.vm.read::<u8>());
        let site = context.vm.read::<u64>();
        Self::operation(context, count, site)
    }

    fn u16_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let count = u32::from(context.vm.read::<u16>());
        let site = context.vm.read::<u64>();
        Self::operation(context, count, site)
    }

    fn u32_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let count = context.vm.read::<u32>();
        let site = context.vm.read::<u64>();
        Self::operation(context, count, site)
    }
}
