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
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
    NativeFunction, js_string,
    object::{
        ObjectInitializer,
        builtins::{JsArray, JsProxy, JsProxyBuilder},
    },
    property::Attribute,
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

    /// Change current working directory (`process.chdir(directory)`).
    ///
    /// # Errors
    /// Returns an error if the current directory cannot be changed.
    fn chdir(&self, dir: &JsString) -> JsResult<()>;

    /// Terminate the current process (`process.exit([code])`).
    ///
    /// # Errors
    /// Returns an error if the process cannot be terminated.
    fn exit(&self, code: i32) -> JsResult<()>;

    /// Get environment variables so as to allow env property (`process.env`)
    fn env(&self) -> impl IntoIterator<Item = (JsString, JsString)> {
        std::env::vars().map(|(k, v)| (js_string!(k), js_string!(v)))
    }

    /// Get the operating system platform (`process.platform`)
    fn platform(&self) -> JsString {
        let platform = match std::env::consts::OS {
            "windows" => "win32",
            "macos" => "darwin",
            "linux" => "linux",
            "freebsd" => "freebsd",
            "openbsd" => "openbsd",
            "android" => "android",
            "solaris" => "sunos",
            "aix" => "aix",
            other => other,
        };
        js_string!(platform)
    }

    /// Get the CPU architecture (`process.arch`)
    fn arch(&self) -> JsString {
        let arch = match std::env::consts::ARCH {
            "x86_64" => "x64",
            "x86" => "ia32",
            "aarch64" => "arm64",
            "arm" => "arm",
            "riscv64" => "riscv64",
            "powerpc64" => "ppc64",
            "s390x" => "s390x",
            "mips" => "mips",
            "mips64" => "mipsel",
            other => other,
        };
        js_string!(arch)
    }

    /// Get command-line arguments (`process.argv`)
    fn argv(&self) -> impl IntoIterator<Item = JsString> {
        std::env::args().map(|arg| js_string!(arg))
    }

    /// Set an environment variable on the host.
    ///
    /// # Errors
    /// Returns an error if the environment variable cannot be set.
    fn set_env(&self, key: &JsString, value: &JsString) -> JsResult<()> {
        // SAFETY: Single-threaded JS context execution in default StdProcessProvider
        unsafe {
            std::env::set_var(key.to_std_string_escaped(), value.to_std_string_escaped());
        }
        Ok(())
    }

    /// Remove an environment variable from the host.
    ///
    /// # Errors
    /// Returns an error if the environment variable cannot be removed.
    fn remove_env(&self, key: &JsString) -> JsResult<()> {
        // SAFETY: Single-threaded JS context execution in default StdProcessProvider
        unsafe {
            std::env::remove_var(key.to_std_string_escaped());
        }
        Ok(())
    }
}

/// The default std implementation of the process provider.
///
/// Implements the [`ProcessProvider`] trait. Outputs the process properties'
/// values on the basis of std.
#[derive(Debug, Trace, Finalize)]
pub struct StdProcessProvider;

impl ProcessProvider for StdProcessProvider {
    fn cwd(&self) -> JsResult<JsString> {
        let path = std::env::current_dir().map_err(|e| {
            JsNativeError::typ()
                .with_message(format!("Failed to get current working directory: {e}"))
        })?;
        Ok(js_string!(path.to_string_lossy()))
    }

    fn chdir(&self, dir: &JsString) -> JsResult<()> {
        let path = dir.to_std_string_escaped();
        std::env::set_current_dir(&path).map_err(|e| {
            JsNativeError::typ()
                .with_message(format!("Failed to change directory to '{path}': {e}"))
        })?;
        Ok(())
    }

