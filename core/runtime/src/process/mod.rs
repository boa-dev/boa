//! Boa's implementation of Node.js' `process` object.
//!
//! The `process` object can be accessed from any global object.
//!
//! More information:
//!  - [Node.js documentation][node]
//!
//! [node]: https://nodejs.org/api/process.html

#[cfg(test)]
pub(crate) mod tests;

use boa_engine::{
    Context, JsData, JsObject, JsResult, JsString, JsSymbol, JsValue, js_error, js_string,
    native_function::NativeFunction, object::ObjectInitializer, property::Attribute,
};
use boa_gc::{Finalize, Trace};
use std::rc::Rc;

/// A trait that can be used to forward process provider to an implementation.
pub trait ProcessProvider: Trace {
    /// Get current working directory (`process.cwd()`)
    ///
    /// # Errors
    /// Returns an error if the current directory cannot be obtained.
    fn cwd(&self) -> JsResult<JsString>;

    /// Get environment variables so as to allow env property (`process.env`)
    fn env(&self) -> impl IntoIterator<Item = (JsString, JsString)>;
}

/// The default std implementation of the process provider.
///
/// Implements the [`ProcessProvider`] trait. Outputs the process properties'
/// values on the basis of std.
#[derive(Debug, Trace, Finalize)]
pub struct StdProcessProvider;

impl ProcessProvider for StdProcessProvider {
    fn cwd(&self) -> JsResult<JsString> {
        let path = std::env::current_dir().map_err(
            |e| js_error!(TypeError: "failed to get current working directory: {}", e.to_string()),
        )?;
        Ok(js_string!(path.to_string_lossy()))
    }

    fn env(&self) -> impl IntoIterator<Item = (JsString, JsString)> {
        std::env::vars().map(|(k, v)| (js_string!(k), js_string!(v)))
    }
}

/// Boa's implementation of Node.js' `process` object.
#[derive(Debug, Trace, Finalize, JsData)]
pub struct Process;

impl Process {
    /// Name of the built-in `process` property.
    pub const NAME: JsString = js_string!("process");

    /// Initializes the `process` built-in object with a custom provider.
    ///
    /// # Errors
    ///
    /// Returns a `JsError` if:
    /// - Custom process provider returns an error
    /// - Defining the `cwd` and `env` properties on the `process` object fails
    pub fn init_with_provider<P>(context: &Context, provider: P) -> JsResult<JsObject>
    where
        P: ProcessProvider + 'static,
    {
        fn process_method<P: ProcessProvider + 'static>(
            f: fn(&JsValue, &[JsValue], &P, &Context) -> JsResult<JsValue>,
            provider: Rc<P>,
        ) -> NativeFunction {
            // SAFETY: `Process` doesn't contain types that need tracing.
            unsafe {
                NativeFunction::from_closure(move |this, args, context| {
                    f(this, args, &provider, context)
                })
            }
        }

        let provider = Rc::new(provider);

        let env = JsObject::default(context.intrinsics());
        for (key, value) in provider.env() {
            env.set(key, JsValue::from(value), false, context)?;
        }

        Ok(ObjectInitializer::new(context)
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("env"),
                env,
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .function(
                process_method(
                    |_, _, provider, _| provider.cwd().map(JsValue::from),
                    provider.clone(),
                ),
                js_string!("cwd"),
                0,
            )
            .build())
    }

    /// Register the `process` object globally by a custom provider.
    ///
    /// # Errors
    /// This function will return an error if the property cannot be defined on the global object.
    pub fn register_with_provider<P>(context: &Context, provider: P) -> JsResult<()>
    where
        P: ProcessProvider + 'static,
    {
        let process_object = Self::init_with_provider(context, provider)?;

        context.register_global_property(
            js_string!(Self::NAME),
            process_object,
            Attribute::CONFIGURABLE,
        )?;

        Ok(())
    }

    /// Initializes the `process` built-in object with the default std provider.
    ///
    /// # Errors
    ///
    /// Returns a `JsError` if:
    /// - Defining the `cwd` and `env` properties on the `process` object fails
    pub fn init(context: &Context) -> JsResult<JsObject> {
        Self::init_with_provider(context, StdProcessProvider)
    }

    /// Register the `process` object globally by the default std provider.
    ///
    /// # Errors
    /// This function will return an error if the property cannot be defined on the global object.
    pub fn register(context: &Context) -> JsResult<()> {
        Self::register_with_provider(context, StdProcessProvider)
    }
}
