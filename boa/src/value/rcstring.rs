use crate::gc::{empty_trace, Finalize, Trace};

use std::{
    borrow::Borrow,
    fmt::{self, Display},
    ops::Deref,
    rc::Rc,
};

#[derive(Debug, Finalize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcString(Rc<str>);

unsafe impl Trace for RcString {
    empty_trace!();
}

impl RcString {
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RcString {
    #[inline]
    fn default() -> Self {
        Self(Rc::from(String::new()))
    }
}

impl Display for RcString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl PartialEq<str> for RcString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<RcString> for str {
    #[inline]
    fn eq(&self, other: &RcString) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<&str> for RcString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<RcString> for &str {
    #[inline]
    fn eq(&self, other: &RcString) -> bool {
        *self == other.as_str()
    }
}

impl Deref for RcString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<str> for RcString {
    #[inline]
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl AsRef<str> for RcString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<String> for RcString {
    #[inline]
    fn from(string: String) -> Self {
        Self(Rc::from(string))
    }
}

impl From<&RcString> for String {
    #[inline]
    fn from(string: &RcString) -> Self {
        string.to_string()
    }
}

impl From<Box<str>> for RcString {
    #[inline]
    fn from(string: Box<str>) -> Self {
        Self(Rc::from(string))
    }
}

impl From<&str> for RcString {
    #[inline]
    fn from(string: &str) -> Self {
        Self(Rc::from(string))
    }
}
