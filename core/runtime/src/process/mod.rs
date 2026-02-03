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
    Context, JsObject, JsResult, JsString, JsValue, js_string, property::Attribute,
    property::PropertyDescriptor,
};
use boa_gc::{Finalize, Trace};

/// Boa's implementation of Node.js' `process` object.    
#[derive(Debug, Trace, Finalize)]
pub struct Process;

impl Process {
    /// Name of the built-in `process` property.    
    pub const NAME: JsString = js_string!("process");

    /// Initializes the `process` built-in object.    
    ///  
    /// # Errors  
    ///  
    /// Returns a `JsError` if:  
    /// - Setting environment variables on the `env` object fails  
    /// - Defining the `env` property on the `process` object fails  
    pub fn init(context: &mut Context) -> JsResult<JsObject> {
        let process = JsObject::default(context.intrinsics());

        // Create env object with current environment variables
        let env = JsObject::default(context.intrinsics());
        for (key, value) in std::env::vars() {
            env.set(
                js_string!(key),
                JsValue::from(js_string!(value)),
                false,
                context,
            )?;
        }

        process.define_property_or_throw(
            js_string!("env"),
            PropertyDescriptor::builder()
                .value(env)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;

        Ok(process)
    }

    /// Register the `process` object globally.    
    ///    
    /// # Errors    
    /// This function will return an error if the property cannot be defined on the global object.    
    pub fn register(context: &mut Context) -> JsResult<()> {
        let process_object = Self::init(context)?;

        context.register_global_property(
            js_string!(Self::NAME),
            process_object,
            Attribute::CONFIGURABLE,
        )?;

        Ok(())
    }
}
