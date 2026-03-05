use crate::{
    Context, JsValue,
    context::Clock,
    job::{Job, JobExecutor, TimeoutJob},
};
use std::cell::Cell;
use std::rc::Rc;

struct FrozenClock;

impl Clock for FrozenClock {
    fn now(&self) -> crate::job::JsInstant {
        // Freeze time at exactly 100ms
        crate::job::JsInstant::new(0, 100_000_000)
    }
}

#[test]
fn timeout_runs_at_exact_time() {
    let mut context = Context::builder()
        .clock(Rc::new(FrozenClock))
        .build()
        .unwrap();

    let ran = Rc::new(Cell::new(false));
    let ran_clone = ran.clone();

    // 1. Create a job scheduled for exactly 'now' (0ms offset).
    // The executor resolves scheduled time as: scheduled_for = now + timeout.
    // So scheduled_for = 100 + 0 = 100.
    let job = TimeoutJob::from_duration(
        move |_| {
            ran_clone.set(true);
            Ok(JsValue::undefined())
        },
        std::time::Duration::from_millis(0),
    );

    let executor = crate::job::SimpleJobExecutor::new();
    let executor_rc = Rc::new(executor);

    executor_rc
        .clone()
        .enqueue_job(Job::from(job), &mut context);

    // 2. Run jobs. At this exact moment, `now` is still exactly 100.
    // Because of `BTreeMap::split_off(&now)` where now=100, the job (key=100)
    // is placed into `>= 100` (`jobs_to_keep`). Thus, it doesn't run!
    executor_rc.clone().run_jobs(&mut context).unwrap();

    // If the bug exists, this will Panic, showing that the job failed to run.
    assert!(
        ran.get(),
        "Timeout job scheduled at exactly 'now' did NOT execute! Bug confirmed: off-by-one in split_off"
    );
}

#[test]
fn cancelled_timeout_should_not_execute() {
    let mut context = Context::default();
    let ran = Rc::new(Cell::new(false));
    let ran_clone = ran.clone();

    let job = TimeoutJob::from_duration(
        move |_| {
            ran_clone.set(true);
            Ok(JsValue::undefined())
        },
        std::time::Duration::from_millis(0),
    );

    let flag = job.cancelled_flag();

    let executor = crate::job::SimpleJobExecutor::new();
    let executor_rc = Rc::new(executor);

    executor_rc
        .clone()
        .enqueue_job(Job::from(job), &mut context);

    // Advance time realistically using system sleep so it becomes "< now"
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Cancel the job!
    flag.set();

    // Run jobs
    executor_rc.clone().run_jobs(&mut context).unwrap();

    if ran.get() {
        panic!("Cancelled timeout job executed anyway! Bug confirmed.");
    }
}
