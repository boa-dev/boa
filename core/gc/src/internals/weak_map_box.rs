use crate::{pointers::RawWeakMap, GcRefCell, Trace, Tracer, WeakGc};

/// A box that is used to track [`WeakMap`][`crate::WeakMap`]s.
pub(crate) struct WeakMapBox<K: Trace + ?Sized + 'static, V: Trace + Sized + 'static> {
    pub(crate) map: WeakGc<GcRefCell<RawWeakMap<K, V>>>,
}

/// A trait that is used to erase the type of a [`WeakMapBox`].
pub(crate) trait ErasedWeakMapBox {
    /// Clear dead entries from the [`WeakMapBox`].
    fn clear_dead_entries(&self);

    /// Returns `true` if the [`WeakMapBox`] is live.
    fn is_live(&self) -> bool;

    /// Traces the weak reference inside of the [`WeakMapBox`] if the weak map is live.
    unsafe fn trace(&self, tracer: &mut Tracer);
}

impl<K: Trace + ?Sized, V: Trace + Clone> ErasedWeakMapBox for WeakMapBox<K, V> {
    fn clear_dead_entries(&self) {
        if let Some(map) = self.map.upgrade() {
            if let Ok(mut map) = map.try_borrow_mut() {
                map.clear_expired();
            }
        }
    }

    fn is_live(&self) -> bool {
        self.map.upgrade().is_some()
    }

    unsafe fn trace(&self, tracer: &mut Tracer) {
        if self.map.upgrade().is_some() {
            // SAFETY: When the weak map is live, the weak reference should be traced.
            unsafe { self.map.trace(tracer) }
        }
    }
}
