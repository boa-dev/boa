use std::{ops::Deref, rc::Rc};

/// A [`Cow`][std::borrow::Cow]-like pointer where the `Owned` variant is an [`Rc`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaybeShared<'a, T: ?Sized> {
    /// Borrowed data.
    Borrowed(&'a T),
    /// `Rc` shared data.
    Shared(Rc<T>),
}

impl<T: ?Sized> Clone for MaybeShared<'_, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrowed(b) => Self::Borrowed(b),
            Self::Shared(sh) => Self::Shared(sh.clone()),
        }
    }
}

impl<T: ?Sized> Deref for MaybeShared<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeShared::Borrowed(b) => b,
            MaybeShared::Shared(sh) => sh,
        }
    }
}

impl<'a, T: ?Sized> From<&'a T> for MaybeShared<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::Borrowed(value)
    }
}

impl<T: ?Sized> From<Rc<T>> for MaybeShared<'static, T> {
    fn from(value: Rc<T>) -> Self {
        Self::Shared(value)
    }
}
