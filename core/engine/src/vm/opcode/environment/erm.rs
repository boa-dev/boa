use crate::{
    Context, JsNativeError, JsResult,
    resource_management::{DisposableResource, DisposableResourceHint, DisposableResourceStack},
    vm::opcode::{Operation, RegisterOperand},
};

/// `AddDisposableResource` implements the Opcode Operation for `Opcode::AddDisposableResource`
///
/// Operation:
///  - Track a synchronous disposable resource in the current scope.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AddDisposableResource;

impl AddDisposableResource {
    #[inline(always)]
    pub(crate) fn operation(value: RegisterOperand, context: &mut Context) -> JsResult<()> {
        let value_js = context.vm.get_register(value.into()).clone();

        // If the value is null or undefined, skip disposal as per spec.
        if value_js.is_null_or_undefined() {
            return Ok(());
        }

        let dispose_method = value_js.get_method(crate::symbol::JsSymbol::dispose(), context)?;

        let Some(dispose_method) = dispose_method else {
            return Err(JsNativeError::typ()
                .with_message("Resource is not synchronously disposable")
                .into());
        };

        let frame = context.vm.frame_mut();

        // If no scope-level stack exists yet (e.g. top-level `using` without
        // an explicit block), create a fallback stack.
        if frame.disposable_resource_stacks.is_empty() {
            frame
                .disposable_resource_stacks
                .push(DisposableResourceStack::new());
        }

        frame
            .disposable_resource_stacks
            .last_mut()
            .expect("just ensured non-empty")
            .push(DisposableResource::new(
                value_js,
                dispose_method.into(),
                DisposableResourceHint::SyncDispose,
            ));

        Ok(())
    }
}

impl Operation for AddDisposableResource {
    const NAME: &'static str = "AddDisposableResource";
    const INSTRUCTION: &'static str = "INST - AddDisposableResource";
    const COST: u8 = 4;
}

/// `AddAsyncDisposableResource` implements the Opcode Operation for `Opcode::AddAsyncDisposableResource`
///
/// Operation:
///  - Track an asynchronous disposable resource in the current scope.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AddAsyncDisposableResource;

impl AddAsyncDisposableResource {
    #[inline(always)]
    pub(crate) fn operation(value: RegisterOperand, context: &mut Context) -> JsResult<()> {
        let value_js = context.vm.get_register(value.into()).clone();

        if value_js.is_null_or_undefined() {
            return Ok(());
        }

        // Try asyncDispose first, fallback to sync dispose later (to be done eventually,
        // but for now let's just grab asyncDispose per basic requirements)
        let mut dispose_method =
            value_js.get_method(crate::symbol::JsSymbol::async_dispose(), context)?;

        let mut hint = DisposableResourceHint::AsyncDispose;

        if dispose_method.is_none() {
            dispose_method = value_js.get_method(crate::symbol::JsSymbol::dispose(), context)?;
            hint = DisposableResourceHint::SyncDispose;
        }

        let Some(dispose_method) = dispose_method else {
            return Err(JsNativeError::typ()
                .with_message("Resource is not asynchronously disposable")
                .into());
        };

        let frame = context.vm.frame_mut();

        // Fallback: create a scope-level stack if none exists.
        if frame.disposable_resource_stacks.is_empty() {
            frame
                .disposable_resource_stacks
                .push(DisposableResourceStack::new());
        }

        frame
            .disposable_resource_stacks
            .last_mut()
            .expect("just ensured non-empty")
            .push(DisposableResource::new(
                value_js,
                dispose_method.into(),
                hint,
            ));

        Ok(())
    }
}

impl Operation for AddAsyncDisposableResource {
    const NAME: &'static str = "AddAsyncDisposableResource";
    const INSTRUCTION: &'static str = "INST - AddAsyncDisposableResource";
    const COST: u8 = 4;
}
