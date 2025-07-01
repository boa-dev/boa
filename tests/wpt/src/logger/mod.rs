use boa_engine::{Context, Finalize, JsData, JsResult, Trace};
use boa_runtime::{ConsoleState, Logger};
use std::cell::RefCell;
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

#[derive(Trace, Finalize, JsData)]
struct RecordingLoggerInner {
    pub log: Vec<RecordingLogEvent>,
    pub error: Vec<RecordingLogEvent>,
}

#[derive(Clone, Trace, Finalize, JsData)]
pub(crate) struct RecordingLogger {
    #[unsafe_ignore_trace]
    inner: Rc<RefCell<RecordingLoggerInner>>,
}

impl Logger for RecordingLogger {
    fn log(&self, msg: String, state: &ConsoleState, _: &mut Context) -> JsResult<()> {
        self.inner
            .borrow_mut()
            .log
            .push(RecordingLogEvent::new(msg, state));
        Ok(())
    }

    fn info(&self, msg: String, state: &ConsoleState, _: &mut Context) -> JsResult<()> {
        self.inner
            .borrow_mut()
            .log
            .push(RecordingLogEvent::new(msg, state));
        Ok(())
    }

    fn warn(&self, msg: String, state: &ConsoleState, _: &mut Context) -> JsResult<()> {
        self.inner
            .borrow_mut()
            .log
            .push(RecordingLogEvent::new(msg, state));
        Ok(())
    }

    fn error(&self, msg: String, state: &ConsoleState, _: &mut Context) -> JsResult<()> {
        self.inner
            .borrow_mut()
            .error
            .push(RecordingLogEvent::new(msg, state));
        Ok(())
    }
}

impl RecordingLogger {
    pub(crate) fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(RecordingLoggerInner {
                log: Vec::new(),
                error: Vec::new(),
            })),
        }
    }

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
