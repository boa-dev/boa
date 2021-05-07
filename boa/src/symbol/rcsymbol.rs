use crate::{
    gc::{empty_trace, Finalize, Trace},
    symbol::Symbol,
};

use std::{
    fmt::{self, Display},
    ops::Deref,
    rc::Rc,
};

#[derive(Debug, Finalize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcSymbol(Rc<Symbol>);

unsafe impl Trace for RcSymbol {
    empty_trace!();
}

impl Display for RcSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.description() {
            Some(desc) => write!(f, "Symbol({})", desc),
            None => write!(f, "Symbol()"),
        }
    }
}

impl Deref for RcSymbol {
    type Target = Symbol;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Symbol> for RcSymbol {
    #[inline]
    fn from(symbol: Symbol) -> Self {
        Self(Rc::from(symbol))
    }
}
