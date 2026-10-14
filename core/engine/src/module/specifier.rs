//! A [`Specifier`] is a typed string that correspond to a module's name in the
//! module loader. It is used to identify a module in the module loader.
//!
//! Specifiers are any valid JavaScript string literal. In Rust, we keep them
//! as UTF-8 strings for performance reasons, and translate them to JavaScript
//! strings when needed.
//!
//! All modules live inside a single specifier namespace. All specifiers are
//! unique to the module they represent. This means that two different modules
//! cannot have the same specifier.
//!
//! Specifiers are used to identify modules in the module loader. They are
//! passed to the module loader to load a module, and are used to store modules
//! in the module loader. When creating a Source, the specifier should
//! optionally be passed as the name of the module.
//!
//! Specifiers are Unix-like; they use forward slashes as separators, and can
//! be relative if they start with a `.` or `..`.

use std::fmt::Display;
use std::path::PathBuf;
use std::{fmt, path};

use crate::value::TryFromJs;
use crate::{Context, JsError, JsNativeError, JsResult, JsString, JsValue};

#[cfg(test)]
mod tests;

/// Specifier component separator.
pub const SEPARATOR: char = '/';

/// Specifier component separator as a string.
pub const SEPARATOR_STR: &str = "/";

/// Returns true if the byte is a specifier separator.
fn is_seperator_byte(b: u8) -> bool {
    b == SEPARATOR as u8
}

/// Internal enum for iterator state.
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
enum State {
    /// Initial value.
    #[default]
    Start,

    /// Normal component.
    Body,

    /// Done iterating.
    Done,
}

/// An iterator over the [`Component`]s of a [`Specifier`].
#[derive(Debug, Clone)]
pub struct Components<'a> {
    // This inner has been verified to be a valid `str` at some point.
    inner: &'a [u8],

    // The state of the front iterator.
    front: State,

    // The state of the back iterator.
    back: State,

    // Whether the components are absolute.
    absolute: bool,
}

impl<'a> Components<'a> {
    /// Create a new components iterator.
    #[inline]
    #[must_use]
    pub fn new(inner: &'a [u8]) -> Self {
        Self {
            inner,
            front: State::Start,
            back: State::Start,
            absolute: !(inner.starts_with(b"./") || inner.starts_with(b"../")),
        }
    }

    /// Return the specifier corresponding to the rest of the components
    /// available in the iterator.
    ///
    /// # Examples
    /// ```
    /// use boa_engine::module::Specifier;
    ///
    /// let spec = Specifier::from_str("/foo/bar/baz");
    /// let mut components = spec.components();
    /// components.next_back();
    /// let rest = components.as_specifier();
    /// assert_eq!(rest.as_str(), "/foo/bar");
    /// ```
    #[must_use]
    pub fn as_specifier(&self) -> &'a Specifier {
        let mut comps = self.clone();
        if comps.front == State::Body {
            comps.trim_left();
        }
        if comps.back == State::Body {
            comps.trim_right();
        }

        // SAFETY: we already verified that inner is a valid string.
        unsafe { Specifier::new_unchecked(std::str::from_utf8_unchecked(comps.inner)) }
    }

    /// Remove empty and duplicated separators from the left of the components.
    #[inline]
    fn trim_left(&mut self) {
        while !self.inner.is_empty() {
            let (size, comp) = self.next_front_component();
            if comp.is_some() {
                return;
            }
            self.inner = &self.inner[size..];
        }
    }

    /// Remove empty and duplicated separators from the left of the components.
    #[inline]
    fn trim_right(&mut self) {
        while !self.inner.is_empty() {
            let (size, comp) = self.next_back_component();
            if comp.is_some() {
                return;
            }
            self.inner = &self.inner[..self.inner.len() - size];
        }
    }

    /// Whether we finished iterating.
    #[inline]
    fn finished(&self) -> bool {
        self.front == State::Done || self.back == State::Done
    }

    /// Parse a single component from the inner string.
    ///
    /// # Safety
    /// This is safe to call, as the input is already verified to be a valid string
    /// when constructing [`Components`]. The string cannot be invalidated later on
    /// as it is split on separators which are guaranteed to be valid.
    unsafe fn parse_single_component(comp: &'a [u8]) -> Option<Component<'a>> {
        match comp {
            b"" => None,
            b"." => Some(Component::Current),
            b".." => Some(Component::Parent),
            _ => {
                // SAFETY: we already verified that comp is a valid string.
                Some(Component::Normal(unsafe {
                    std::str::from_utf8_unchecked(comp)
                }))
            }
        }
    }

    /// Parse the next component from inner.
    #[inline]
    fn next_front_component(&self) -> (usize, Option<Component<'a>>) {
        let (extra, comp) = match self.inner.iter().position(|c| is_seperator_byte(*c)) {
            None => (0, self.inner),
            Some(i) => (1, &self.inner[..i]),
        };

        // SAFETY: we already verified that inner is a valid string, since it was already split
        // on the separator.
        (comp.len() + extra, unsafe {
            Self::parse_single_component(comp)
        })
    }

    /// Parse the next back component from inner.
    #[inline]
    fn next_back_component(&self) -> (usize, Option<Component<'a>>) {
        let (extra, comp) = match self.inner.iter().rposition(|c| is_seperator_byte(*c)) {
            None => (0, self.inner),
            Some(i) => (1, &self.inner[i + 1..]),
        };

        // SAFETY: we already verified that inner is a valid string, since it was already split
        // on the separator.
        (comp.len() + extra, unsafe {
            Self::parse_single_component(comp)
        })
    }
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.finished() {
            match self.front {
                State::Start => {
                    if self.absolute {
                        self.trim_left();
                        self.front = State::Body;
                        return Some(Component::Root);
                    }

                    let (size, comp) = self.next_front_component();
                    self.front = State::Body;
                    self.inner = &self.inner[size..];
                    if comp.is_some() {
                        return comp;
                    }
                }

                State::Body => {
                    let (size, comp) = self.next_front_component();
                    self.inner = &self.inner[size..];
                    if comp.is_some() {
                        return comp;
                    }

                    if self.inner.is_empty() {
                        self.front = State::Done;
                    }
                }

                // `finished()` already checked that we cannot reach this.
                State::Done => unreachable!(),
            }
        }
        None
    }
}

