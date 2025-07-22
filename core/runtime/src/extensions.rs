//! Module for the register extensions type.

use crate::{DefaultLogger, Logger};
use boa_engine::realm::Realm;
use boa_engine::{Context, JsResult};
use std::fmt::Debug;

/// Optional registrable extension (with arguments) in the Boa Runtime should
/// implement this.
pub trait RuntimeExtension: Debug {
    /// Register this extension in the context using the specified Realm.
    /// This consumes the extension options.
    ///
    /// # Errors
    /// This should error if the extension was not able to register classes, modules or
    /// functions in the context.
    fn register(self, realm: Option<Realm>, context: &mut Context) -> JsResult<()>;
}

/// Register the `Console` JavaScript object with the specified logger.
/// Use [`ConsoleExtension::default()`] to register the console with a default logger.
#[derive(Debug)]
pub struct ConsoleExtension<L: Logger>(pub L);

impl Default for ConsoleExtension<DefaultLogger> {
    fn default() -> Self {
        ConsoleExtension(DefaultLogger)
    }
}

impl<L: Logger + Debug + 'static> RuntimeExtension for ConsoleExtension<L> {
    fn register(self, _realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
        crate::console::Console::register_with_logger(self.0, context)
    }
}

/// Register the `fetch` JavaScript API with the specified [`Fetcher`].
#[cfg(feature = "fetch")]
#[derive(Debug)]
pub struct FetchExtension<F: crate::fetch::Fetcher>(pub F);

#[cfg(feature = "fetch")]
impl<F: crate::fetch::Fetcher + Debug + 'static> RuntimeExtension for FetchExtension<F> {
    fn register(self, realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
        crate::fetch::register(self.0, realm, context)
    }
}

macro_rules! decl_runtime_ext_tuple {
    ($first_name: ident : $first_type: ident) => {
        impl<$first_type: RuntimeExtension> RuntimeExtension for ($first_type,) {
            fn register(self, realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
                RuntimeExtension::register(self.0, realm.clone(), context)?;
                Ok(())
            }
        }
    };
    ($first_name: ident : $first_type: ident, $($name: ident : $type: ident),*) => {
        impl<$first_type: RuntimeExtension, $($type: RuntimeExtension),*> RuntimeExtension for ($first_type, $($type),*) {
            fn register(self, realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
                let ($first_name, $($name),*) = self;
                RuntimeExtension::register($first_name, realm.clone(), context)?;
                $( RuntimeExtension::register($name, realm.clone(), context)?; )*
                Ok(())
            }
        }

        decl_runtime_ext_tuple!($($name: $type),*);
    };
}

decl_runtime_ext_tuple!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K, l: L);
