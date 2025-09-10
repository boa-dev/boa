use boa_engine::{Context, Finalize, JsResult, Trace};
use boa_runtime::{ConsoleState, Logger};
use rustyline::ExternalPrinter;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

#[derive(Clone, Trace, Finalize)]
pub(crate) struct SharedExternalPrinterLogger {
    #[unsafe_ignore_trace]
    inner: Arc<Mutex<Option<Box<dyn ExternalPrinter + Send>>>>,
}

impl SharedExternalPrinterLogger {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) fn set<T: ExternalPrinter + Send + 'static>(&self, inner: T) {
        self.inner
            .lock()
            .expect("printer lock failed")
            .replace(Box::new(inner));
    }

    pub(crate) fn print(&self, message: String) {
        if let Some(l) = &mut *self.inner.lock().expect("printer lock failed") {
            // Ignore errors, there's nothing we can do at this point.
            drop(l.print(message));
        }
    }
}

impl Debug for SharedExternalPrinterLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedExternalPrinterLogger").finish()
    }
}

impl Logger for SharedExternalPrinterLogger {
    #[inline]
    fn log(&self, msg: String, state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
        let indent = state.indent();
        self.print(format!("{msg:>indent$}\n"));
        Ok(())
    }

    #[inline]
    fn info(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }

    #[inline]
    fn warn(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }

    #[inline]
    fn error(&self, msg: String, state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
        let indent = state.indent();
        self.print(format!("{msg:>indent$}\n"));
        Ok(())
    }
}