impl DoubleEndedIterator for Components<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while !self.finished() {
            match self.back {
                State::Start => {
                    let (size, comp) = self.next_back_component();
                    self.back = State::Body;
                    self.inner = &self.inner[..self.inner.len() - size];
                    if comp.is_some() {
                        return comp;
                    }
                    // Else we keep trying.
                }

                State::Body => {
                    let (size, comp) = self.next_back_component();
                    self.inner = &self.inner[..self.inner.len() - size];
                    return if comp.is_none() && self.front != State::Done && self.absolute {
                        self.back = State::Done;
                        Some(Component::Root)
                    } else {
                        comp
                    };
                }

                // `finished()` already checked that we cannot reach this.
                State::Done => unreachable!(),
            }
        }
        None
    }
}

/// A single component of a specifier. Specifiers can be split into components
/// using the `components` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Component<'a> {
    /// A root specifier component. Specifiers are absolute if they start with
    /// this component.
    Root,

    /// A normal, named, specifier component.
    Normal(&'a str),

    /// A parent specifier component. This is used to go up a directory in the
    /// specifier.
    Parent,

    /// A current specifier component. This is used to refer to the current
    /// directory in the specifier.
    Current,
}

impl<'a> Component<'a> {
    /// Extracts the underlying string from the specifier component.
    #[must_use]
    pub fn as_str(self) -> &'a str {
        match self {
            Component::Root => "/",
            Component::Normal(s) => s,
            Component::Parent => "..",
            Component::Current => ".",
        }
    }
}

impl AsRef<Specifier> for Component<'_> {
    fn as_ref(&self) -> &Specifier {
        Specifier::from_str(self.as_str())
    }
}

impl AsRef<str> for Component<'_> {
    fn as_ref(&self) -> &str {
        (*self).as_str()
    }
}

impl<'a> TryFrom<path::Component<'a>> for Component<'a> {
    type Error = &'static str;

    fn try_from(value: path::Component<'a>) -> Result<Self, Self::Error> {
        match value {
            path::Component::Prefix(_) => Err("Invalid component"),
            path::Component::RootDir => Ok(Component::Root),
            path::Component::CurDir => Ok(Component::Current),
            path::Component::ParentDir => Ok(Component::Parent),
            path::Component::Normal(s) => s.to_str().ok_or("Invalid UTF8").map(Component::Normal),
        }
    }
}

/// A slice of a module specifier. Specifiers are UTF8.
///
/// This is a low-level type that is used to refer to a specifier.
#[derive(PartialEq, Eq, Hash)]
pub struct Specifier {
    inner: str,
}

impl Specifier {
    /// Creates a new specifier from a string. This is a private method as it
    /// does not verify the validity of the string.
    ///
    /// # Safety
    /// This is safe to call, as the resulting Specifier is guaranteed to
    /// be a value reference as long as the input string is.
    fn new_unchecked<S: AsRef<str> + ?Sized>(s: &S) -> &Specifier {
        #[allow(trivial_casts)]
        unsafe {
            &*(s.as_ref() as *const str as *const Specifier)
        }
    }

