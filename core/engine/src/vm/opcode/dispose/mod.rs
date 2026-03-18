use super::RegisterOperand;
use crate::{
    Context, JsResult, JsValue, builtins::OrdinaryObject, error::JsNativeError, js_string,
    symbol::JsSymbol, vm::opcode::Operation,
};
use boa_gc::{Finalize, Trace};

/// A disposable resource entry in the disposal stack.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct DisposableResource {
    /// The resource value (or undefined for null/undefined resources).
    pub(crate) value: JsValue,
    /// The dispose method to call.
    pub(crate) method: JsValue,
}

/// Entry on the disposal stack — either a resource or a scope marker.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) enum DisposeEntry {
    /// A disposable resource.
    Resource(DisposableResource),
    /// A scope boundary marker.
    Marker,
}

/// `CreateDisposeCapability`
///
/// Pushes a scope marker onto the disposal stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateDisposeCapability;

impl CreateDisposeCapability {
    #[inline(always)]
    pub(super) fn operation((): (), context: &mut Context) {
        context
            .vm
            .frame_mut()
            .dispose_stack
            .push(DisposeEntry::Marker);
    }
}

impl Operation for CreateDisposeCapability {
    const NAME: &'static str = "CreateDisposeCapability";
    const INSTRUCTION: &'static str = "INST - CreateDisposeCapability";
    const COST: u8 = 1;
}

/// `AddDisposableResource`
///
/// Takes a value from a register, gets its `Symbol.dispose` method,
/// and pushes the resource onto the disposal stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AddDisposableResource;

impl AddDisposableResource {
    #[inline(always)]
    pub(super) fn operation(value: RegisterOperand, context: &mut Context) -> JsResult<()> {
        let value = context.vm.get_register(value.into()).clone();

        // If value is null or undefined, add a no-op resource (spec step).
        if value.is_null_or_undefined() {
            context
                .vm
                .frame_mut()
                .dispose_stack
                .push(DisposeEntry::Resource(DisposableResource {
                    value: JsValue::undefined(),
                    method: JsValue::undefined(),
                }));
            return Ok(());
        }

        // Ensure value is an object.
        if !value.is_object() {
            return Err(JsNativeError::typ()
                .with_message("using declaration requires an object or null/undefined")
                .into());
        }

        // Get @@dispose method.
        let method = value.get_method(JsSymbol::dispose(), context)?;

        match method {
            Some(m) => {
                let method_value: JsValue = m.into();
                context
                    .vm
                    .frame_mut()
                    .dispose_stack
                    .push(DisposeEntry::Resource(DisposableResource {
                        value,
                        method: method_value,
                    }));
                Ok(())
            }
            None => Err(JsNativeError::typ()
                .with_message("value does not have a [Symbol.dispose] method")
                .into()),
        }
    }
}

impl Operation for AddDisposableResource {
    const NAME: &'static str = "AddDisposableResource";
    const INSTRUCTION: &'static str = "INST - AddDisposableResource";
    const COST: u8 = 4;
}

/// `DisposeResources`
///
/// Pops and disposes resources back to the last scope marker.
/// Implements the `DisposeResources` abstract operation (sync).
#[derive(Debug, Clone, Copy)]
pub(crate) struct DisposeResources;

impl DisposeResources {
    #[inline(always)]
    pub(super) fn operation((): (), context: &mut Context) -> JsResult<()> {
        Self::dispose_sync(context)
    }

    /// Core sync disposal logic.
    pub(crate) fn dispose_sync(context: &mut Context) -> JsResult<()> {
        // Collect resources back to the last marker.
        let mut resources = Vec::new();
        loop {
            let entry = context.vm.frame_mut().dispose_stack.pop();
            match entry.as_ref() {
                Some(DisposeEntry::Resource(resource)) => {
                    resources.push(resource.clone());
                }
                Some(DisposeEntry::Marker) | None => break,
            }
        }

        // Per spec: DisposeResources(disposeCapability, completion)
        // The initial completion is the pending exception (if any) from the body.
        let had_pending = context.vm.pending_exception.is_some();
        let mut completion: Result<(), crate::JsError> =
            if let Some(pending) = context.vm.pending_exception.take() {
                Err(pending)
            } else {
                Ok(())
            };

        // Resources are already in reverse order (LIFO from the stack).
        // Dispose each one, collecting errors.
        for resource in &resources {
            if resource.method.is_undefined() {
                continue;
            }

            let result = resource.method.as_callable().map_or_else(
                || {
                    Err(JsNativeError::typ()
                        .with_message("dispose method is not callable")
                        .into())
                },
                |method| method.call(&resource.value, &[], context).map(|_| ()),
            );

            if let Err(err) = result {
                completion = match completion {
                    Ok(()) => Err(err),
                    Err(prev_err) => {
                        // Create SuppressedError: new error suppresses previous one.
                        let suppressed_error = crate::object::JsObject::from_proto_and_data(
                            context
                                .intrinsics()
                                .constructors()
                                .suppressed_error()
                                .prototype(),
                            OrdinaryObject,
                        );
                        let error_val = err.into_opaque(context).unwrap_or(JsValue::undefined());
                        let suppressed_val = prev_err
                            .into_opaque(context)
                            .unwrap_or(JsValue::undefined());
                        suppressed_error.create_non_enumerable_data_property_or_throw(
                            js_string!("error"),
                            error_val,
                            context,
                        );
                        suppressed_error.create_non_enumerable_data_property_or_throw(
                            js_string!("suppressed"),
                            suppressed_val,
                            context,
                        );
                        suppressed_error.create_non_enumerable_data_property_or_throw(
                            js_string!("message"),
                            js_string!(""),
                            context,
                        );
                        Err(crate::JsError::from_opaque(suppressed_error.into()))
                    }
                };
            }
        }

        // If we were in the exception handler path (had pending exception),
        // set the combined error back as pending exception for ReThrow.
        // Otherwise (normal path), return errors via `?` propagation.
        if had_pending {
            if let Err(err) = completion {
                context.vm.pending_exception = Some(err);
            }
            Ok(())
        } else {
            completion
        }
    }
}

impl Operation for DisposeResources {
    const NAME: &'static str = "DisposeResources";
    const INSTRUCTION: &'static str = "INST - DisposeResources";
    const COST: u8 = 8;
}
