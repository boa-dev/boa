use boa_engine::{Context, Finalize, JsData, JsResult, Trace};
use boa_gc::Gc;
use boa_runtime::{ConsoleState, Logger};
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A unique index of all logs.
static UNIQUE: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug, Trace, Finalize, JsData)]
pub(crate) struct RecordingLogEvent {
    pub index: usize,
    pub indent: usize,
    pub msg: String,
}

impl RecordingLogEvent {
    pub(crate) fn new(msg: String, state: &ConsoleState) -> Self {
        Self {
            index: UNIQUE.fetch_add(1, Ordering::SeqCst),
            indent: state.indent(),
            msg,
        }
    }
}

#[derive(Default, Trace, Finalize, JsData)]
struct RecordingLoggerInner {
    pub log: Vec<RecordingLogEvent>,
    pub error: Vec<RecordingLogEvent>,
}

#[derive(Clone, Trace, Finalize, JsData)]
pub(crate) struct RecordingLogger {
    /// Also send logs to this logger.
    tee: Gc<Box<dyn Logger>>,

    #[unsafe_ignore_trace]
    inner: Rc<RefCell<RecordingLoggerInner>>,
}

impl Debug for RecordingLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RecordingLogger { ... }")
    }
}

impl Logger for RecordingLogger {
    fn log(&self, msg: String, state: &ConsoleState, ctx: &Context) -> JsResult<()> {
        self.inner
            .borrow_mut()
            .log
            .push(RecordingLogEvent::new(msg.clone(), state));
        self.tee.log(msg, state, ctx)?;
        Ok(())
    }

    fn info(&self, msg: String, state: &ConsoleState, ctx: &Context) -> JsResult<()> {
        self.inner
            .borrow_mut()
            .log
            .push(RecordingLogEvent::new(msg.clone(), state));
        self.tee.info(msg, state, ctx)?;
        Ok(())
    }

    fn warn(&self, msg: String, state: &ConsoleState, ctx: &Context) -> JsResult<()> {
        self.inner
            .borrow_mut()
            .log
            .push(RecordingLogEvent::new(msg.clone(), state));
        self.tee.warn(msg, state, ctx)?;
        Ok(())
    }

    fn error(&self, msg: String, state: &ConsoleState, ctx: &Context) -> JsResult<()> {
        self.inner
            .borrow_mut()
            .error
            .push(RecordingLogEvent::new(msg.clone(), state));
        self.tee.error(msg, state, ctx)?;
        Ok(())
    }
}

impl RecordingLogger {
    pub(crate) fn new<L: Logger + 'static>(tee: L) -> Self {
        Self {
            tee: Gc::new(Box::new(tee)),
            inner: Rc::new(RefCell::new(Default::default())),
        }
    }
}

impl RecordingLogger {
    pub(crate) fn all_logs(&self) -> Vec<RecordingLogEvent> {
        let mut all: Vec<RecordingLogEvent> = self.log().into_iter().chain(self.error()).collect();
        all.sort_by_key(|x| x.index);
        all
    }

    pub(crate) fn log(&self) -> Vec<RecordingLogEvent> {
        self.inner.borrow().log.clone()
    }

    pub(crate) fn error(&self) -> Vec<RecordingLogEvent> {
        self.inner.borrow().error.clone()
    }
}
