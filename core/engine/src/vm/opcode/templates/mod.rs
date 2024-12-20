use crate::{
    builtins::array::Array,
    js_string,
    object::IntegrityLevel,
    property::PropertyDescriptor,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `TemplateLookup` implements the Opcode Operation for `Opcode::TemplateLookup`
///
/// Operation:
///  - Lookup if a tagged template object is cached and skip the creation if it is.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TemplateLookup;

impl TemplateLookup {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        jump: u32,
        site: u64,
        dst: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        if let Some(template) = context.realm().lookup_template(site) {
            registers.set(dst, template.into());
            context.vm.frame_mut().pc = jump;
        }

        Ok(CompletionType::Normal)
    }
}

impl Operation for TemplateLookup {
    const NAME: &'static str = "TemplateLookup";
    const INSTRUCTION: &'static str = "INST - TemplateLookup";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let jump = context.vm.read::<u32>();
        let site = context.vm.read::<u64>();
        let dst = context.vm.read::<u8>().into();
        Self::operation(jump, site, dst, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let jump = context.vm.read::<u32>();
        let site = context.vm.read::<u64>();
        let dst = context.vm.read::<u16>().into();
        Self::operation(jump, site, dst, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let jump = context.vm.read::<u32>();
        let site = context.vm.read::<u64>();
        let dst = context.vm.read::<u32>();
        Self::operation(jump, site, dst, registers, context)
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
    fn operation(
        site: u64,
        dst: u32,
        count: u64,
        values: &[(u32, u32)],
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let template = Array::array_create(count, None, context).expect("cannot fail per spec");
        let raw_obj = Array::array_create(count, None, context).expect("cannot fail per spec");

        for (index, (cooked, raw)) in values.iter().enumerate() {
            let raw_value = registers.get(*raw);
            let cooked_value = registers.get(*cooked);
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

        registers.set(dst, template.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for TemplateCreate {
    const NAME: &'static str = "TemplateCreate";
    const INSTRUCTION: &'static str = "INST - TemplateCreate";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let site = context.vm.read::<u64>();
        let dst = context.vm.read::<u8>().into();
        let count = context.vm.read::<u8>().into();
        let mut values = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let cooked = context.vm.read::<u8>().into();
            let raw = context.vm.read::<u8>().into();
            values.push((cooked, raw));
        }
        Self::operation(site, dst, count, &values, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let site = context.vm.read::<u64>();
        let dst = context.vm.read::<u16>().into();
        let count = context.vm.read::<u16>().into();
        let mut values = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let cooked = context.vm.read::<u16>().into();
            let raw = context.vm.read::<u16>().into();
            values.push((cooked, raw));
        }
        Self::operation(site, dst, count, &values, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let site = context.vm.read::<u64>();
        let dst = context.vm.read::<u32>();
        let count = context.vm.read::<u32>().into();
        let mut values = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let cooked = context.vm.read::<u32>();
            let raw = context.vm.read::<u32>();
            values.push((cooked, raw));
        }
        Self::operation(site, dst, count, &values, registers, context)
    }
}
