use crate::{Ephemeron, Finalize, Gc, Trace};

#[derive(Trace, Finalize)]
#[repr(transparent)]
pub struct WeakGc<T: Trace + ?Sized + 'static> {
    inner: Ephemeron<T, ()>,
}

impl<T: Trace + ?Sized> WeakGc<T> {
    pub fn new(value: &Gc<T>) -> Self {
        Self {
            inner: Ephemeron::new(value, ()),
        }
    }
}

impl<T: Trace + ?Sized> WeakGc<T> {
    #[inline]
    pub fn value(&self) -> Option<&T> {
        self.inner.key()
    }
}

impl<T: Trace + ?Sized> From<Ephemeron<T, ()>> for WeakGc<T> {
    fn from(inner: Ephemeron<T, ()>) -> Self {
        Self { inner }
    }
}