    fn exit(&self, code: i32) -> JsResult<()> {
        std::process::exit(code);
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

    /// Builds the `Proxy`-wrapped `process.env` object.
    fn create_env_proxy<P: ProcessProvider + 'static>(
        context: &mut Context,
        provider: &P,
    ) -> JsResult<JsProxy> {
        fn set_trap(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
            let target = args.get_or_undefined(0);
            let prop = args.get_or_undefined(1);
            let value = args.get_or_undefined(2);

            if let Some(obj) = target.as_object() {
                obj.set(
                    prop.to_property_key(context)?,
                    value.clone(),
                    false,
                    context,
                )?;
            }
            Ok(JsValue::from(true))
        }

        fn delete_trap(
            _this: &JsValue,
            args: &[JsValue],
            context: &mut Context,
        ) -> JsResult<JsValue> {
            let target = args.get_or_undefined(0);
            let prop = args.get_or_undefined(1);

            if let Some(obj) = target.as_object() {
                let deleted =
                    obj.delete_property_or_throw(prop.to_property_key(context)?, context)?;
                return Ok(JsValue::from(deleted));
            }
            Ok(JsValue::from(true))
        }

        let env = JsObject::default(context.intrinsics());
        for (key, value) in provider.env() {
            env.set(key, JsValue::from(value), false, context)?;
        }

        JsProxyBuilder::new(env)
            .set(set_trap)
            .delete_property(delete_trap)
            .build(context)
    }

    /// Initializes the `process` built-in object with a custom provider.
    ///
    /// # Errors
    ///
    /// Returns a `JsError` if:
    /// - Custom process provider returns an error
    /// - Defining the `cwd` and `env` properties on the `process` object fails
    pub fn init_with_provider<P>(context: &mut Context, provider: P) -> JsResult<JsObject>
    where
        P: ProcessProvider + 'static,
    {
        fn process_method<P: ProcessProvider + 'static>(
            f: fn(&JsValue, &[JsValue], &P, &mut Context) -> JsResult<JsValue>,
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
        let env_proxy = Self::create_env_proxy(context, provider.as_ref())?;

        // Build process.argv as a real JS array from the provider's argv iterator.
        let argv_values: Vec<JsValue> = provider.argv().into_iter().map(JsValue::from).collect();
        let argv_array = JsArray::from_iter(argv_values, context);

        Ok(ObjectInitializer::new(context)
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("env"),
                env_proxy,
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("platform"),
                provider.platform(),
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("arch"),
                provider.arch(),
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("argv"),
                argv_array,
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .function(
                process_method(
                    |_, _, provider, _| provider.cwd().map(JsValue::from),
                    provider.clone(),
                ),
                js_string!("cwd"),
                0,
            )
            .function(
                process_method(
                    |_, args, provider, context| {
                        let dir = args.get_or_undefined(0).to_string(context)?;
                        provider.chdir(&dir)?;
                        Ok(JsValue::undefined())
                    },
                    provider.clone(),
                ),
                js_string!("chdir"),
                1,
            )
            .function(
                process_method(
                    |_, args, provider, context| {
                        let code_arg = args.get_or_undefined(0);
                        let code = if code_arg.is_undefined() {
                            0
                        } else {
                            code_arg.to_i32(context)?
                        };
                        provider.exit(code)?;
                        Ok(JsValue::undefined())
                    },
                    provider.clone(),
                ),
                js_string!("exit"),
                1,
            )
            .build())
    }

    /// Register the `process` object globally by a custom provider.
    ///
    /// # Errors
    /// This function will return an error if the property cannot be defined on the global object.
    pub fn register_with_provider<P>(context: &mut Context, provider: P) -> JsResult<()>
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
    pub fn init(context: &mut Context) -> JsResult<JsObject> {
        Self::init_with_provider(context, StdProcessProvider)
    }

    /// Register the `process` object globally by the default std provider.
    ///
    /// # Errors
    /// This function will return an error if the property cannot be defined on the global object.
    pub fn register(context: &mut Context) -> JsResult<()> {
        Self::register_with_provider(context, StdProcessProvider)
    }
}
