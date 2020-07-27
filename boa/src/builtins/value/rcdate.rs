use crate::builtins::Date;

use std::fmt::{self, Display};
use std::ops::Deref;
use std::rc::Rc;

use gc::{unsafe_empty_trace, Finalize, Trace};

#[derive(Debug, Finalize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcDate(Rc<Date>);

unsafe impl Trace for RcDate {
    unsafe_empty_trace!();
}

impl Display for RcDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for RcDate {
    type Target = Date;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Date> for RcDate {
    #[inline]
    fn from(date: Date) -> Self {
        Self(Rc::from(date))
    }
}