    /// Create a new specifier from a string. This is the trait-less version.
    // We allow this because the trait version has different semantics and lifetimes.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn from_str(s: &str) -> &Self {
        Self::new_unchecked(s)
    }

    /// Returns the specifier as a string.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Return an owned version of this [`Specifier`].
    #[must_use]
    pub fn to_owned(&self) -> OwnedSpecifier {
        OwnedSpecifier::from_string(self.inner.to_string())
    }

    /// Returns an iterator over the components of the specifier.
    #[must_use]
    pub fn components(&self) -> Components<'_> {
        Components::new(self.inner.as_bytes())
    }

    /// Returns the parent specifier of this specifier.
    #[must_use]
    pub fn parent(&self) -> Option<&Specifier> {
        let mut comps = self.components();
        let comp = comps.next_back();
        comp.and_then(move |p| match p {
            Component::Parent => None,
            _ => Some(comps.as_specifier()),
        })
    }
}

impl fmt::Debug for Specifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Specifier").field(&&self.inner).finish()
    }
}

impl<'a> From<&'a str> for &'a Specifier {
    fn from(value: &'a str) -> &'a Specifier {
        Specifier::from_str(value)
    }
}

impl AsRef<str> for Specifier {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl AsRef<Specifier> for &str {
    fn as_ref(&self) -> &Specifier {
        Specifier::from_str(self)
    }
}

impl PartialEq<str> for Specifier {
    fn eq(&self, other: &str) -> bool {
        &self.inner == other
    }
}

/// A module specifier that is owned.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct OwnedSpecifier {
    inner: String,
}

impl OwnedSpecifier {
    /// Create a new empty specifier.
    #[must_use]
    pub fn new() -> Self {
        OwnedSpecifier {
            inner: String::new(),
        }
    }

    /// Try creating an owned specifier from a file path.
    pub fn try_from_path<P: AsRef<path::Path>>(path: P) -> Result<Self, &'static str> {
        path.as_ref()
            .components()
            .map(Component::try_from)
            .collect()
    }

    /// Convert this specifier to a file path.
    #[must_use]
    pub fn to_path_buf(&self) -> PathBuf {
        self.components().map(Component::as_str).collect()
    }

    /// Create a new owned specifier from a string.
    #[must_use]
    pub fn from_string(inner: impl Into<String>) -> Self {
        Specifier::from_str(&inner.into()).components().collect()
    }

    /// Returns this specifier as a reference.
    #[must_use]
    pub fn as_specifier(&self) -> &Specifier {
        Specifier::from_str(self.inner.as_str())
    }

    /// Returns this specifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Converts this specifier to a JavaScript string.
    #[must_use]
    pub fn to_js_string(&self) -> JsString {
        JsString::from(self.inner.as_str())
    }

    /// Returns true if the specifier is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the parent specifier of this specifier.
    #[must_use]
    pub fn parent(&self) -> Option<&Specifier> {
        let mut comps = self.components();
        let comp = comps.next_back();
        comp.and_then(move |p| match p {
            Component::Parent => None,
            _ => Some(comps.as_specifier()),
        })
    }

    /// Returns an iterator over the components of the specifier.
    #[must_use]
    pub fn components(&self) -> Components<'_> {
        Components::new(self.inner.as_bytes())
    }

    /// Push a specifier to the end. Does not normalize the specifier,
    /// ie. if pushing a `"."` string, it will be added as is. Will add
    /// a separator if there isn't one at the end or beginning of the
    /// specifier.
    ///
    /// # Examples
    /// ```
    /// # use boa_engine::module::OwnedSpecifier;
    /// let mut spec = OwnedSpecifier::from_string("/foo/bar");
    /// spec.push("/baz");
    /// assert_eq!(spec.as_specifier(), "/foo/bar/baz");
    ///
    /// spec.push("../qux");
    /// assert_eq!(spec.as_specifier(), "/foo/bar/baz/../qux");
    ///
    /// spec.push("///quux");
    /// assert_eq!(spec.as_specifier(), "/foo/bar/baz/../qux///quux");
    /// ```
    pub fn push<S: AsRef<Specifier>>(&mut self, spec: S) {
        self._push(spec.as_ref());
    }

    // Internal version without any generics.
    fn _push(&mut self, spec: &Specifier) {
        if self.inner.is_empty() {
            self.inner.push_str(spec.as_str());
        } else {
            if !self.inner.ends_with(SEPARATOR) && !spec.as_str().starts_with(SEPARATOR) {
                self.inner.push(SEPARATOR);
            }
            self.inner.push_str(spec.as_str());
        }
    }

    /// Removes the last component from the specifier, returning it.
    ///
    /// # Examples
    /// ```
    /// # use boa_engine::module::OwnedSpecifier;
    /// let mut spec = OwnedSpecifier::from_string("/foo/bar/baz");
    /// assert_eq!(spec.pop(), true);
    /// assert_eq!(spec.as_specifier(), "/foo/bar");
    /// ```
    pub fn pop(&mut self) -> bool {
        match self.parent().map(|p| p.inner.len()) {
            None => false,
            Some(l) => {
                self.inner.truncate(l);
                true
            }
        }
    }

    /// Normalize the specifier, removing any current directory notations,
    /// and resolving parents in the middle of the specifier.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::module::OwnedSpecifier;
    /// let spec = OwnedSpecifier::from_string("/foo/bar/../baz");
    /// assert_eq!(spec.normalize().as_specifier(), "/foo/baz");
    /// ```
    ///
    /// Current and Parent components are removed from the middle of the specifier,
    /// but kept at the start.
    /// ```
    /// # use boa_engine::module::OwnedSpecifier;
    /// let spec = OwnedSpecifier::from_string("../../foo/../bar/./baz/.");
    /// assert_eq!(spec.normalize().as_specifier(), "../../bar/baz");
    /// let spec = OwnedSpecifier::from_string("./foo/.././bar/./baz/.");
    /// assert_eq!(spec.normalize().as_specifier(), "./bar/baz");
    /// // It also removes `./` if we go up a parent.
    /// let spec = OwnedSpecifier::from_string("./../bar/./baz/..");
    /// assert_eq!(spec.normalize().as_specifier(), "../bar");
    /// ```
    ///
    /// To remove ambiguity, a separator is always added at the start
    /// of the specifier if it is not relative. For this reason,
    /// normalizing should be done after validating the path is
    /// supposed to be normalized (ie. don't call this on URLs).
    /// ```
    /// # use boa_engine::module::OwnedSpecifier;
    /// let spec = OwnedSpecifier::from_string("abc/def");
    /// assert_eq!(spec.normalize().as_specifier(), "/abc/def");
    /// // Even if the specifier is empty.
    /// let spec = OwnedSpecifier::from_string("");
    /// assert_eq!(spec.normalize().as_specifier(), "/");
    /// ```
    #[must_use]
    pub fn normalize(&self) -> OwnedSpecifier {
        // Whether we're at the body (middle) of the specifier.
        let mut body = false;
        self.components()
            .fold(OwnedSpecifier::new(), move |mut acc, comp| {
                match comp {
                    Component::Parent => {
                        if body {
                            if !acc.pop() {
                                body = false;
                            }
                        } else {
                            if acc.as_str() == "." {
                                acc.pop();
                            }

                            acc.push(Component::Parent);
                        }
                        if acc.as_str() == ".." || acc.as_str() == "." {
                            body = false;
                        }
                    }
                    Component::Current => {
                        if acc.is_empty() {
                            acc.push(Component::Current);
                        }
                    }
                    Component::Normal(_) => {
                        acc.push(comp);
                        body = true;
                    }
                    Component::Root => {
                        acc.push(Component::Root);
                        body = true;
                    }
                }
                acc
            })
    }
}

