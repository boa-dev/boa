use crate::scope::SCOPE_STACK;
use crate::{Finalize, Trace, Tracer};

pub(crate) struct HandleScopeTracker;

impl Finalize for HandleScopeTracker {
    fn finalize(&self) {}
}

unsafe impl Trace for HandleScopeTracker {
    unsafe fn trace(&self, tracer: &mut Tracer<'_>) {
        SCOPE_STACK.with(|stack| {
            for scope in stack.borrow().iter() {
                for root in scope.iter() {
                    unsafe {
                        root.trace(tracer);
                    }
                }
            }
        });
    }
}
