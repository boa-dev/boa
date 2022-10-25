//! This module implements a wrapper for the `Generator` Builtin JavaScript Object
use crate::{
    builtins::generator::{Generator, GeneratorState},
    object::{JsObject, JsObjectType, ObjectData},
    Context, JsNativeError, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsGenerator` provides a wrapper for Boa's implementation of the JavaScript `Generator` builtin object
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsGenerator {
    inner: JsObject,
}

impl JsGenerator {
    /// Create a new `JsGenerator` object
    #[inline]
    pub fn new(context: &mut Context) -> Self {
        let prototype = context.intrinsics().constructors().generator().prototype();

        let generator = JsObject::from_proto_and_data(
            prototype,
            ObjectData::generator(Generator {
                state: GeneratorState::Undefined,
                context: None,
            }),
        );

        Self { inner: generator }
    }

    /// Create a `JsGenerator` from a regular expression `JsObject`
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.borrow().is_generator() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not a Generator")
                .into())
        }
    }

    /// Calls `Generator.prototype.next()`
    ///
    /// This method returns an object with the properties `done` and `value`
    #[inline]
    pub fn next<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Generator::next(&self.inner.clone().into(), &[value.into()], context)
    }

    /// Calls `Generator.prototype.return()`
    ///
    /// This method returns the given value and finishes the generator
    #[inline]
    pub fn r#return<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Generator::r#return(&self.inner.clone().into(), &[value.into()], context)
    }

    /// Calls `Generator.prototype.throw()`
    ///
    /// This method resumes the execution of a generator by throwing an error and returning an
    /// an object with the properties `done` and `value`
    #[inline]
    pub fn throw<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Generator::throw(&self.inner.clone().into(), &[value.into()], context)
    }
}

impl From<JsGenerator> for JsObject {
    #[inline]
    fn from(o: JsGenerator) -> Self {
        o.inner.clone()
    }
}

impl From<JsGenerator> for JsValue {
    #[inline]
    fn from(o: JsGenerator) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsGenerator {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsGenerator {}