impl fmt::Debug for OwnedSpecifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("OwnedSpecifier").field(&self.inner).finish()
    }
}

impl Display for OwnedSpecifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.clone())
    }
}

impl Default for OwnedSpecifier {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: AsRef<Specifier>> Extend<P> for OwnedSpecifier {
    #[inline]
    fn extend<T: IntoIterator<Item = P>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |p| self.push(p));
    }
}

impl<'a> FromIterator<Component<'a>> for OwnedSpecifier {
    fn from_iter<T: IntoIterator<Item = Component<'a>>>(iter: T) -> Self {
        let mut owned = Self::new();
        owned.extend(iter);
        owned
    }
}

impl From<String> for OwnedSpecifier {
    fn from(value: String) -> Self {
        OwnedSpecifier { inner: value }
    }
}

impl std::str::FromStr for OwnedSpecifier {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(OwnedSpecifier {
            inner: value.to_owned(),
        })
    }
}

impl TryFromJs for OwnedSpecifier {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let s = value.to_string(context)?;
        Ok(OwnedSpecifier {
            inner: s.to_std_string().map_err(|e| {
                JsError::from_native(JsNativeError::typ().with_message(e.to_string()))
            })?,
        })
    }
}

impl TryFrom<JsString> for OwnedSpecifier {
    type Error = std::string::FromUtf16Error;

    fn try_from(value: JsString) -> Result<Self, Self::Error> {
        Ok(OwnedSpecifier {
            inner: value.to_std_string()?,
        })
    }
}

impl AsRef<Specifier> for OwnedSpecifier {
    fn as_ref(&self) -> &Specifier {
        Specifier::new_unchecked(&self.inner)
    }
}

impl std::borrow::Borrow<Specifier> for OwnedSpecifier {
    fn borrow(&self) -> &Specifier {
        self.as_ref()
    }
}
