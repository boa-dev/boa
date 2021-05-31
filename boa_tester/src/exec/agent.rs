use boa::{
    object::{GcObject, ObjectInitializer},
    Context, Result, Value,
};

/// Creates the `agent` object.
pub(super) fn init(context: &mut Context) -> GcObject {
    ObjectInitializer::new(context)
        // .function(start, "start", 1)
        // .function(broadcast, "broadcast", 2)
        // .function(get_report, "getReport", 0)
        .function(sleep, "sleep", 1)
        // .function(monotonic_now, "monotonicNow", 1)
        .build()
}

/// Creates the "receiver" `agent` object.
#[allow(dead_code)]
pub(super) fn init_receiver(context: &mut Context) -> GcObject {
    ObjectInitializer::new(context)
        // .function(receive_broadcast, "receiveBroadcast", 1)
        // .function(report, "report", 1)
        .function(sleep, "sleep", 1)
        // .function(leaving, "leaving", 0)
        // .function(monotonic_now, "monotonicNow", 1)
        .build()
}

/// Takes a script source string and runs the script in a concurrent agent. Will block until that agent is running.
#[allow(dead_code)]
fn start(_this: &Value, _args: &[Value], _context: &mut Context) -> Result<Value> {
    todo!()
}

/// Takes a millisecond argument and sleeps the execution for approximately that duration.
fn sleep(_this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
    use std::time::Duration;

    let milliseconds = args.get(0).cloned().unwrap_or_default().to_u32(context)?;
    std::thread::sleep(Duration::from_millis(u64::from(milliseconds)));

    Ok(Value::default())
}
