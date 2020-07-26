use std::borrow::Borrow;
use std::fmt::{self, Display};
use std::ops::Deref;
use std::rc::Rc;

use gc::{unsafe_empty_trace, Finalize, Trace};

#[derive(Debug, Finalize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcString(Rc<str>);

unsafe impl Trace for RcString {
    unsafe_empty_trace!();
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
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<RcString> for str {
    fn eq(&self, other: &RcString) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<&str> for RcString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<RcString> for &str {
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
