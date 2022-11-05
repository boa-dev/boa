use crate::{prelude::JsObject, Context, JsResult, JsValue};
use boa_gc::{Finalize, Trace};

/// `JobCallback` records
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-jobcallback-records
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JobCallback {
    callback: JsObject,
}

impl JobCallback {
    /// `HostMakeJobCallback ( callback )`
    ///
    /// The host-defined abstract operation `HostMakeJobCallback` takes argument `callback` (a
    /// function object) and returns a `JobCallback` Record.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hostmakejobcallback
    pub fn make_job_callback(callback: JsObject) -> Self {
        // 1. Return the JobCallback Record { [[Callback]]: callback, [[HostDefined]]: empty }.
        Self { callback }
    }

    /// `HostCallJobCallback ( jobCallback, V, argumentsList )`
    ///
    /// The host-defined abstract operation `HostCallJobCallback` takes arguments `jobCallback` (a
    /// `JobCallback` Record), `V` (an ECMAScript language value), and `argumentsList` (a `List` of
    /// ECMAScript language values) and returns either a normal completion containing an ECMAScript
    /// language value or a throw completion.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hostcalljobcallback
    pub fn call_job_callback(
        &self,
        v: &JsValue,
        arguments_list: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // It must perform and return the result of Call(jobCallback.[[Callback]], V, argumentsList).

        // 1. Assert: IsCallable(jobCallback.[[Callback]]) is true.
        assert!(
            self.callback.is_callable(),
            "the callback of the job callback was not callable"
        );

        // 2. Return ? Call(jobCallback.[[Callback]], V, argumentsList).
        self.callback.__call__(v, arguments_list, context)
    }
}
