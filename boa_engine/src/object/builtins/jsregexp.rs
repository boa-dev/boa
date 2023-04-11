//! A Rust API wrapper for Boa's `RegExp` Builtin ECMAScript Object
use crate::{
    builtins::RegExp,
    object::{JsArray, JsObject, JsObjectType},
    value::TryFromJs,
    Context, JsNativeError, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsRegExp` provides a wrapper for Boa's implementation of the ECMAScript `RegExp` builtin object
///
/// # Examples
///
/// Create a `JsRegExp` and run RegExp.prototype.test( String )
///
/// ```
/// # use boa_engine::{
/// #  object::builtins::JsRegExp,
/// #  Context, JsValue, JsResult,
/// # };
/// # fn main() -> JsResult<()> {
/// // Initialize the `Context`
/// let context = &mut Context::default();
///
/// // Create a new RegExp with pattern and flags
/// let regexp = JsRegExp::new("foo", "gi", context)?;
///
/// let test_result = regexp.test("football", context)?;
/// assert!(test_result);
///
/// let to_string = regexp.to_string(context)?;
/// assert_eq!(to_string, String::from("/foo/gi"));
/// # Ok(())
/// # }
/// ```
///
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsRegExp {
    inner: JsObject,
}

impl JsRegExp {
    /// Create a new `JsRegExp` object
    /// ```
    /// # use boa_engine::{
    /// #  object::builtins::JsRegExp,
    /// #  Context, JsValue, JsResult,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// // Initialize the `Context`
    /// let context = &mut Context::default();
    ///
    /// // Create a new RegExp with pattern and flags
    /// let regexp = JsRegExp::new("foo", "gi", context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<S>(pattern: S, flags: S, context: &mut Context<'_>) -> JsResult<Self>
    where
        S: Into<JsValue>,
    {
        let constructor = &context
            .intrinsics()
            .constructors()
            .regexp()
            .constructor()
            .into();
        let obj = RegExp::alloc(constructor, context)?;

        let regexp = RegExp::initialize(obj, &pattern.into(), &flags.into(), context)?
            .as_object()
            .expect("RegExp::initialize must return a RegExp object")
            .clone();

        Ok(Self { inner: regexp })
    }

    /// Create a `JsRegExp` from a regular expression `JsObject`
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is_regexp() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not a RegExp")
                .into())
        }
    }

    /// Returns a boolean value for whether the `d` flag is present in `JsRegExp` flags
    #[inline]
    pub fn has_indices(&self, context: &mut Context<'_>) -> JsResult<bool> {
        RegExp::get_has_indices(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    /// Returns a boolean value for whether the `g` flag is present in `JsRegExp` flags
    #[inline]
    pub fn global(&self, context: &mut Context<'_>) -> JsResult<bool> {
        RegExp::get_global(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    /// Returns a boolean value for whether the `i` flag is present in `JsRegExp` flags
    #[inline]
    pub fn ignore_case(&self, context: &mut Context<'_>) -> JsResult<bool> {
        RegExp::get_ignore_case(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    /// Returns a boolean value for whether the `m` flag is present in `JsRegExp` flags
    #[inline]
    pub fn multiline(&self, context: &mut Context<'_>) -> JsResult<bool> {
        RegExp::get_multiline(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    /// Returns a boolean value for whether the `s` flag is present in `JsRegExp` flags
    #[inline]
    pub fn dot_all(&self, context: &mut Context<'_>) -> JsResult<bool> {
        RegExp::get_dot_all(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    /// Returns a boolean value for whether the `u` flag is present in `JsRegExp` flags
    #[inline]
    pub fn unicode(&self, context: &mut Context<'_>) -> JsResult<bool> {
        RegExp::get_unicode(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    /// Returns a boolean value for whether the `y` flag is present in `JsRegExp` flags
    #[inline]
    pub fn sticky(&self, context: &mut Context<'_>) -> JsResult<bool> {
        RegExp::get_sticky(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    /// Returns the flags of `JsRegExp` as a string
    /// ```
    /// # use boa_engine::{
    /// #  object::builtins::JsRegExp,
    /// #  Context, JsValue, JsResult,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # let context = &mut Context::default();
    /// let regexp = JsRegExp::new("foo", "gi", context)?;
    ///
    /// let flags = regexp.flags(context)?;
    /// assert_eq!(flags, String::from("gi"));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn flags(&self, context: &mut Context<'_>) -> JsResult<String> {
        RegExp::get_flags(&self.inner.clone().into(), &[], context).map(|v| {
            v.as_string()
                .expect("value must be string")
                .to_std_string()
                .expect("flags must be a valid string")
        })
    }

    /// Returns the source pattern of `JsRegExp` as a string
    /// ```
    /// # use boa_engine::{
    /// #  object::builtins::JsRegExp,
    /// #  Context, JsValue, JsResult,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # let context = &mut Context::default();
    /// let regexp = JsRegExp::new("foo", "gi", context)?;
    ///
    /// let src = regexp.source(context)?;
    /// assert_eq!(src, String::from("foo"));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn source(&self, context: &mut Context<'_>) -> JsResult<String> {
        RegExp::get_source(&self.inner.clone().into(), &[], context).map(|v| {
            v.as_string()
                .expect("value must be string")
                .to_std_string()
                .expect("source must be a valid string")
        })
    }

    /// Executes a search for a match between `JsRegExp` and the provided string
    /// ```
    /// # use boa_engine::{
    /// #  object::builtins::JsRegExp,
    /// #  Context, JsValue, JsResult,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # let context = &mut Context::default();
    /// let regexp = JsRegExp::new("foo", "gi", context)?;
    ///
    /// let test_result = regexp.test("football", context)?;
    /// assert!(test_result);
    /// # Ok(())
    /// # }
    /// ```
    pub fn test<S>(&self, search_string: S, context: &mut Context<'_>) -> JsResult<bool>
    where
        S: Into<JsValue>,
    {
        RegExp::test(&self.inner.clone().into(), &[search_string.into()], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    /// Executes a search for a match in a specified string
    ///
    /// Returns a `JsArray` containing matched value and updates the `lastIndex` property, or `None`
    pub fn exec<S>(&self, search_string: S, context: &mut Context<'_>) -> JsResult<Option<JsArray>>
    where
        S: Into<JsValue>,
    {
        RegExp::exec(&self.inner.clone().into(), &[search_string.into()], context).map(|v| {
            if v.is_null() {
                None
            } else {
                Some(
                    JsArray::from_object(v.to_object(context).expect("v must be an array"))
                        .expect("from_object must not fail if v is an array object"),
                )
            }
        })
    }

    /// Return a string representing the regular expression.
    /// ```
    /// # use boa_engine::{
    /// #  object::builtins::JsRegExp,
    /// #  Context, JsValue, JsResult,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # let context = &mut Context::default();
    /// let regexp = JsRegExp::new("foo", "gi", context)?;
    ///
    /// let to_string = regexp.to_string(context)?;
    /// assert_eq!(to_string, String::from("/foo/gi"));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn to_string(&self, context: &mut Context<'_>) -> JsResult<String> {
        RegExp::to_string(&self.inner.clone().into(), &[], context).map(|v| {
            v.as_string()
                .expect("value must be a string")
                .to_std_string()
                .expect("to_string value must be a valid string")
        })
    }
}

impl From<JsRegExp> for JsObject {
    #[inline]
    fn from(o: JsRegExp) -> Self {
        o.inner.clone()
    }
}

impl From<JsRegExp> for JsValue {
    #[inline]
    fn from(o: JsRegExp) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsRegExp {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsRegExp {}

impl TryFromJs for JsRegExp {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Object(o) => Self::from_object(o.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("value is not a RegExp object")
                .into()),
        }
    }
}
