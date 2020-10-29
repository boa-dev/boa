use crate::{
    builtins::BigInt,
    gc::{empty_trace, Finalize, Trace},
};

use std::{
    fmt::{self, Display},
    ops::Deref,
    rc::Rc,
};

#[derive(Debug, Finalize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcBigInt(Rc<BigInt>);

unsafe impl Trace for RcBigInt {
    empty_trace!();
}

impl RcBigInt {
    pub(crate) fn as_inner(&self) -> &BigInt {
        &self.0
    }
}

impl Display for RcBigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for RcBigInt {
    type Target = BigInt;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BigInt> for RcBigInt {
    #[inline]
    fn from(bigint: BigInt) -> Self {
        Self(Rc::from(bigint))
    }
}
