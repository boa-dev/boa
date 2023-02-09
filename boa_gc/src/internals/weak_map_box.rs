use crate::{GcRefCell, Trace, WeakGc};
use std::{cell::Cell, collections::HashMap, ptr::NonNull};

/// A box that is used to track [`WeakMap`][`crate::WeakMap`]s.
pub(crate) struct WeakMapBox<K: Trace + Sized + 'static, V: Trace + Sized + 'static> {
    pub(crate) map: WeakGc<GcRefCell<HashMap<WeakGc<K>, V>>>,
    pub(crate) next: Cell<Option<NonNull<dyn ErasedWeakMapBox>>>,
}

/// A trait that is used to erase the type of a [`WeakMapBox`].
pub(crate) trait ErasedWeakMapBox {
    /// Clear dead entries from the [`WeakMapBox`].
    fn clear_dead_entires(&self);

    /// A pointer to the next [`WeakMapBox`].
    fn next(&self) -> &Cell<Option<NonNull<dyn ErasedWeakMapBox>>>;

    /// Returns `true` if the [`WeakMapBox`] is live.
    fn is_live(&self) -> bool;

    /// Traces the weak reference inside of the [`WeakMapBox`] it the weak map is live.
    unsafe fn trace(&self);
}

impl<K: Trace, V: Trace> ErasedWeakMapBox for WeakMapBox<K, V> {
    fn clear_dead_entires(&self) {
        if let Some(map) = self.map.upgrade() {
            let mut map = map.borrow_mut();
            map.retain(|k, _| k.upgrade().is_some());
        }
    }

    fn next(&self) -> &Cell<Option<NonNull<dyn ErasedWeakMapBox>>> {
        &self.next
    }

    fn is_live(&self) -> bool {
        self.map.upgrade().is_some()
    }

    unsafe fn trace(&self) {
        if self.map.upgrade().is_some() {
            // SAFETY: When the weak map is live, the weak reference should be traced.
            unsafe { self.map.trace() }
        }
    }
}
